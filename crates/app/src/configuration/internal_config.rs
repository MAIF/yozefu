//! module defining the configuration of the yozefu application

use std::{collections::HashMap, path::PathBuf};

use chrono::Local;

use crate::configuration::{GlobalConfig, SchemaRegistryConfig};

use super::{Configuration, yozefu_config::YozefuConfig};

#[derive(Debug, Clone)]
pub struct InternalConfig {
    specific: YozefuConfig,
    pub global: GlobalConfig,
    output_file: PathBuf,
}

impl Configuration for InternalConfig {
    fn kafka_config_map(&self) -> HashMap<String, String> {
        self.specific.kafka_config_map()
    }
}

impl InternalConfig {
    pub fn new(specific: YozefuConfig, global: GlobalConfig) -> Self {
        let directory = match &specific.export_directory {
            Some(e) => e,
            None => &global.export_directory,
        }
        .clone();

        let output_file = directory.join(format!(
            "export-{}.json",
            // Windows does not support ':' in filenames
            Local::now()
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
                .replace(':', "-"),
        ));

        Self {
            specific,
            global,
            output_file,
        }
    }

    /// web URL template for a given cluster
    pub fn url_template_of(&self, cluster: &str) -> String {
        match &self.specific.url_template() {
            Some(url) => url.to_string(),
            None => self.global.url_template_of(cluster),
        }
    }

    /// Returns the schema registry configuration for the given cluster.
    pub fn schema_registry_config_of(&self, cluster: &str) -> Option<SchemaRegistryConfig> {
        match &self.specific.schema_registry() {
            Some(schema_registry) => Some(schema_registry.clone()),
            None => self.global.schema_registry_config_of(cluster),
        }
    }

    /// Returns the output file path for exported kafka records.
    pub fn output_file(&self) -> &PathBuf {
        &self.output_file
    }
}
