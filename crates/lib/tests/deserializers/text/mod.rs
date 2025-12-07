use insta::{assert_debug_snapshot, glob};
use std::fs;
use tokio::runtime::Runtime;
use yozefu_lib::{ExportedKafkaRecord, KafkaRecord};

use crate::{KeyValue, fix_timezone};

#[test]
fn test_exported_record() {
    fix_timezone();
    glob!("inputs/exported/record*.json", |path| {
        let input = fs::read_to_string(path).unwrap();
        let record: KafkaRecord = serde_json::from_str(&input).unwrap();
        assert_debug_snapshot!(ExportedKafkaRecord::from(&record));
    });
}

#[test]
fn test_parse_records() {
    let rt: Runtime = Runtime::new().unwrap();
    glob!("inputs/record*.json", |path| {
        let input = fs::read_to_string(path).unwrap();
        let key_value: KeyValue = serde_json::from_str(&input).unwrap();
        let owned_message = key_value.into_owned_message();
        rt.block_on(async {
            assert_debug_snapshot!(KafkaRecord::parse(owned_message, &mut None).await);
        });
    });
}
