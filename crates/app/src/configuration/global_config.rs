//! module defining the configuration structure of the application

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use indexmap::IndexMap;
use itertools::Itertools;
use lib::Error;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{APPLICATION_NAME, configuration::ConsumerConfig};

use super::cluster_config::{ClusterConfig, SchemaRegistryConfig};

const EXAMPLE_PROMPTS: &[&str] = &[
    r#"timestamp between "2 hours ago" and "1 hour ago" limit 100 from beginning"#,
    r#"offset > 100000 and value contains "music" limit 10"#,
    r#"key == "ABC" and timestamp >= "2 days ago""#,
];

/// Configuration of the application
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub struct GlobalConfig {
    /// Path of this config
    #[serde(skip)]
    pub path: PathBuf,
    /// Path to the Yozefu directory containing themes, config, filters...
    #[serde(skip)]
    pub yozefu_directory: PathBuf,
    /// The file to write logs to
    #[serde(skip)]
    pub logs: Option<PathBuf>,
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

impl TryFrom<&PathBuf> for GlobalConfig {
    type Error = Error;

    fn try_from(path: &PathBuf) -> Result<Self, Self::Error> {
        Ok(Self {
            path: path.clone(),
            yozefu_directory: Self::yozefu_directory()?,
            logs: None,
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
        })
    }
}

impl GlobalConfig {
    /// The default config file path
    pub fn path() -> Result<PathBuf, Error> {
        Self::yozefu_directory().map(|d| d.join("config.json"))
    }

    /// The default yozefu directory containing themes, filters, config...
    pub fn yozefu_directory() -> Result<PathBuf, Error> {
        ProjectDirs::from("io", "maif", APPLICATION_NAME)
            .ok_or(Error::Error(
                "Failed to find the yozefu configuration directory".to_string(),
            ))
            .map(|e| e.config_dir().to_path_buf())
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
        config.yozefu_directory = Self::yozefu_directory()?;
        config.path = file.to_path_buf();
        Ok(config)
    }

    /// Returns the name of the logs file
    pub fn logs_file(&self) -> PathBuf {
        self.logs
            .clone()
            .unwrap_or(self.path.parent().unwrap().join("application.log"))
    }

    /// Returns the name of the logs file
    pub fn themes_file(&self) -> PathBuf {
        self.yozefu_directory.join("themes.json")
    }

    /// Returns the list of available theme names.
    pub fn themes(&self) -> Vec<String> {
        let file = self.themes_file();
        let content = fs::read_to_string(file).unwrap_or("{}".to_string());
        let themes: HashMap<String, Value> = serde_json::from_str(&content).unwrap_or_default();
        themes
            .keys()
            .map(std::string::ToString::to_string)
            .collect_vec()
    }

    /// Returns the name of the directory containing wasm filters
    pub fn filters_dir(&self) -> PathBuf {
        let dir = self.yozefu_directory.join("filters");
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
