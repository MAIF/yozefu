//! Additional struct definitions regarding topics metadata:
//!  - List of consumers, their states, the lag...
//!  - Number of partitions
//!  - Number of replicas

use std::collections::HashMap;

use rdkafka::admin::ConfigResource;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};

/// Information regarding a given topic, their consumers, the number of partitions...
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct TopicDetail {
    pub name: String,
    pub partitions: usize,
    pub replicas: usize,
    pub consumer_groups: Vec<ConsumerGroupDetail>,
    pub count: i64,
    pub config: Option<TopicConfig>,
}

/// Information regarding a given consumer
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Default, Ord)]
pub struct ConsumerGroupDetail {
    pub name: String,
    pub members: Vec<ConsumerGroupMember>,
    pub state: ConsumerGroupState,
}

/// All the different states of a kafka consumer
#[derive(
    Debug,
    Clone,
    EnumString,
    EnumIter,
    Display,
    Deserialize,
    Serialize,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Copy,
)]
#[strum(serialize_all = "PascalCase")]
#[serde(rename_all = "PascalCase")]
#[derive(Default)]
pub enum ConsumerGroupState {
    #[default]
    Unknown,
    Empty,
    Dead,
    Stable,
    PreparingRebalance,
    CompletingRebalance,
    Rebalancing,
    UnknownRebalance,
}

impl ConsumerGroupDetail {
    pub fn lag(&self) -> usize {
        self.members
            .iter()
            .map(|m| m.end_offset - m.start_offset)
            .sum()
    }

    pub fn state(&self) -> bool {
        true
    }
}

/// Information regarding a consumer group member.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Default, Ord)]
pub struct ConsumerGroupMember {
    pub member: String,
    pub start_offset: usize,
    pub end_offset: usize,
    pub assignments: Vec<MemberAssignment>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Default, Ord)]
pub struct MemberAssignment {
    pub topic: String,
    pub partitions: Vec<i32>,
}

/// A configurable resource and its current configuration values.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TopicConfig {
    pub name: String,
    pub entries: HashMap<String, String>,
}

impl From<&ConfigResource> for TopicConfig {
    fn from(resource: &ConfigResource) -> Self {
        let entries = resource
            .entries
            .iter()
            .filter_map(|entry| {
                entry
                    .value
                    .as_ref()
                    .map(|value| (entry.name.clone(), value.clone()))
            })
            .collect();

        TopicConfig {
            name: match &resource.specifier {
                rdkafka::admin::OwnedResourceSpecifier::Topic(s) => s.to_string(),
                rdkafka::admin::OwnedResourceSpecifier::Group(s) => {
                    panic!("unexpected group specifier '{}'", s)
                }
                rdkafka::admin::OwnedResourceSpecifier::Broker(s) => {
                    panic!("unexpected broker specifier '{}'", s)
                }
            },
            entries,
        }
    }
}

impl TopicConfig {}
