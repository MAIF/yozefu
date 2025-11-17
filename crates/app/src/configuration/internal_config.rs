//! module defining the configuration of the yozefu application

use std::{collections::HashMap, fs, path::PathBuf};

use chrono::Local;
use lib::Error;

use crate::configuration::{ConsumerConfig, SchemaRegistryConfig, Workspace};

use super::{Configuration, yozefu_config::YozefuConfig};

#[derive(Debug, Clone)]
pub struct InternalConfig {
    pub specific: YozefuConfig,
    workspace: Workspace,
    output_file: PathBuf,
}

impl Configuration for InternalConfig {
    fn kafka_config_map(&self) -> HashMap<String, String> {
        let mut config_map: HashMap<String, String> = self
            .workspace
            .config()
            .default_kafka_config
            .clone()
            .into_iter()
            .collect();
        config_map.extend(self.specific.kafka_config_map());
        config_map
    }
}

impl InternalConfig {
    pub fn new(specific: YozefuConfig, workspace: Workspace) -> Self {
        let directory = match &specific.export_directory {
            Some(e) => e,
            None => &workspace.config().export_directory,
        }
        .clone();

        let output_file = directory.join(format!(
            "export-{}.json",
            // Windows does not support ':' in filenames
            Local::now()
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
                .replace(':', "-"),
        ));

        let output_file = std::path::absolute(output_file)
            .expect("Failed to get absolute path for the export file");

        Self {
            specific,
            workspace,
            output_file,
        }
    }

    /// web URL template for a given cluster
    pub fn url_template_of(&self, cluster: &str) -> String {
        match &self.specific.url_template() {
            Some(url) => url.to_string(),
            None => self.workspace.config().url_template_of(cluster),
        }
    }

    pub fn cluster(&self) -> &str {
        self.specific.cluster()
    }

    /// Consumer configuration for the given cluster.
    pub fn consumer_config(&self, cluster: &str) -> ConsumerConfig {
        self.workspace.config().consumer_config_of(cluster)
    }

    /// Returns the schema registry configuration for the given cluster.
    pub fn schema_registry_config_of(&self, cluster: &str) -> Option<SchemaRegistryConfig> {
        match &self.specific.schema_registry() {
            Some(schema_registry) => Some(schema_registry.clone()),
            None => self.workspace.config().schema_registry_config_of(cluster),
        }
    }

    /// Returns the output file path for exported kafka records.
    pub fn output_file(&self) -> &PathBuf {
        &self.output_file
    }

    pub fn workspace(&self) -> &Workspace {
        &self.workspace
    }

    pub fn history(&self) -> &[String] {
        &self.workspace.config().history
    }

    pub fn push_history(&mut self, prompt: &str) {
        self.workspace.config.history.push(prompt.to_string());
        self.workspace.config.history.dedup();
    }

    pub fn initial_query(&self) -> &str {
        &self.workspace.config.initial_query
    }

    pub fn theme(&self) -> &str {
        &self.workspace.config.theme
    }

    pub fn save_config(&mut self) -> Result<(), Error> {
        let history = &self.workspace.config.history;
        if history.len() > 1000 {
            self.workspace.config.history = history.iter().skip(500).cloned().collect();
        }
        fs::write(
            &self.workspace.config.path,
            serde_json::to_string_pretty(&self.workspace.config)?,
        )?;
        Ok(())
    }
}
