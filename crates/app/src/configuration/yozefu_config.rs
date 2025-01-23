//! module defining the configuration of the yozefu application

use super::{Configuration, SchemaRegistryConfig};
use crate::{configuration::ClusterConfig, APPLICATION_NAME};
use std::{collections::HashMap, path::PathBuf};

/// composed of kafka properties and
/// an optional user-specific configuration.
#[derive(Debug, Clone)]
pub struct YozefuConfig {
    cluster_config: ClusterConfig,
    pub logs_file: Option<PathBuf>,
    pub export_directory: Option<PathBuf>,
}

impl YozefuConfig {
    pub fn new(cluster_config: ClusterConfig) -> Self {
        Self {
            cluster_config: cluster_config.normalize_paths(),
            logs_file: None,
            export_directory: None,
        }
    }

    pub fn url_template(&self) -> Option<String> {
        self.cluster_config.url_template.clone()
    }

    pub fn schema_registry(&self) -> Option<SchemaRegistryConfig> {
        self.cluster_config.schema_registry.clone()
    }

    pub fn with_exported_directory(self, exported_directory: PathBuf) -> Self {
        Self {
            cluster_config: self.cluster_config,
            logs_file: self.logs_file,
            export_directory: Some(exported_directory),
        }
    }

    pub fn with_logs_file(self, logs_file: PathBuf) -> Self {
        Self {
            cluster_config: self.cluster_config,
            logs_file: Some(logs_file),
            export_directory: self.export_directory,
        }
    }

    pub fn set_kafka_property(&mut self, key: &str, value: &str) {
        self.cluster_config
            .kafka
            .insert(key.to_string(), value.to_string());
    }

    /// Overrides the kafka properties with the properties provided by the user
    pub fn update_kafka_properties(self, kafka_properties: HashMap<String, String>) -> Self {
        Self {
            cluster_config: ClusterConfig {
                url_template: None,
                schema_registry: None,
                kafka: indexmap::IndexMap::from_iter(kafka_properties),
            },
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
