use std::{collections::BTreeMap, fs, path::PathBuf};

use chrono::{Local, TimeZone};
use rdkafka::message::OwnedMessage;

use crate::{
    DataType, KafkaRecord,
    kafka::{SchemaId, schema::Schema},
};

#[tokio::test]
async fn test_kafka_record_deserialization() {
    let payload = b"\x00\x00\x00\x00\x01{\"key\":\"value\"}";
    let message = OwnedMessage::new(
        Some(payload.to_vec()),
        None,
        "my-awesome-topic".to_string(),
        rdkafka::Timestamp::CreateTime(0),
        0,
        313,
        None,
    );
    let record = KafkaRecord::parse(message, &mut None).await;
    assert_eq!(record.size, 20);
    assert_eq!(
        record.timestamp_as_local_date_time(),
        Some(Local.timestamp_opt(0, 0).unwrap())
    );
}

#[test]
fn test_has_schemas() {
    let record = KafkaRecord {
        topic: "topic".into(),
        timestamp: None,
        partition: 1,
        offset: 32,
        headers: BTreeMap::default(),
        key_schema: Some(Schema::new(SchemaId(12), None)),
        value_schema: Some(Schema::new(SchemaId(13), None)),
        size: 32,
        key_as_string: "".into(),
        key: DataType::String("".into()),
        value_as_string: "".into(),
        value: DataType::String("".into()),
    };
    assert!(record.has_schemas());

    let record = KafkaRecord {
        topic: "topic".into(),
        timestamp: None,
        partition: 1,
        offset: 32,
        headers: BTreeMap::default(),
        key_schema: None,
        value_schema: None,
        size: 32,
        key_as_string: "".into(),
        key: DataType::String("".into()),
        value_as_string: "".into(),
        value: DataType::String("".into()),
    };

    assert!(!record.has_schemas());
}

#[test]
fn generate_json_schema_for_kafka_record() {
    use schemars::schema_for;
    let mut schema = schema_for!(KafkaRecord);
    schema.insert("$id".into(), "https://raw.githubusercontent.com/MAIF/yozefu/refs/heads/main/docs/json-schemas/kafka-record.json".into());
    let output_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("docs")
        .join("json-schemas")
        .join("kafka-record.json");

    fs::create_dir_all(output_file.parent().unwrap()).unwrap();
    fs::write(output_file, serde_json::to_string_pretty(&schema).unwrap()).unwrap();
}
