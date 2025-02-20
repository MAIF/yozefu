use insta::{assert_debug_snapshot, glob};
use rdkafka::message::OwnedMessage;
use serde::Deserialize;
use std::fs;
use tokio::runtime::Runtime;
use yozefu_lib::{ExportedKafkaRecord, KafkaRecord, parse_search_query};

#[test]
fn test_inputs() {
    unsafe {
        use std::env;
        // Set the timezone to Paris to have a fixed timezone for the tests
        env::set_var("TZ", "Europe/Paris");
    }
    glob!("inputs/search-queries/*.sql", |path| {
        let input = fs::read_to_string(path).unwrap();
        let input = input.trim();
        insta::with_settings!({
            description => input.replace("\n", " "),
            filters => vec![
            ("[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}\\.[0-9]{6,}\\+[0-9]{2}:[0-9]{2}", "[datetime]"),
        ]}, {
            assert_debug_snapshot!(parse_search_query(input));
        });
    });
}

#[test]
fn test_exported_record() {
    glob!("inputs/parsed-records/record*.json", |path| {
        let input = fs::read_to_string(path).unwrap();
        let record: KafkaRecord = serde_json::from_str(&input).unwrap();
        assert_debug_snapshot!(ExportedKafkaRecord::from(&record));
    });
}

#[test]
fn test_parse_records() {
    let rt = Runtime::new().unwrap();
    glob!("inputs/raw-records/record*.json", |path| {
        let input = fs::read_to_string(path).unwrap();
        let key_value: KeyValue = serde_json::from_str(&input).unwrap();
        let owned_message = OwnedMessage::new(
            key_value.value,
            key_value.key,
            "my-topic".to_string(),
            rdkafka::Timestamp::CreateTime(0),
            0,
            0,
            None,
        );
        rt.block_on(async {
            assert_debug_snapshot!(KafkaRecord::parse(owned_message, &mut None).await);
        });
    });
}

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
struct KeyValue {
    pub key: Option<Vec<u8>>,
    pub value: Option<Vec<u8>>,
}
