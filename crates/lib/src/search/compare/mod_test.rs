use crate::search::{
    compare::{parse_compare, parse_equal},
    parse_search_query,
};

#[test]
fn test_parse_compare() {
    assert!(parse_compare(r#"timestamp between "2024-05-28T17:55:08.145+02:00" and now"#).is_ok());
    assert!(
        parse_search_query(
            r#"timestamp between "2024-05-28T17:55:08.145+02:00" and now from begin"#
        )
        .is_ok()
    );
}

#[test]
fn test_parse_search_query() {
    assert!(
        parse_search_query(
            r#"timestamp between "2024-05-28T17:55:08.145+02:00" and now from begin"#
        )
        .is_ok()
    );
}

#[test]
fn test_parse_equal() {
    assert!(parse_equal(r#"="#).is_ok());
}

#[test]
fn test_parse_equal_2() {
    assert!(parse_equal(r#"=="#).is_ok());
}
