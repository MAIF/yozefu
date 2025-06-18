use crate::headless::formatter::KafkaFormatter;
use crate::headless::formatter::PlainFormatter;
use lib::DataType;
use lib::KafkaRecord;
use std::collections::BTreeMap;

#[test]
fn test_plain_formatter() {
    unsafe {
        use std::env;
        // Set the timezone to Paris to have a fixed timezone for the tests
        env::set_var("TZ", "Europe/Paris");
    }

    let record = KafkaRecord {
        topic: "test-topic".to_string(),
        partition: 0,
        offset: 1,
        key: DataType::String("key".to_string()),
        value: DataType::String("value".to_string()),
        timestamp: None,
        headers: BTreeMap::default(),
        key_schema: None,
        value_schema: None,
        size: 12,
        key_as_string: "key".to_string(),
        value_as_string: "value".to_string(),
    };
    let formatter = PlainFormatter::new();
    assert_eq!(
        formatter.fmt(&record),
        "1970-01-01T01:00:00.000+01:00    test-topic[0][1]    key - value"
    );

    // long topic name
    let record = KafkaRecord {
        topic: "a.topic.with.more.than.sixteen.chars".to_string(),
        ..record
    };
    let formatted = formatter.fmt(&record);
    assert_eq!(
        formatted,
        "1970-01-01T01:00:00.000+01:00    an.sixteen.chars[0][1]    key - value"
    );
}
