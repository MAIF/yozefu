use nom::IResult;
use nom::Parser;
use nom::branch::alt;
use nom::bytes::complete::tag;

use crate::search::wsi::wsi;

/// Parse the equal operator.
/// It could be `=` or `==`.
pub fn parse_equal(input: &str) -> IResult<&str, &str> {
    wsi(alt((tag("=="), tag("=")))).parse(input)
}

#[test]
fn test_parse_equal() {
    assert!(parse_equal(r#"="#).is_ok());
}

#[test]
fn test_parse_equal_2() {
    assert!(parse_equal(r#"=="#).is_ok());
}
