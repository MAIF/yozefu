use crate::search::number::{parse_number, parse_unsigned_number_as_string};

#[test]
fn test_parse_number() {
    assert!(parse_number::<usize>("2343").is_ok());
    assert!(parse_unsigned_number_as_string("2_343").is_ok());
    assert_eq!(parse_number("10"), Ok(("", 10)));
    assert_eq!(parse_number("10_0"), Ok(("", 100)));
}
