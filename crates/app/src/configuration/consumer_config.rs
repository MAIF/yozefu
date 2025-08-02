use serde::{Deserialize, Serialize};

/// Configuration for the kafka consumer
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub struct ConsumerConfig {
    pub buffer_capacity: usize,
    pub timeout_in_ms: u64,
}

impl Default for ConsumerConfig {
    fn default() -> Self {
        Self {
            buffer_capacity: 1000,
            timeout_in_ms: 10,
        }
    }
}
