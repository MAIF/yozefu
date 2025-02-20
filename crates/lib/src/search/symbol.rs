//! Symbols built-in variables representing each attribute of a kafka record
//! Symbols have also aliases: 't' for 'topic, 'o' for 'offset'...
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_while},
    combinator::{map, opt, recognize, value},
    error::ErrorKind,
    sequence::preceded,
};
use strum::Display;

use super::wsi::wsi;

#[derive(Debug, Display, PartialEq, Eq, Clone)]
pub enum Symbol {
    Offset,
    Topic,
    Partition,
    OffsetTail,
    Key,
    Size,
    Timestamp,
    Value(Option<String>),
    Header(String),
}

// pub(crate) fn parse_symbol(input: &str) -> IResult<&str, Symbol> {
//     alt((
//         parse_offset,
//         parse_timestamp_symbol,
//         parse_topic,
//         parse_partition,
//         parse_key,
//         parse_size,
//         parse_value,
//         map(parse_value_symbol, |e| e.0),
//         map(parse_header_symbol, |e| e.0),
//     ))
//     .parse(input)
// }

pub(crate) fn parse_offset(input: &str) -> IResult<&str, Symbol> {
    value(Symbol::Offset, wsi(alt((tag("offset"), tag("o"))))).parse(input)
}

pub(crate) fn parse_size(input: &str) -> IResult<&str, Symbol> {
    value(Symbol::Size, wsi(alt((tag("size"), tag("si"))))).parse(input)
}

pub(crate) fn parse_partition(input: &str) -> IResult<&str, Symbol> {
    value(Symbol::Partition, wsi(alt((tag("partition"), tag("p"))))).parse(input)
}

pub(crate) fn parse_value(input: &str) -> IResult<&str, Symbol> {
    value(Symbol::Value(None), wsi(alt((tag("value"), tag("v"))))).parse(input)
}

pub(crate) fn parse_topic(input: &str) -> IResult<&str, Symbol> {
    value(Symbol::Topic, wsi(alt((tag("topic"), tag("t"))))).parse(input)
}

pub(crate) fn parse_key(input: &str) -> IResult<&str, Symbol> {
    value(Symbol::Key, wsi(alt((tag("key"), tag("k"))))).parse(input)
}

pub(crate) fn parse_timestamp_symbol(input: &str) -> IResult<&str, Symbol> {
    value(Symbol::Timestamp, wsi(alt((tag("timestamp"), tag("ts"))))).parse(input)
}

pub(crate) fn parse_value_symbol(input: &str) -> IResult<&str, (Symbol, Option<String>)> {
    map(
        preceded(wsi(alt((tag("value"), tag("v")))), opt(parse_json_path)),
        |json_path| (Symbol::Value(json_path.clone()), json_path),
    )
    .parse(input)
}

pub(crate) fn parse_header_symbol(input: &str) -> IResult<&str, (Symbol, String)> {
    map(
        preceded(alt((wsi(tag("headers")), wsi(tag("h")))), parse_json_path),
        |json_path| {
            let t = json_path.replace('.', "");
            (Symbol::Header(t.clone()), t)
        },
    )
    .parse(input)
}

/// Parse a JSON Pointer, producing a list of decoded segments.
pub(crate) fn parse_json_path(input: &str) -> IResult<&str, String> {
    let (remaining, json_path) = recognize(take_while(|ch| ch != ' ')).parse(input)?;
    match json_path.is_empty() {
        true => Err(nom::Err::Error(nom::error::Error::new(
            remaining,
            ErrorKind::Fail,
        ))),
        false => Ok((remaining, json_path.to_string())),
    }
}

pub(crate) fn parse_end_keyword(input: &str) -> IResult<&str, ()> {
    map(wsi(alt((tag_no_case("end"), tag_no_case("now")))), |_| ()).parse(input)
}
