use app::configuration::{ClusterConfig, YozefuConfig};
use clap::Parser;
use indexmap::IndexMap;
use rdkafka::ClientConfig;
use strum::{Display, EnumIter, EnumString};
use tui::TuiError;
use yozefu_command::Cli;

#[derive(Debug, Clone, PartialEq, Eq, Display, EnumString, EnumIter)]
#[strum(serialize_all = "lowercase")]
enum Cluster {
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
    pub fn kafka_client_config(&self) -> ClusterConfig {
        let mut config = ClientConfig::new();
        config.set_log_level(rdkafka::config::RDKafkaLogLevel::Emerg);
        if let Some(cluster) = self.command.cluster() {
            match cluster {
                Cluster::Localhost => {
                    config.set("bootstrap.servers", "kafka-localhost.acme:9092".to_string())
                }
                Cluster::Test => {
                    config.set("bootstrap.servers", "kafka-test.acme:9092".to_string())
                }
                Cluster::Development => config.set(
                    "bootstrap.servers",
                    "kafka-development.acme:9092".to_string(),
                ),
                Cluster::Production => config.set(
                    "bootstrap.servers",
                    "kafka-production.acme:9092".to_string(),
                ),
            };
        }

        ClusterConfig {
            url_template: None,
            schema_registry: None,
            kafka: IndexMap::from_iter(config.config_map().clone()),
        }
    }

    pub async fn execute(&self) -> Result<(), TuiError> {
        let yozefu_config = YozefuConfig::new(self.kafka_client_config());
        self.command.execute_with(yozefu_config).await
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let parsed = MyCli::parse();
    parsed.execute().await.map_err(|e| e.to_string())
}
