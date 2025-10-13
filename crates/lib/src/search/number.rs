use std::str::FromStr;

use nom::Parser;
use nom::character::complete::char;
use nom::multi::many0;
use nom::{
    IResult,
    character::complete::digit1,
    combinator::{map_res, recognize},
    sequence::pair,
};

/// Parses an unsigned number.
/// The number can contains '_' for readability purposes.
pub(crate) fn parse_unsigned_number_as_string(input: &str) -> IResult<&str, &str> {
    recognize(pair(digit1, many0((char('_'), digit1)))).parse(input)
}

/// Parses an unsigned number.
pub(crate) fn parse_number<T>(input: &str) -> IResult<&str, T>
where
    T: FromStr,
{
    map_res(parse_unsigned_number_as_string, |d: &str| {
        d.replace('_', "").parse()
    })
    .parse(input)
}
