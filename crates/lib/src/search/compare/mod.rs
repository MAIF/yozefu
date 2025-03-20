#[cfg(feature = "native")]
pub mod expression;
#[cfg(feature = "native")]
pub mod number;
pub mod string;

#[cfg(feature = "native")]
pub use expression::CompareExpression;
#[cfg(feature = "native")]
pub use expression::parse_compare;
use nom::IResult;
use nom::Parser;
use nom::branch::alt;
use nom::bytes::complete::tag;
#[cfg(feature = "native")]
pub use number::NumberOperator;
pub use string::StringOperator;

use super::wsi::wsi;

#[cfg(test)]
pub mod mod_test;

pub fn parse_equal(input: &str) -> IResult<&str, &str> {
    alt((wsi(tag("==")), wsi(tag("=")))).parse(input)
}
