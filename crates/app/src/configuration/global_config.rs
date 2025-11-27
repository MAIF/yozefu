//! module defining the configuration structure of the application

use std::{
    fs,
    path::{Path, PathBuf},
};

use indexmap::IndexMap;
use itertools::Itertools;
use lib::Error;
use serde::{Deserialize, Serialize};

use crate::{
    APPLICATION_NAME,
    configuration::{ClusterConfig, ConsumerConfig},
};

use super::cluster_config::SchemaRegistryConfig;

const EXAMPLE_PROMPTS: &[&str] = &[
    r#"timestamp between "2 hours ago" and "1 hour ago" limit 100 from beginning"#,
    r#"offset > 100000 and value contains "music" limit 10"#,
    r#"key == "ABC" and timestamp >= "2 days ago""#,
];

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Default)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub enum TimestampFormat {
    #[default]
    DateTime,
    Ago,
}

/// Configuration of the application
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub struct GlobalConfig {
    /// Path of this config
    #[serde(skip)]
    pub path: PathBuf,
    /// A placeholder url that will be used when you want to open a kafka record in the browser
    #[serde(default = "default_url_template")]
    pub default_url_template: String,
    /// The initial search query when you start the UI
    pub initial_query: String,
    /// The theme to use in the TUI
    #[serde(default = "default_theme")]
    pub theme: String,
    /// The theme to use for syntax highlighting
    pub highlighter_theme: Option<String>,
    /// The kafka properties for each cluster
    pub clusters: IndexMap<String, ClusterConfig>,
    #[serde(default)]
    /// The default configuration for the yozefu kafka consumer
    pub consumer: ConsumerConfig,
    /// The default kafka properties inherited for every cluster
    pub default_kafka_config: IndexMap<String, String>,
    /// History of past search queries
    pub history: Vec<String>,
    /// Show shortcuts
    #[serde(default = "default_show_shortcuts")]
    pub show_shortcuts: bool,
    #[serde(default = "default_export_directory")]
    pub export_directory: PathBuf,
    /// The file to write logs to
    pub log_file: Option<PathBuf>,
    /// Show the timestamp as a date time or as "X minutes ago"
    #[serde(default = "TimestampFormat::default")]
    pub timestamp_format: TimestampFormat,
}

fn default_url_template() -> String {
    "http://localhost/cluster/{topic}/{partition}/{offset}".to_string()
}

fn default_export_directory() -> PathBuf {
    PathBuf::from(format!("./{APPLICATION_NAME}-exports"))
}

fn default_theme() -> String {
    if cfg!(target_os = "windows") {
        "dark".to_string()
    } else {
        "light".to_string()
    }
}

fn default_show_shortcuts() -> bool {
    true
}

impl GlobalConfig {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            default_url_template: default_url_template(),
            history: EXAMPLE_PROMPTS
                .iter()
                .map(|e| (*e).to_string())
                .collect_vec(),
            initial_query: "from end - 10".to_string(),
            clusters: IndexMap::default(),
            default_kafka_config: IndexMap::default(),
            theme: default_theme(),
            highlighter_theme: None,
            show_shortcuts: true,
            export_directory: default_export_directory(),
            consumer: ConsumerConfig::default(),
            log_file: None,
            timestamp_format: TimestampFormat::default(),
        }
    }

    /// Reads a configuration file.
    pub fn read(file: &Path) -> Result<Self, Error> {
        let content = fs::read_to_string(file);
        if let Err(e) = &content {
            return Err(Error::Error(format!(
                "Failed to read the configuration file {:?}: {}",
                file.display(),
                e
            )));
        }

        let content = content.unwrap();
        let mut config: Self = serde_json::from_str(&content).map_err(|e| {
            Error::Error(format!(
                "Failed to parse the configuration file {:?}: {}",
                file.display(),
                e
            ))
        })?;
        config.path = file.to_path_buf();
        Ok(config)
    }

    /// Returns the name of the logs file
    pub fn log_file(&self) -> Option<PathBuf> {
        self.log_file.clone()
    }

    /// web URL template for a given cluster
    pub fn url_template_of(&self, cluster: &str) -> String {
        self.clusters
            .get(cluster)
            .and_then(|e| e.url_template.clone())
            .unwrap_or(self.default_url_template.clone())
    }

    /// Consumer config of a given cluster
    pub(crate) fn consumer_config_of(&self, cluster: &str) -> ConsumerConfig {
        self.clusters
            .get(cluster)
            .and_then(|e| e.consumer.clone())
            .unwrap_or(self.consumer.clone())
    }

    /// Returns the schema registry configuration for the given cluster.
    pub fn schema_registry_config_of(&self, cluster: &str) -> Option<SchemaRegistryConfig> {
        self.clusters
            .get(cluster.trim())
            .and_then(|config| config.schema_registry.clone())
    }
}

#[test]
fn generate_json_schema_for_global_config() {
    use schemars::schema_for;
    let mut schema = schema_for!(GlobalConfig);
    schema.insert("$id".into(), "https://raw.githubusercontent.com/MAIF/yozefu/refs/heads/main/docs/json-schemas/global-config.json".into());
    fs::write(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("docs")
            .join("json-schemas")
            .join("global-config.json"),
        serde_json::to_string_pretty(&schema).unwrap(),
    )
    .unwrap();
}
