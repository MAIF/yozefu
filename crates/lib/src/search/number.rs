use nom::Parser;
use nom::character::complete::char;
use nom::multi::many0;
use nom::{
    IResult,
    character::complete::digit1,
    combinator::{map_res, opt, recognize},
    sequence::pair,
};

/// Parses an unsigned number.
pub(crate) fn parse_unsigned_number(input: &str) -> IResult<&str, usize> {
    map_res(parse_unsigned_number_as_string, |d: &str| {
        d.replace('_', "").parse()
    })
    .parse(input)
}

/// Parses an unsigned number.
/// The number can contains '_' for readability purposes.
pub(crate) fn parse_unsigned_number_as_string(input: &str) -> IResult<&str, &str> {
    recognize(pair(digit1, many0((char('_'), digit1)))).parse(input)
}

/// Parses a signed number.
pub(crate) fn parse_number(input: &str) -> IResult<&str, i64> {
    map_res(
        pair(opt(char('-')), parse_unsigned_number_as_string),
        |(o, n)| {
            let n = n.replace('_', "").parse::<i64>();
            if o.is_some() {
                return n.map(|e| -e);
            }
            n
        },
    )
    .parse(input)
}

#[test]
fn test_parse_number() {
    assert_eq!(parse_number("10"), Ok(("", 10)));
    assert_eq!(parse_number("10_0"), Ok(("", 100)));
    assert_eq!(parse_number("-10_0"), Ok(("", -100)));
}
