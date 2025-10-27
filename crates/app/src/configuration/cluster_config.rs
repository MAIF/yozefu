//! module defining the configuration structure of the application

use indexmap::IndexMap;
use resolve_path::PathResolveExt;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use url::Url;

use crate::configuration::{ConsumerConfig, YozefuConfig};

use super::Configuration;

/// List of kafka properties that are a file location.
pub const KAFKA_PROPERTIES_WITH_LOCATIONS: [&str; 6] = [
    "ssl.ca.location",
    "ssl.certificate.location",
    "ssl.key.location",
    "ssl.keystore.location",
    "ssl.crl.location",
    "ssl.engine.location",
];

/// List of kafka properties that are a file location.
pub const SENSITIVE_KAFKA_PROPERTIES: [&str; 3] =
    ["sasl.password", "ssl.key.password", "ssl.keystore.password"];

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            url_template: None,
            schema_registry: None,
            kafka: IndexMap::new(),
            consumer: None,
        }
    }
}

/// Specific configuration for a cluster
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub struct ClusterConfig {
    /// A placeholder url that will be used when you want to open a kafka record in the browser
    pub url_template: Option<String>,
    /// Schema registry configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_registry: Option<SchemaRegistryConfig>,
    /// Kafka consumer properties for this cluster, see <https://github.com/confluentinc/librdkafka/blob/master/CONFIGURATION.md> for more details
    pub kafka: IndexMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumer: Option<ConsumerConfig>,
}

impl ClusterConfig {
    /// Normalize all properties that are file locations.
    /// For instance, `~/certificates/ca.pem` will be resolved to `/home/user/certificates/ca.pem`.
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

    //    // cluster is something that can be converted to &str, must be a generic though
    //    pub fn create<T>(self, cluster: T) -> ClusterConfig
    //    where
    //        T: ToString,
    //    {
    //        ClusterConfig {
    //            cluster: cluster.to_string(),
    //            config: self.normalize_paths(),
    //        }
    //    }
}

/// Schema registry configuration of a given cluster
#[derive(Debug, Deserialize, PartialEq, Eq, Serialize, Clone)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub struct SchemaRegistryConfig {
    /// Url of the schema registry
    pub url: Url,
    /// HTTP headers to be used when communicating with the schema registry
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

impl Configuration for ClusterConfig {
    fn kafka_config_map(&self) -> HashMap<String, String> {
        let mut properties = HashMap::new();
        properties.extend(self.kafka.clone());
        properties
    }
}

impl ClusterConfig {
    pub fn with_kafka_properties(self, kafka_properties: HashMap<String, String>) -> Self {
        Self {
            url_template: None,
            schema_registry: None,
            kafka: indexmap::IndexMap::from_iter(kafka_properties),
            consumer: self.consumer,
        }
    }

    pub fn set_kafka_property(&mut self, key: &str, value: &str) {
        self.kafka.insert(key.to_string(), value.to_string());
    }

    pub fn create(self, cluster: &str) -> YozefuConfig {
        YozefuConfig::new(cluster, self)
    }
}
//
