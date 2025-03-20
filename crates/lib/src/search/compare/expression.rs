/// This module defines the parsers to comparison expressions.
/// ```bash
/// offset != 234
/// key == "my-key"
/// timestamp between "2 hours ago" and "1 hour ago"
/// ```
use std::fmt::Display;

#[cfg(feature = "native")]
use chrono::{DateTime, Local};
use nom::Parser;
use nom::bytes::complete::tag_no_case;
use nom::{
    IResult,
    branch::alt,
    bytes::complete::tag,
    combinator::{map, value},
};

use super::number::NumberOperator;
use super::string::StringOperator;

#[cfg(feature = "native")]
#[derive(Debug, PartialEq, Clone, Eq)]
pub enum CompareExpression {
    Partition(NumberOperator, i32),
    OffsetTail(i64),
    Offset(NumberOperator, i64),
    Topic(StringOperator, String),
    Key(StringOperator, String),
    Value(Option<String>, StringOperator, String),
    Header(String, StringOperator, String),
    Size(NumberOperator, i64),
    Timestamp(NumberOperator, DateTime<Local>),
    TimestampBetween(DateTime<Local>, DateTime<Local>),
}

#[cfg(feature = "native")]
impl Display for CompareExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompareExpression::Partition(op, r) => write!(f, "partition {} {}", op, r),
            CompareExpression::OffsetTail(r) => write!(f, "offsetTail - {}", r),
            CompareExpression::Offset(op, r) => write!(f, "offset {} {}", op, r),
            CompareExpression::Topic(op, r) => write!(f, "topic {} {}", op, r),
            CompareExpression::Key(op, r) => write!(f, "key {} {}", op, r),
            CompareExpression::Value(left, op, r) => write!(
                f,
                "value{} {} {}",
                left.clone().unwrap_or("".to_string()),
                op,
                r
            ),
            CompareExpression::Header(left, op, r) => {
                write!(f, "headers.{} {} {}", left.clone(), op, r)
            }
            CompareExpression::Size(op, r) => write!(f, "size {} {}", op, r),
            CompareExpression::Timestamp(op, r) => write!(
                f,
                r#"timestamp {} "{}""#,
                op,
                r.to_rfc3339_opts(chrono::SecondsFormat::Millis, false)
            ),
            CompareExpression::TimestampBetween(l, r) => write!(
                f,
                r#"timestamp between "{}" and "{}"#,
                l.to_rfc3339_opts(chrono::SecondsFormat::Millis, false),
                r.to_rfc3339_opts(chrono::SecondsFormat::Millis, false)
            ),
        }
    }
}

#[cfg(feature = "native")]
pub fn parse_compare(input: &str) -> IResult<&str, CompareExpression> {
    use crate::search::{
        compare::{parse_equal, string::parse_string_operator},
        number::parse_number,
        string::parse_string,
        symbol::{
            Symbol, parse_header_symbol, parse_key, parse_offset, parse_partition, parse_size,
            parse_timestamp_symbol, parse_topic, parse_value_symbol,
        },
        timestamp::parse_timestamp,
        wsi::wsi,
    };

    use super::number::parse_number_operator;

    alt((
        map(
            (parse_offset, wsi(parse_number_operator), wsi(parse_number)),
            |(_, op, r)| CompareExpression::Offset(op, r),
        ),
        map(
            (parse_size, wsi(parse_number_operator), wsi(parse_number)),
            |(_, op, r)| CompareExpression::Size(op, r),
        ),
        map(
            (
                value(Symbol::OffsetTail, wsi(tag("offsetTail"))),
                parse_equal,
                wsi(parse_number),
            ),
            |(_, _, r)| CompareExpression::OffsetTail(r),
        ),
        map(
            (
                parse_partition,
                wsi(parse_number_operator),
                wsi(parse_number),
            ),
            |(_, op, r)| CompareExpression::Partition(op, r as i32),
        ),
        map(
            (parse_topic, wsi(parse_string_operator), wsi(parse_string)),
            |(_, op, r)| CompareExpression::Topic(op, r),
        ),
        map(
            (parse_key, wsi(parse_string_operator), wsi(parse_string)),
            |(_, op, r)| CompareExpression::Key(op, r),
        ),
        map(
            (
                parse_value_symbol,
                wsi(parse_string_operator),
                wsi(parse_string),
            ),
            |(left, op, r)| CompareExpression::Value(left.1, op, r),
        ),
        map(
            (
                parse_header_symbol,
                wsi(parse_string_operator),
                wsi(parse_string),
            ),
            |(left, op, r)| CompareExpression::Header(left.1, op, r),
        ),
        map(
            (
                parse_timestamp_symbol,
                wsi(parse_number_operator),
                wsi(parse_timestamp),
            ),
            |(_, op, r)| CompareExpression::Timestamp(op, r),
        ),
        map(
            (
                parse_timestamp_symbol,
                wsi(tag_no_case("between")),
                wsi(parse_timestamp),
                wsi(tag_no_case("and")),
                wsi(parse_timestamp),
            ),
            |(_, _, from, _, to)| CompareExpression::TimestampBetween(from, to),
        ),
    ))
    .parse(input)
}
