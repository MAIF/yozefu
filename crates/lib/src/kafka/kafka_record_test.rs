use std::collections::BTreeMap;

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
    assert_eq!(record.size, 20)
}

#[test]
fn test_has_schema() {
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
    assert!(record.has_schemas())
}
