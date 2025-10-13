//! Formats a kafka record as a json object.
//! ```json
//! {
//!    "value": "{ \"name\": \"Tarte Tatin\", \"ingredients\": [\"apples\", \"puff pastry\", \"butter\", \"sugar\"], \"instructions\": \"Caramelize apples in butter and sugar, top with puff pastry, and bake.\"},",
//!    "key": "1",
//!    "topic": "patisserie-delights-dlq",
//!    "timestamp": 1727734680195,
//!    "partition": 0,
//!    "offset": 529896,
//!    "headers": {
//!      "kafka_dlt-exception-fqcn": "panic: runtime error: invalid memory address or nil pointer dereference",
//!      "kafka_dlt-exception-message": "The cooking process has failed",
//!      "kafka_dlt-exception-stacktrace": "[signal SIGSEGV: segmentation violation code=0xffffffff addr=0x0 pc=0x20314]",
//!      "kafka_dlt-original-offset": "197939",
//!      "kafka_dlt-original-partition": "0",
//!      "kafka_dlt-original-topic": "patisserie-delights-dlq",
//!      "kafka_timestampType": "2024-09-30T22:18:00.193234027Z"
//!    }
//! }
//! ```
use lib::KafkaRecord;

use super::KafkaFormatter;

#[derive(Clone)]
pub struct JsonFormatter {}

impl Default for JsonFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonFormatter {
    pub fn new() -> Self {
        Self {}
    }
}

impl KafkaFormatter for JsonFormatter {
    fn fmt(&self, record: &KafkaRecord) -> String {
        serde_json::to_string_pretty(&record).unwrap_or_default()
    }
}
