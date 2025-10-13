use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::SearchQuery;

use super::KafkaRecord;

/// An exported Kafka record is a wrapper around the `KafkaRecord` struct
/// with additional fields for analytics purposes.
#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub struct ExportedKafkaRecord {
    #[serde(flatten)]
    pub record: KafkaRecord,
    date_time: Option<DateTime<Local>>,
    /// Milliseconds elapsed since the first exported kafka record.
    pub absolute_delta_in_ms: i64,
    /// Milliseconds elapsed since the previous exported kafka record.
    pub relative_delta_in_ms: i64,
    pub search_query: String,
}

impl ExportedKafkaRecord {
    /// Calculate 2 time deltas:
    ///  -  between the current record and the first received.
    ///  -  between the current record and the previous one.
    ///
    /// The unit is the millisecond.
    pub fn compute_deltas_ms(&mut self, first_ts: Option<i64>, previous_ts: Option<i64>) {
        self.relative_delta_in_ms = self.record.timestamp.unwrap_or(0) - previous_ts.unwrap_or(0);
        self.absolute_delta_in_ms = self.record.timestamp.unwrap_or(0) - first_ts.unwrap_or(0);
    }

    pub fn set_search_query(&mut self, search_query: &SearchQuery) {
        self.search_query = search_query.to_string();
    }
}

impl From<&KafkaRecord> for ExportedKafkaRecord {
    fn from(record: &KafkaRecord) -> Self {
        let date_time = record.timestamp_as_local_date_time();
        Self {
            record: record.clone(),
            date_time,
            absolute_delta_in_ms: 0,
            relative_delta_in_ms: 0,
            search_query: String::new(),
        }
    }
}

#[test]
fn generate_json_schema_for_exported_kafka_record() {
    use schemars::schema_for;
    let mut schema = schema_for!(ExportedKafkaRecord);
    schema.insert("$id".into(), "https://raw.githubusercontent.com/MAIF/yozefu/refs/heads/main/docs/json-schemas/exported-kafka-record.json".into());
    std::fs::write(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("docs")
            .join("json-schemas")
            .join("exported-kafka-record.json"),
        serde_json::to_string_pretty(&schema).unwrap(),
    )
    .unwrap();
}
