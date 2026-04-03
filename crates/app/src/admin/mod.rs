use std::{collections::HashSet, time::Duration};

use itertools::Itertools;
use lib::{ConsumerGroupDetail, Error, TopicConfig, TopicDetail};
use rdkafka::{
    Offset, TopicPartitionList,
    admin::{AdminClient as RDAdminClient, AdminOptions, ResourceSpecifier},
    client::DefaultClientContext,
    config::FromClientConfigAndContext,
    consumer::{BaseConsumer, Consumer, StreamConsumer},
};
use tracing::warn;

use crate::configuration::{Configuration, InternalConfig};

pub struct AdminClient {
    client: RDAdminClient<DefaultClientContext>,
    config: InternalConfig,
    options: AdminOptions,
}

impl AdminClient {
    pub fn new(config: InternalConfig) -> Result<Self, Error> {
        let client =
            RDAdminClient::from_config_and_context(&config.client_config(), DefaultClientContext)?;
        let options = AdminOptions::new();
        Ok(Self {
            client,
            config,
            options,
        })
    }
    /// Loads the configuration details for the specified topic from the Kafka cluster.
    pub async fn topic_config(&self, topic: &str) -> Result<Option<TopicConfig>, Error> {
        let resource = ResourceSpecifier::Topic(topic);
        let result = self
            .client
            .describe_configs(&[resource], &self.options)
            .await?;

        if result.is_empty() {
            return Ok(None);
        }

        match result.first() {
            Some(Ok(c)) => Ok(Some(c.into())),
            Some(Err(e)) => Err(Error::Error(e.to_string())),
            None => Ok(None),
        }
    }

    /// Returns the topics details for a given list topics
    /// This function is not ready yet
    pub fn topic_details(&self, topics: HashSet<String>) -> Result<Vec<TopicDetail>, Error> {
        let mut results = vec![];
        for topic in topics {
            let consumer: BaseConsumer = self.config.create_kafka_consumer()?;
            let metadata = consumer.fetch_metadata(Some(&topic), Duration::from_secs(10))?;
            let metadata = metadata.topics().first().unwrap();
            let mut detail = TopicDetail {
                name: topic.clone(),
                replicas: metadata.partitions().first().unwrap().replicas().len(),
                partitions: metadata.partitions().len(),
                consumer_groups: vec![],
                count: self.count_records_in_topic(&topic)?,
                config: None,
            };
            let mut consumer_groups = vec![];
            let metadata = consumer.fetch_group_list(None, Duration::from_secs(10))?;
            for g in metadata.groups() {
                consumer_groups.push(ConsumerGroupDetail {
                    name: g.name().to_string(),
                    members: vec![], //Self::parse_members(g, g.members())?,
                    state: g.state().parse()?,
                });
            }
            detail.consumer_groups = consumer_groups;
            results.push(detail);
        }

        Ok(results)
    }

    pub fn estimate_number_of_records_to_read(
        &self,
        topic_partition_list: &TopicPartitionList,
    ) -> Result<i64, Error> {
        let client: StreamConsumer = self.config.create_kafka_consumer()?;
        let mut count = 0;
        for t in topic_partition_list.elements() {
            // this function call be very slow
            let watermarks: (i64, i64) =
                match client.fetch_watermarks(t.topic(), t.partition(), Duration::from_secs(10)) {
                    Ok(i) => i,
                    Err(e) => {
                        warn!(
                            "I was not able to fetch watermarks of topic '{}', partition {}: {}",
                            t.topic(),
                            t.partition(),
                            e
                        );
                        (0, 0)
                    }
                };
            count += match t.offset() {
                Offset::Beginning => watermarks.1 - watermarks.0,
                Offset::End => 0,
                Offset::Stored => 1,
                Offset::Invalid => 1,
                Offset::Offset(o) => watermarks.1 - o,
                Offset::OffsetTail(o) => o,
            }
        }
        Ok(count)
    }

    fn count_records_in_topic(&self, topic: &str) -> Result<i64, Error> {
        let mut count = 0;
        let consumer: BaseConsumer = self.config.create_kafka_consumer()?;
        let metadata = consumer.fetch_metadata(Some(topic), Duration::from_secs(10))?;
        let metadata_topic = metadata.topics().first();
        if metadata_topic.is_none() {
            return Ok(0);
        }

        let metadata_topic = metadata_topic.unwrap();
        for partition in metadata_topic.partitions() {
            let watermarks =
                consumer.fetch_watermarks(topic, partition.id(), Duration::from_secs(10))?;
            count += watermarks.1 - watermarks.0;
        }

        Ok(count)
    }

    pub fn list_topics(&self) -> Result<Vec<String>, Error> {
        let consumer: StreamConsumer = self.config.create_kafka_consumer()?;
        let metadata = consumer.fetch_metadata(None, Duration::from_secs(10))?;
        let topics = metadata
            .topics()
            .iter()
            .map(|t| t.name().to_string())
            .collect_vec();
        Ok(topics)
    }
}
