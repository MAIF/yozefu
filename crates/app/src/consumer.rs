//! A custom Kafka consumer for Yozefu.
//! this module wraps the rdkafka consumer and provides additional functionalities.

use std::time::Duration;

use futures::StreamExt;
use futures_batch::TryChunksTimeoutStreamExt;
use lib::{Error, SearchQuery, search::offset::FromOffset};
use rdkafka::{
    Offset, TopicPartitionList,
    consumer::{Consumer as _, stream_consumer::StreamConsumer},
    message::OwnedMessage,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    configuration::{Configuration, ConsumerConfig, YozefuConfig},
    search::Search,
};

pub struct Consumer {
    config: YozefuConfig,
    consumer_config: ConsumerConfig,
    consumer: StreamConsumer,
}

impl Consumer {
    pub fn new(
        config: YozefuConfig,
        consumer_config: ConsumerConfig,
        query: SearchQuery,
        topics: &Vec<String>,
    ) -> Result<Self, Error> {
        let consumer: StreamConsumer = config.create_kafka_consumer()?;
        let assignments = Self::create_assignments(&config, query, topics)?;
        consumer.assign(&assignments)?;

        Ok(Self {
            config,
            consumer_config,
            consumer,
        })
    }

    fn consume(&self, tx: UnboundedSender<OwnedMessage>) -> Result<(), Error> {
        let _ = self
            .consumer
            .stream()
            .try_chunks_timeout(
                self.consumer_config.buffer_capacity,
                Duration::from_millis(self.consumer_config.timeout_in_ms),
            )
            .for_each(|bulk_of_records| {
                let bulk_of_records = bulk_of_records.unwrap();
                for record in bulk_of_records {
                    tx.send(record.detach()).unwrap();
                }
                futures::future::ready(())
            });

        Ok(())
    }

    fn create_assignments(
        config: &YozefuConfig,
        query: SearchQuery,
        topics: &Vec<String>,
    ) -> Result<TopicPartitionList, Error> {
        let offset = query.offset().unwrap_or(FromOffset::End);
        let mut assignments = TopicPartitionList::new();
        for topic in topics {
            let consumer: StreamConsumer = config.create_kafka_consumer()?;
            let metadata = consumer.fetch_metadata(Some(topic), Duration::from_secs(10))?;
            let assignments_for_topic = match offset {
                FromOffset::Beginning => {
                    Self::assign_partitions(topic, &metadata, Offset::Beginning)
                }
                FromOffset::End => Self::assign_partitions(topic, &metadata, Offset::End),
                FromOffset::Offset(o) => {
                    Self::assign_partitions(topic, &metadata, Offset::Offset(o))
                }
                FromOffset::OffsetTail(o) => {
                    Self::assign_partitions(topic, &metadata, Offset::OffsetTail(o))
                }
                FromOffset::Timestamp(timestamp) => {
                    let mut assignments = TopicPartitionList::new();
                    for m in metadata.topics() {
                        for p in m.partitions() {
                            assignments.add_partition(m.name(), p.id());
                        }
                    }
                    assignments.set_all_offsets(Offset::Offset(timestamp))?;
                    consumer.offsets_for_times(assignments, Duration::from_secs(60))?
                }
            };

            for elem in assignments_for_topic.elements() {
                assignments
                    .add_partition_offset(elem.topic(), elem.partition(), elem.offset())
                    .expect(
                        "Failed to add partition to assignment in 'create_assignments' function",
                    );
            }
        }

        Ok(assignments)
    }

    /// Assigns topics to a consumer
    fn assign_partitions(
        topic: &str,
        metadata: &rdkafka::metadata::Metadata,
        offset: Offset,
    ) -> TopicPartitionList {
        let mut assignments = TopicPartitionList::new();
        for m in metadata.topics() {
            for p in m.partitions() {
                assignments
                    .add_partition_offset(topic, p.id(), offset)
                    .expect(
                        "Failed to add partition to assignment in 'assign_partitions' function",
                    );
            }
        }
        assignments
    }
}
