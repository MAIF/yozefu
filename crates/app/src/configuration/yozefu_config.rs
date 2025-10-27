//! module defining the configuration of the yozefu application

use super::{Configuration, SchemaRegistryConfig};
use crate::{APPLICATION_NAME, configuration::ClusterConfig};
use std::{collections::HashMap, path::PathBuf};

/// composed of kafka properties and
/// an optional user-specific configuration.
#[derive(Debug, Clone)]
pub struct YozefuConfig {
    cluster: String,
    cluster_config: ClusterConfig,
    pub logs_file: Option<PathBuf>,
    pub export_directory: Option<PathBuf>,
}

impl YozefuConfig {
    pub(super) fn new(cluster: &str, cluster_config: ClusterConfig) -> Self {
        Self {
            cluster: cluster.to_string(),
            cluster_config,
            logs_file: None,
            export_directory: None,
        }
    }

    pub fn cluster(&self) -> &str {
        &self.cluster
    }

    pub fn url_template(&self) -> Option<String> {
        self.cluster_config.url_template.clone()
    }

    pub fn config(&self) -> &ClusterConfig {
        &self.cluster_config
    }

    pub fn schema_registry(&self) -> Option<SchemaRegistryConfig> {
        self.cluster_config.schema_registry.clone()
    }

    pub fn with_exported_directory(self, exported_directory: PathBuf) -> Self {
        Self {
            cluster: self.cluster,
            cluster_config: self.cluster_config,
            logs_file: self.logs_file,
            export_directory: Some(exported_directory),
        }
    }

    pub fn with_logs_file(self, logs_file: PathBuf) -> Self {
        Self {
            cluster: self.cluster,
            cluster_config: self.cluster_config,
            logs_file: Some(logs_file),
            export_directory: self.export_directory,
        }
    }

    pub fn set_kafka_property(&mut self, key: &str, value: &str) {
        self.cluster_config.set_kafka_property(key, value);
    }

    /// Overrides the kafka properties with the properties provided by the user
    pub fn update_kafka_properties(self, kafka_properties: HashMap<String, String>) -> Self {
        Self {
            cluster: self.cluster,
            cluster_config: self.cluster_config.with_kafka_properties(kafka_properties),
            logs_file: self.logs_file,
            export_directory: self.export_directory,
        }
    }
}

impl Configuration for YozefuConfig {
    /// Returns the kafka properties
    fn kafka_config_map(&self) -> HashMap<String, String> {
        let mut config_map = self.cluster_config.kafka_config_map();

        // Default properties
        for (key, value) in [
            ("group.id", APPLICATION_NAME),
            ("enable.auto.commit", "false"),
        ] {
            if !config_map.contains_key(key) {
                config_map.insert(key.into(), value.into());
            }
        }
        config_map
    }
}
