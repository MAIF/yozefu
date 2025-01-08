//! module defining the configuration structure of the application

use indexmap::IndexMap;
use itertools::Itertools;
use lib::Error;
use resolve_path::PathResolveExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use url::Url;

use crate::APPLICATION_NAME;

/// List of kafka properties that are a file location.
const KAFKA_PROPERTIES_WITH_LOCATIONS: [&str; 6] = [
    "ssl.ca.location",
    "ssl.certificate.location",
    "ssl.key.location",
    "ssl.keystore.location",
    "ssl.crl.location",
    "ssl.engine.location",
];

const EXAMPLE_PROMPTS: &[&str] = &[
    r#"timestamp between "2 hours ago" and "1 hour ago" limit 100 from beginning"#,
    r#"offset > 100000 and value contains "music" limit 10"#,
    r#"key == "ABC" and timestamp >= "2 days ago""#,
];

/// Configuration of the application
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct Config {
    /// Path of the config file
    #[serde(skip)]
    pub path: PathBuf,
    /// A placeholder url that will be used when you want to open a kafka record in the browser
    #[serde(default = "default_url_template")]
    pub default_url_template: String,
    /// The initial search query when you start the UI
    pub initial_query: String,
    #[serde(default = "default_theme")]
    pub theme: String,
    /// The kafka properties for each cluster
    pub clusters: IndexMap<String, ClusterConfig>,
    /// The default kafka properties inherited for every cluster
    pub default_kafka_config: IndexMap<String, String>,
    /// History of past search queries
    pub history: Vec<String>,
    /// Show shortcuts
    #[serde(default = "default_show_shortcuts")]
    pub show_shortcuts: bool,
    #[serde(default = "default_export_directory")]
    pub export_directory: PathBuf,
}

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

fn default_export_directory() -> PathBuf {
    PathBuf::from(format!("./{}-exports", APPLICATION_NAME))
}

fn default_theme() -> String {
    "light".to_string()
}

fn default_show_shortcuts() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        let mut kafka_config = IndexMap::new();
        kafka_config.insert("fetch.max.bytes".to_string(), "3000000".to_string());
        Self {
            path: PathBuf::default(),
            default_url_template: default_url_template(),
            history: EXAMPLE_PROMPTS.iter().map(|e| e.to_string()).collect_vec(),
            initial_query: "from end - 10".to_string(),
            clusters: IndexMap::default(),
            default_kafka_config: IndexMap::default(),
            theme: default_theme(),
            show_shortcuts: true,
            export_directory: default_export_directory(),
        }
    }
}

impl Config {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            ..Default::default()
        }
    }

    /// Reads a configuration file.
    pub fn read(file: &Path) -> Result<Config, Error> {
        let content = fs::read_to_string(file)?;
        let mut config: Config = serde_json::from_str(&content)?;
        config.path = file.to_path_buf();
        Ok(config)
    }

    /// Returns the name of the logs file
    pub fn logs_file(&self) -> PathBuf {
        self.path.parent().unwrap().join("application.log")
    }

    /// Returns the name of the logs file
    pub fn themes_file(&self) -> PathBuf {
        self.path.parent().unwrap().join("themes.json")
    }

    /// Returns the list of available theme names.
    pub fn themes(&self) -> Vec<String> {
        let file = self.themes_file();
        let content = fs::read_to_string(file).unwrap_or("{}".to_string());
        let themes: HashMap<String, Value> = serde_json::from_str(&content).unwrap_or_default();
        themes.keys().map(|e| e.to_string()).collect_vec()
    }

    /// Returns the name of the directory containing wasm filters
    pub fn filters_dir(&self) -> PathBuf {
        let dir = self.path.parent().unwrap().join("filters");
        let _ = fs::create_dir_all(&dir);
        dir
    }

    /// web URL template for a given cluster
    pub fn url_template_of(&self, cluster: &str) -> String {
        self.clusters
            .get(cluster)
            .and_then(|e| e.url_template.clone())
            .unwrap_or(self.default_url_template.clone())
    }

    /// Returns the kafka properties for the given cluster.
    pub fn kafka_config_of(&self, cluster: &str) -> Result<HashMap<String, String>, Error> {
        let mut config = HashMap::new();
        config.extend(self.default_kafka_config.clone());

        if !self.clusters.contains_key(cluster) {
            return Err(Error::Error(format!(
                "I was not able to find the '{}' cluster. Available clusters are: [{}]",
                cluster,
                self.clusters.keys().join(", ")
            )));
        }

        let env_config = self.clusters.get(cluster.trim()).unwrap();
        config.extend(env_config.kafka.clone());
        config = Self::normalize_path_locations(config);

        Ok(config)
    }

    /// Returns the kafka properties for the given cluster.
    fn normalize_path_locations(mut config: HashMap<String, String>) -> HashMap<String, String> {
        for key in KAFKA_PROPERTIES_WITH_LOCATIONS {
            if let Some(path) = config.get(key) {
                let normalized_path = PathBuf::from(path)
                    .resolve()
                    .canonicalize()
                    .map(|d| d.display().to_string())
                    .unwrap_or(path.to_string());
                config.insert(key.to_string(), normalized_path);
            }
        }
        config
    }

    /// Returns the schema registry configuration for the given cluster.
    pub fn schema_registry_config_of(&self, cluster: &str) -> Option<SchemaRegistryConfig> {
        self.clusters
            .get(cluster.trim())
            .and_then(|config| config.schema_registry.clone())
    }
}
