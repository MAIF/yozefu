//! A plain formatter formats a given kafka record like this
//! ```log
//! 2023-01-01T01:00:00.000+01:00    hello-world[0][2]                    - Step 3: Consume data: rpk topic consume my-topic
//! ```

use super::KafkaFormatter;
use lib::KafkaRecord;
#[derive(Clone)]
pub struct PlainFormatter {}

impl Default for PlainFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl PlainFormatter {
    pub fn new() -> Self {
        Self {}
    }
}

impl KafkaFormatter for PlainFormatter {
    fn fmt(&self, record: &KafkaRecord) -> String {
        let topic = &record.topic;
        let split = match topic.len() < 16 {
            true => topic.len() - 1,
            false => 15,
        };
        let split_pos = topic.char_indices().nth_back(split).unwrap().0;

        let prefix = format!(
            "{}[{}][{}]",
            &topic[split_pos..],
            record.partition,
            record.offset
        );
        format!(
            "{}    {:<10}    {:>15} - {}",
            record
                .timestamp_as_local_date_time()
                .map(|t| t.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
                .unwrap_or_default(),
            prefix,
            record.key,
            record.value_as_string
        )
    }
}
