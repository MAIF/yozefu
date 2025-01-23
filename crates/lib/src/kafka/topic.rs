//! Additional struct definitions regarding topics metadata:
//!  - List of consumers, their states, the lag...
//!  - Number of partitions
//!  - Number of replicas

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};

/// Information regarding a given topic, their consumers, the number of partitions...
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Default, Ord)]
pub struct TopicDetail {
    pub name: String,
    pub partitions: usize,
    pub replicas: usize,
    pub consumer_groups: Vec<ConsumerGroupDetail>,
    pub count: i64,
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
