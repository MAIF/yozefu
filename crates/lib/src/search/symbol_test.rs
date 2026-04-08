use crate::search::symbol::{
    Symbol, parse_key, parse_offset, parse_partition, parse_size, parse_timestamp_symbol,
    parse_topic, parse_value,
};

#[test]
fn test_parse_value() {
    assert_eq!(parse_value(r#"value"#), Ok(("", Symbol::Value(None))));
    assert_eq!(parse_value(r#"v"#), Ok(("", Symbol::Value(None))));
}

#[test]
fn test_parse_topic() {
    assert_eq!(parse_topic(r#"topic"#), Ok(("", Symbol::Topic)));
    assert_eq!(parse_topic(r#"t"#), Ok(("", Symbol::Topic)));
}

#[test]
fn test_parse_key() {
    assert_eq!(parse_key(r#"key"#), Ok(("", Symbol::Key)));
    assert_eq!(parse_key(r#"k"#), Ok(("", Symbol::Key)));
}

#[test]
fn test_parse_partition() {
    assert_eq!(parse_partition(r#"partition"#), Ok(("", Symbol::Partition)));
    assert_eq!(parse_partition(r#"p"#), Ok(("", Symbol::Partition)));
}

#[test]
fn test_parse_offset() {
    assert_eq!(parse_offset(r#"offset"#), Ok(("", Symbol::Offset)));
    assert_eq!(parse_offset(r#"o"#), Ok(("", Symbol::Offset)));
}

#[test]
fn test_parse_timestamp() {
    assert_eq!(
        parse_timestamp_symbol(r#"timestamp"#),
        Ok(("", Symbol::Timestamp))
    );
    assert_eq!(parse_timestamp_symbol(r#"ts"#), Ok(("", Symbol::Timestamp)));
}

#[test]
fn test_parse_size() {
    assert_eq!(parse_size(r#"size"#), Ok(("", Symbol::Size)));
    assert_eq!(parse_size(r#"si"#), Ok(("", Symbol::Size)));
}
