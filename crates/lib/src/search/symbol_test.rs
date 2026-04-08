use crate::search::symbol::{Symbol, parse_symbol};

#[test]
fn test_parse_value() {
    assert_eq!(parse_symbol(r#"value"#), Ok(("", Symbol::Value(None))));
    assert_eq!(parse_symbol(r#"v"#), Ok(("", Symbol::Value(None))));
}

#[test]
fn test_parse_topic() {
    assert_eq!(parse_symbol(r#"topic"#), Ok(("", Symbol::Topic)));
    assert_eq!(parse_symbol(r#"t"#), Ok(("", Symbol::Topic)));
}

#[test]
fn test_parse_key() {
    assert_eq!(parse_symbol(r#"key"#), Ok(("", Symbol::Key)));
    assert_eq!(parse_symbol(r#"k"#), Ok(("", Symbol::Key)));
}

#[test]
fn test_parse_partition() {
    assert_eq!(parse_symbol(r#"partition"#), Ok(("", Symbol::Partition)));
    assert_eq!(parse_symbol(r#"p"#), Ok(("", Symbol::Partition)));
}

#[test]
fn test_parse_offset() {
    assert_eq!(parse_symbol(r#"offset"#), Ok(("", Symbol::Offset)));
    assert_eq!(parse_symbol(r#"o"#), Ok(("", Symbol::Offset)));
}

#[test]
fn test_parse_timestamp() {
    assert_eq!(parse_symbol(r#"timestamp"#), Ok(("", Symbol::Timestamp)));
    assert_eq!(parse_symbol(r#"ts"#), Ok(("", Symbol::Timestamp)));
}

#[test]
fn test_parse_size() {
    assert_eq!(parse_symbol(r#"size"#), Ok(("", Symbol::Size)));
    assert_eq!(parse_symbol(r#"si"#), Ok(("", Symbol::Size)));
}
