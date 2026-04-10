//! This app is both a kafka consumer and a kafka admin client.
use lib::{
    Error, ExportedKafkaRecord, KafkaRecord, TopicConfig, TopicDetail, kafka::SchemaRegistryClient,
};
use rdkafka::{
    TopicPartitionList,
    consumer::{Consumer as AA, StreamConsumer},
};
use thousands::Separable;
use tracing::info;

use std::{collections::HashSet, fs, time::Duration};

use itertools::Itertools;

use crate::{
    AdminClient,
    configuration::{Configuration, ConsumerConfig, InternalConfig, YozefuConfig},
    consumer::Consumer,
    search::ValidSearchQuery,
};

/// Struct exposing different functions for consuming kafka records.
#[derive(Debug, Clone)]
pub struct App {
    pub cluster: String,
    pub config: InternalConfig,
    pub search_query: ValidSearchQuery,
}

impl App {
    pub fn new(cluster: String, config: InternalConfig, search_query: ValidSearchQuery) -> Self {
        Self {
            cluster,
            config,
            search_query,
        }
    }

    pub fn schema_registry(&self) -> Option<SchemaRegistryClient> {
        match self.config.schema_registry_config_of(&self.cluster) {
            Some(config) => Some(SchemaRegistryClient::new(config.url, &config.headers)),
            None => None,
        }
    }

    pub fn create_consumer_2(&self, topics: &Vec<String>) -> Result<Consumer, Error> {
        Consumer::new(
            self.config.specific.clone(),
            self.consumer_config(),
            self.search_query.query().clone(),
            topics,
        )
    }

    /// Create a kafka consumer
    pub fn create_consumer(&self, topics: &Vec<String>) -> Result<StreamConsumer, Error> {
        Ok(self.create_consumer_2(topics)?.stream_consumer())
    }

    pub fn consumer_config(&self) -> ConsumerConfig {
        self.config.consumer_config(&self.cluster)
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
        topic_partition_list: &TopicPartitionList,
    ) -> Result<i64, Error> {
        let count = self
            .admin_client()?
            .estimate_number_of_records_to_read(topic_partition_list)?;
        info!(
            "{} records are about to be consumed from the following topic partitions: [{}]",
            count.separate_with_underscores(),
            topic_partition_list
                .elements()
                .iter()
                .map(|e| format!("{}-{}", e.topic(), e.partition()))
                .join(", ")
        );
        Ok(count)
    }

    pub fn topic_details(&self, topics: HashSet<String>) -> Result<Vec<TopicDetail>, Error> {
        self.admin_client()?.topic_details(topics)
    }

    pub async fn topic_config_of(&self, topic: &str) -> Result<Option<TopicConfig>, Error> {
        self.admin_client()?.topic_config(topic).await
    }

    pub fn admin_client(&self) -> Result<AdminClient, Error> {
        AdminClient::new(self.config.clone())
    }

    /// Lists available kafka topics on the cluster.
    pub fn list_topics(&self) -> Result<Vec<String>, Error> {
        self.admin_client()?.list_topics()
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
