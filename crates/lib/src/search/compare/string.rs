use std::fmt::Display;

#[cfg(feature = "native")]
use crate::search::wsi::wsi;
#[cfg(feature = "native")]
use nom::{
    IResult, Parser, branch::alt, bytes::complete::tag, bytes::complete::tag_no_case,
    combinator::value, sequence::pair,
};

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum StringOperator {
    Contain,
    Equal,
    NotEqual,
    StartWith,
}

impl Display for StringOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StringOperator::Contain => write!(f, "contains"),
            StringOperator::Equal => write!(f, "=="),
            StringOperator::NotEqual => write!(f, "!="),
            StringOperator::StartWith => write!(f, "starts with"),
        }
    }
}

#[cfg(feature = "native")]
pub fn parse_string_operator(input: &str) -> IResult<&str, StringOperator> {
    alt((
        value(StringOperator::Contain, wsi(alt((tag("~="), tag("=~"))))),
        value(
            StringOperator::Contain,
            wsi(alt((
                tag_no_case("contains"),
                tag_no_case("c"),
                tag_no_case("contain"),
                tag_no_case("include"),
                tag_no_case("includes"),
            ))),
        ),
        value(
            StringOperator::StartWith,
            wsi(pair(
                wsi(alt((tag_no_case("starts"), tag_no_case("start")))),
                wsi(tag_no_case("with")),
            )),
        ),
        value(StringOperator::Equal, wsi(tag("=="))),
        value(StringOperator::NotEqual, wsi(tag("!="))),
    ))
    .parse(input)
}
