use std::collections::HashMap;

use lib::Error;
use rdkafka::{ClientConfig, config::FromClientConfig};

mod cluster_config;
mod consumer_config;
mod global_config;
mod internal_config;
mod workspace;
mod yozefu_config;

pub use cluster_config::ClusterConfig;
pub use cluster_config::KAFKA_PROPERTIES_WITH_LOCATIONS;
pub use cluster_config::SENSITIVE_KAFKA_PROPERTIES;
pub use cluster_config::SchemaRegistryConfig;
pub use consumer_config::ConsumerConfig;
pub use global_config::GlobalConfig;
pub use global_config::TimestampFormat;
pub use internal_config::InternalConfig;
use tracing::debug;
use tracing::enabled;
pub use workspace::Workspace;
pub use yozefu_config::YozefuConfig;

pub trait Configuration {
    /// Returns the kafka properties
    fn kafka_config_map(&self) -> HashMap<String, String>;

    /// Properties you can set for the kafka consumer: <https://github.com/confluentinc/librdkafka/blob/master/CONFIGURATION.md>
    fn create_kafka_consumer<T>(&self) -> Result<T, Error>
    where
        T: FromClientConfig,
    {
        self.client_config()
            .create()
            .map_err(std::convert::Into::into)
    }

    /// Properties you can set for the kafka consumer: <https://github.com/confluentinc/librdkafka/blob/master/CONFIGURATION.md>
    fn client_config(&self) -> ClientConfig {
        Self::kafka_client_config_from_properties(self.kafka_config_map().clone())
    }

    fn kafka_client_config_from_properties(
        kafka_properties: HashMap<String, String>,
    ) -> ClientConfig {
        let mut config = ClientConfig::new();
        config.set_log_level(rdkafka::config::RDKafkaLogLevel::Emerg);
        for (key, value) in kafka_properties {
            config.set(key, value);
        }

        if enabled!(tracing::Level::DEBUG) {
            config.set("debug", "consumer,cgrp,topic");
            for (k, v) in config.config_map().iter() {
                debug!("'{}' set to '{}'", k, v);
            }
        }

        config
    }
}
