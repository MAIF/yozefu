use std::fmt::Display;

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::{complete::tag, tag_no_case},
    combinator::{map, value},
    sequence::preceded,
};

use super::{
    compare::parse_equal,
    number::parse_number,
    symbol::{parse_end_keyword, parse_offset},
    timestamp::parse_timestamp,
    wsi::wsi,
};

/// A kafka offset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FromOffset {
    /// Start consuming from the beginning of the partition.
    Beginning,
    /// Start consuming from the end of the partition.
    End,
    /// A specific offset to consume from.
    Offset(i64),
    /// An offset relative to the end of the partition.
    OffsetTail(i64),
    /// Start consuming from a specific timestamp end of the partition.
    Timestamp(i64),
}

impl Display for FromOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FromOffset::Beginning => write!(f, "beginning"),
            FromOffset::End => write!(f, "end"),
            FromOffset::Offset(o) => write!(f, "{}", o),
            FromOffset::OffsetTail(o) => write!(f, "end - {}", o),
            FromOffset::Timestamp(_) => write!(f, ""),
        }
    }
}

/// parses the clause defining from where the consumer should starting reading records.
/// ```text
/// from begin
/// from end
/// from "3 hours ago"
/// from 34895
/// from -10
/// ```
pub(crate) fn parse_from_offset(input: &str) -> IResult<&str, FromOffset> {
    preceded(
        wsi(tag_no_case("from")),
        alt((
            map(wsi(parse_timestamp), |t| {
                FromOffset::Timestamp(t.to_utc().timestamp_millis())
            }),
            value(
                FromOffset::Beginning,
                wsi(alt((tag("beginning"), tag_no_case("begin")))),
            ),
            map(
                (parse_offset, parse_equal, wsi(parse_number)),
                |(_, _, d)| FromOffset::Offset(d),
            ),
            map(wsi(parse_number), FromOffset::Offset),
            map(
                (parse_end_keyword, wsi(tag("-")), wsi(parse_number)),
                |(_, _, r)| FromOffset::OffsetTail(r),
            ),
            value(FromOffset::End, parse_end_keyword),
        )),
    )
    .parse(input)
}
