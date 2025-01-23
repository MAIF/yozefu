//! module defining the configuration structure of the application

use indexmap::IndexMap;
use resolve_path::PathResolveExt;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use url::Url;

use super::Configuration;

/// List of kafka properties that are a file location.
const KAFKA_PROPERTIES_WITH_LOCATIONS: [&str; 6] = [
    "ssl.ca.location",
    "ssl.certificate.location",
    "ssl.key.location",
    "ssl.keystore.location",
    "ssl.crl.location",
    "ssl.engine.location",
];

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            url_template: Some(default_url_template()),
            schema_registry: None,
            kafka: Default::default(),
        }
    }
}

/// Specific configuration for a cluster
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct ClusterConfig {
    /// A placeholder url that will be used when you want to open a kafka record in the browser
    pub url_template: Option<String>,
    /// Schema registry configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_registry: Option<SchemaRegistryConfig>,
    // Kafka consumer properties for this cluster, see <https://github.com/confluentinc/librdkafka/blob/master/CONFIGURATION.md> for more details
    pub kafka: IndexMap<String, String>,
}

impl ClusterConfig {
    /// Returns the kafka properties for the given cluster.
    pub fn normalize_paths(self) -> Self {
        let mut cloned = self.clone();
        for key in KAFKA_PROPERTIES_WITH_LOCATIONS {
            if let Some(path) = cloned.kafka.get(key) {
                let normalized_path = PathBuf::from(path)
                    .resolve()
                    .canonicalize()
                    .map(|d| d.display().to_string())
                    .unwrap_or(path.to_string());
                cloned.kafka.insert(key.to_string(), normalized_path);
            }
        }
        cloned
    }
}

/// Schema registry configuration of a given cluster
#[derive(Debug, Deserialize, PartialEq, Eq, Serialize, Clone)]
pub struct SchemaRegistryConfig {
    /// Url of the schema registry
    pub url: Url,
    /// HTTP headers to be used when communicating with the schema registry
    #[serde(default = "HashMap::default")]
    pub headers: HashMap<String, String>,
}

fn default_url_template() -> String {
    "http://localhost/cluster/{topic}/{partition}/{offset}".to_string()
}

impl Configuration for ClusterConfig {
    fn kafka_config_map(&self) -> HashMap<String, String> {
        let mut properties = HashMap::new();
        properties.extend(self.kafka.clone());
        properties
    }
}
