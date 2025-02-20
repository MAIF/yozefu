use chrono::{DateTime, Local};
use fuzzydate::parse;
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag_no_case,
    combinator::{map_res, value},
};

use super::{string::parse_string, wsi::wsi};

/// Parses a timestamp.
/// It can be a RFC3339 date time
/// or a fuzzy date (`3 hours ago`) or the 'now' keyword.
///
/// ```text
/// "3 hours ago"
/// '2024-09-19T17:59:25.815+02:00'
/// now
/// ```
pub(crate) fn parse_timestamp(input: &str) -> IResult<&str, DateTime<Local>> {
    alt((
        map_res(parse_string, |s| {
            DateTime::parse_from_rfc3339(&s).map(|d| d.with_timezone(&Local))
        }),
        map_res(parse_string, |s| {
            parse(s).map(|d| d.and_local_timezone(Local).unwrap())
        }),
        value(Local::now(), wsi(tag_no_case("now"))),
    ))
    .parse(input)
}

#[test]
fn test_parse_timestamp() {
    assert!(parse_timestamp(r#"'3 hours ago'"#).is_ok());
    assert!(parse_timestamp(r#"now"#).is_ok());
    assert!(parse_timestamp(r#""2024-09-17T06:44:59Z""#).is_ok());
}
