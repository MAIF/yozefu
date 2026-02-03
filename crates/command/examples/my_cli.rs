//! This module shows you how to include yozefu to your own CLI.

use std::collections::HashMap;

use app::configuration::{ClusterConfig, ConsumerConfig, YozefuConfig};
use clap::Parser;
use indexmap::IndexMap;
use strum::{Display, EnumIter, EnumString};
use tui::TuiError;
use yozefu_command::Cli;

/// I have 4 kafka clusters
#[derive(Debug, Clone, PartialEq, Eq, Display, EnumString, EnumIter, Default)]
#[strum(serialize_all = "lowercase")]
enum Cluster {
    #[default]
    Localhost,
    Test,
    Development,
    Production,
}

#[derive(Parser)]
struct MyCli {
    #[clap(flatten)]
    command: Cli<Cluster>,
}

impl MyCli {
    pub fn yozefu_config(&self) -> YozefuConfig {
        let cluster = self.command.cluster().unwrap_or_default();
        let url = match cluster {
            Cluster::Localhost => "kafka.localhost.acme:9092",
            Cluster::Test => "kafka.test.acme:9092",
            Cluster::Development => "kafka.development.acme:9092",
            Cluster::Production => "kafka.production.acme:9092",
        };

        let mut config = HashMap::new();
        config.insert("bootstrap.servers".to_string(), url.to_string());
        ClusterConfig {
            url_template: None,
            schema_registry: None,
            kafka: IndexMap::from_iter(config),
            consumer: Some(ConsumerConfig {
                buffer_capacity: 1000,
                timeout_in_ms: 100,
            }),
        }
        .create(&cluster.to_string())
    }

    pub async fn execute(&self) -> Result<(), TuiError> {
        // To pass your configuration, create a `YozefuConfig`.
        let yozefu_config = self.yozefu_config();
        // And pass it to the `yozefu_command::Cli`
        self.command.execute_with(yozefu_config).await
    }
}

/// Yozefu uses an async runtime
#[tokio::main]
async fn main() -> Result<(), String> {
    let parsed = MyCli::parse();
    parsed.execute().await.map_err(|e| e.to_string())
}
