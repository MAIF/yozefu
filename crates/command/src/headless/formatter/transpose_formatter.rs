/// Formats a kafka record this way:
/// ```log
///    Topic: my-topic
///Partition: 0
///   Offset: 42
///Timestamp: 21673674232
///      Key: my-key
///    Value: my-value
///  Headers:
/// ```
use super::KafkaFormatter;
use itertools::Itertools;
use lib::KafkaRecord;

#[derive(Clone)]
pub struct TransposeFormatter {}

impl Default for TransposeFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl TransposeFormatter {
    pub fn new() -> Self {
        Self {}
    }
}

impl KafkaFormatter for TransposeFormatter {
    fn fmt(&self, record: &KafkaRecord) -> String {
        format!(
            "    Topic: {}
Partition: {}
   Offset: {}
Timestamp: {}
      Key: {}
    Value: {}
  Headers: {}
",
            record.topic,
            record.partition,
            record.offset,
            record
                .timestamp_as_local_date_time()
                .map(|d| d.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
                .unwrap_or_default(),
            record.key,
            record.value_as_string,
            record
                .headers
                .iter()
                .sorted_by(|a, b| a.0.cmp(b.0))
                .map(|(k, v)| format!("{k}='{v}'"))
                .join(", ")
        )
    }
}
