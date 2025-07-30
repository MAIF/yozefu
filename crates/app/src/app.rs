//! This app is both a kafka consumer and a kafka admin client.
use lib::{
    ConsumerGroupDetail, Error, ExportedKafkaRecord, KafkaRecord, TopicDetail,
    kafka::SchemaRegistryClient, search::offset::FromOffset,
};
use rdkafka::{
    Offset, TopicPartitionList,
    consumer::{BaseConsumer, Consumer, StreamConsumer},
};
use thousands::Separable;
use tracing::{info, warn};

use std::{collections::HashSet, fs, time::Duration};

use itertools::Itertools;

use crate::{
    configuration::{Configuration, InternalConfig, YozefuConfig},
    search::{Search, ValidSearchQuery},
};

/// Struct exposing different functions for consuming kafka records.
#[derive(Debug, Clone)]
pub struct App {
    pub cluster: String,
    pub config: InternalConfig,
    pub search_query: ValidSearchQuery,
    //pub output_file: PathBuf,
}

impl App {
    pub fn new(
        cluster: String,
        config: InternalConfig,
        search_query: ValidSearchQuery,
        //     output_file: PathBuf,
    ) -> Self {
        Self {
            cluster,
            config,
            search_query,
            //    output_file,
        }
    }

    pub fn schema_registry(&self) -> Option<SchemaRegistryClient> {
        match self.config.schema_registry_config_of(&self.cluster) {
            Some(config) => Some(SchemaRegistryClient::new(config.url, &config.headers)),
            None => None,
        }
    }

    /// Create a kafka consumer
    pub fn create_consumer(&self, topics: &Vec<String>) -> Result<StreamConsumer, Error> {
        let offset = self.search_query.offset().unwrap_or(FromOffset::End);
        match offset {
            FromOffset::Beginning => self.assign_partitions(topics, Offset::Beginning),
            FromOffset::End => self.assign_partitions(topics, Offset::End),
            FromOffset::Offset(o) => self.assign_partitions(topics, Offset::Offset(o)),
            FromOffset::OffsetTail(o) => self.assign_partitions(topics, Offset::OffsetTail(o)),
            FromOffset::Timestamp(timestamp) => {
                let consumer: StreamConsumer = self.config.create_kafka_consumer()?;
                let mut tp = TopicPartitionList::new();
                for t in topics {
                    let metadata = consumer.fetch_metadata(Some(t), Duration::from_secs(10))?;
                    for m in metadata.topics() {
                        for p in m.partitions() {
                            tp.add_partition(m.name(), p.id());
                        }
                    }
                }
                tp.set_all_offsets(Offset::Offset(timestamp))?;
                let tt = consumer.offsets_for_times(tp, Duration::from_secs(60))?;
                consumer.assign(&tt)?;
                Ok(consumer)
            }
        }
    }

    /// Exports a given kafka record to a file.
    /// The Name of the file is automatically generated at the runtime
    pub fn export_record(&self, record: &KafkaRecord) -> Result<(), Error> {
        let output_file = self.config.output_file();
        fs::create_dir_all(output_file.parent().unwrap())?;
        let content = fs::read_to_string(output_file).unwrap_or("[]".to_string());
        let mut exported_records: Vec<ExportedKafkaRecord> = serde_json::from_str(&content)?;

        let mut exported_record_kafka: ExportedKafkaRecord = record.into();
        exported_record_kafka.set_search_query(self.search_query.query());
        exported_records.push(exported_record_kafka);
        exported_records.sort_by(|a, b| {
            a.record
                .timestamp
                .cmp(&b.record.timestamp)
                .then(a.record.offset.cmp(&b.record.offset))
        });
        exported_records.dedup();
        for i in 1..exported_records.len() {
            let first_ts = exported_records.first().unwrap().record.timestamp;
            let previous_ts = exported_records.get(i - 1).unwrap().record.timestamp;
            let current = exported_records.get_mut(i).unwrap();
            current.compute_deltas_ms(first_ts, previous_ts);
        }

        fs::write(
            output_file,
            serde_json::to_string_pretty(&exported_records)?,
        )?;
        info!(
            "A record has been exported into file '{}'",
            output_file.display()
        );
        Ok(())
    }

    /// Calculates an estimate of the number of records that are going to be read.
    /// This function is used to render a progress bar.
    pub fn estimate_number_of_records_to_read(
        &self,
        topic_partition_list: TopicPartitionList,
    ) -> Result<i64, Error> {
        let client: StreamConsumer = self.create_assigned_consumer()?;
        let mut count = 0;
        for t in topic_partition_list.elements() {
            // this function call be very slow
            let watermarks: (i64, i64) =
                match client.fetch_watermarks(t.topic(), t.partition(), Duration::from_secs(10)) {
                    Ok(i) => i,
                    Err(e) => {
                        warn!(
                            "I was not able to fetch watermarks of topic '{}', partition {}: {}",
                            t.partition(),
                            t.topic(),
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

        info!(
            "{} records are about to be consumed on the following topic partitions: [{}]",
            count.separate_with_underscores(),
            topic_partition_list
                .elements()
                .iter()
                .map(|e| format!("{}-{}", e.topic(), e.partition()))
                .join(", ")
        );
        Ok(count)
    }

    fn create_assigned_consumer(&self) -> Result<StreamConsumer, Error> {
        self.config.create_kafka_consumer()
    }

    /// Assigns topics to a consumer
    fn assign_partitions(
        &self,
        topics: &Vec<String>,
        offset: Offset,
    ) -> Result<StreamConsumer, Error> {
        let consumer = self.create_assigned_consumer()?;
        let mut assignments = TopicPartitionList::new();
        for topic in topics {
            let metadata = consumer.fetch_metadata(Some(topic), Duration::from_secs(10))?;
            for t in metadata.topics() {
                for p in t.partitions() {
                    assignments.add_partition_offset(topic, p.id(), offset)?;
                }
            }
        }
        consumer.assign(&assignments)?;
        info!("New Consumer created, about to consume {topics:?}");
        Ok(consumer)
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

    pub fn count_records_in_topic(&self, topic: &str) -> Result<i64, Error> {
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

    /// Lists available kafka topics on the cluster.
    pub fn list_topics(&self) -> Result<Vec<String>, Error> {
        let consumer: StreamConsumer = self.create_assigned_consumer()?;
        let metadata = consumer.fetch_metadata(None, Duration::from_secs(10))?;
        let topics = metadata
            .topics()
            .iter()
            .map(|t| t.name().to_string())
            .collect_vec();
        Ok(topics)
    }

    // TODO https://github.com/fede1024/rust-rdkafka/pull/680
    //    pub fn parse_members(
    //        group: &GroupInfo,
    //        members: &[GroupMemberInfo],
    //    ) -> Result<Vec<ConsumerGroupMember>, anyhow::Error> {
    //        return Ok(vec![]);
    //        let members = members
    //            .iter()
    //            .map(|member| {
    //                let mut assigns = Vec::new();
    //                if group.protocol_type() == "consumer" {
    //                    if let Some(assignment) = member.assignment() {
    //                        let mut payload_rdr = Cursor::new(assignment);
    //                        assigns = Self::parse_member_assignment(&mut payload_rdr)
    //                            .expect("Parse member assignment failed");
    //                    }
    //                }
    //                ConsumerGroupMember {
    //                    member: member.id().to_owned(),
    //                    start_offset: 0,
    //                    end_offset: 0,
    //                    assignments: assigns,
    //                }
    //            })
    //            .collect::<Vec<_>>();
    //
    //        Ok(members)
    //    }
    //
    //    fn parse_member_assignment(
    //        payload_rdr: &mut Cursor<&[u8]>,
    //    ) -> Result<Vec<MemberAssignment>, anyhow::Error> {
    //        return Ok(vec![]);
    //        let _version = payload_rdr.read_i16::<BigEndian>()?;
    //        let assign_len = payload_rdr.read_i32::<BigEndian>()?;
    //        let mut assigns = Vec::with_capacity(assign_len as usize);
    //        for _ in 0..assign_len {
    //            let topic = read_str(payload_rdr)?.to_owned();
    //            let partition_len = payload_rdr.read_i32::<BigEndian>()?;
    //            let mut partitions = Vec::with_capacity(partition_len as usize);
    //            for _ in 0..partition_len {
    //                let partition = payload_rdr.read_i32::<BigEndian>()?;
    //                partitions.push(partition);
    //            }
    //            assigns.push(MemberAssignment { topic, partitions })
    //        }
    //        Ok(assigns)
    //    }

    /// Lists available topics on the cluster with a custom kafka client.
    pub fn list_topics_from_client(yozefu_config: &YozefuConfig) -> Result<Vec<String>, Error> {
        let consumer: StreamConsumer = yozefu_config.create_kafka_consumer()?;
        let metadata = consumer.fetch_metadata(None, Duration::from_secs(3))?;
        let topics = metadata
            .topics()
            .iter()
            .map(|t| t.name().to_string())
            .collect_vec();
        Ok(topics)
    }
}

//fn read_str<'a>(rdr: &'a mut Cursor<&[u8]>) -> Result<&'a str, Error> {
//    let len = (rdr.read_i16::<BigEndian>())? as usize;
//    let pos = rdr.position() as usize;
//    let slice = str::from_utf8(&rdr.get_ref()[pos..(pos + len)])?;
//    rdr.consume(len);
//    Ok(slice)
//}
//
