/// Show a kafka record this way
/// ```log
/// 31234242   key:my-key   42    partition:0   my-topic  size:91
/// ```
use lib::KafkaRecord;

use super::KafkaFormatter;

#[derive(Clone)]
pub struct SimpleFormatter {}

impl Default for SimpleFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl SimpleFormatter {
    pub fn new() -> Self {
        Self {}
    }
}

impl KafkaFormatter for SimpleFormatter {
    fn fmt(&self, record: &KafkaRecord) -> String {
        format!(
            "{}   key:{:<20}   {:>17}    partition:{:<5}   {:<70}  size:{}",
            record
                .timestamp_as_local_date_time()
                .map(|t| t.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
                .unwrap_or_default(),
            record.key,
            format!("offset:{}", record.offset),
            record.partition,
            format!("topic:{}", record.topic),
            record.size
        )
    }
}
