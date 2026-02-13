use std::time::Duration;

use rdkafka::{
    admin::AdminClient,
    client::DefaultClientContext,
    consumer::{BaseConsumer, Consumer},
};

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct PartitionLag {
    topic: String,
    partition: i32,
    consumer_group: String,
    member_id: Option<String>,
    committed_offset: i64,
    high_watermark: i64,
}

#[allow(dead_code)]
async fn get_consumer_group_lag(
    topic: &str,
    consumer: BaseConsumer,
    _admin_client: AdminClient<DefaultClientContext>,
) -> Result<Vec<PartitionLag>, Box<dyn std::error::Error>> {
    // Get topic metadata to find all partitions
    let metadata = consumer.fetch_metadata(Some(topic), Duration::from_secs(10))?;

    let _topic_metadata = metadata
        .topics()
        .iter()
        .find(|t| t.name() == topic)
        .ok_or(format!("Topic '{}' not found", topic))?;

    //let partitions: Vec<i32> = topic_metadata.partitions().iter().map(|p| p.id()).collect();
    let all_partition_lags = Vec::new();

    //// List all consumer groups
    //let groups = admin_client.
    //    .list_consumer_groups(Some(Duration::from_secs(10)))
    //    .await?;
    //

    //
    //// For each consumer group, check if it consumes this topic
    //for group_listing in groups {
    //    let group_id = group_listing.group_id();
    //
    //    // Describe the consumer group
    //    let group_results = [];
    //
    //    for group_result in group_results {
    //        let group = match group_result {
    //            Ok(g) => g,
    //            Err(e) => {
    //                eprintln!("Error describing group {}: {}", group_id, e);
    //                continue;
    //            }
    //        };
    //
    //        let mut consumes_topic = false;
    //        let mut partition_to_member: HashMap<i32, String> = HashMap::new();
    //
    //        for member in group.members() {
    //            if let Some(assignment) = member.assignment() {
    //                for tp in assignment.elements() {
    //                    if tp.topic() == topic {
    //                        consumes_topic = true;
    //                        partition_to_member
    //                            .insert(tp.partition(), member.member_id().to_string());
    //                    }
    //                }
    //            }
    //        }
    //
    //        if !consumes_topic {
    //            continue;
    //        }
    //
    //        for &partition in &partitions {
    //            let mut tpl = TopicPartitionList::new();
    //            tpl.add_partition(topic, partition);
    //
    //            let committed =
    //                group_consumer.committed_offsets(tpl.clone(), Duration::from_secs(10))?;
    //
    //            let committed_offset = committed
    //                .find_partition(topic, partition)
    //                .map(|tp| match tp.offset() {
    //                    Offset::Offset(o) => o,
    //                    Offset::Invalid => -1,
    //                    _ => -1,
    //                })
    //                .unwrap_or(-1);
    //
    //            // Get high watermark
    //            let (_low, high) =
    //                consumer.fetch_watermarks(topic, partition, Duration::from_secs(10))?;
    //
    //            // Calculate lag
    //            let lag = if committed_offset >= 0 {
    //                high - committed_offset
    //            } else {
    //                high // No commits yet, full topic is lag
    //            };
    //
    //            let member_id = partition_to_member.get(&partition).cloned();
    //
    //            all_partition_lags.push(PartitionLag {
    //                topic: topic.to_string(),
    //                partition,
    //                consumer_group: group_id.to_string(),
    //                member_id,
    //                committed_offset,
    //                high_watermark: high,
    //            });
    //        }
    //    }
    //}

    Ok(all_partition_lags)
}
