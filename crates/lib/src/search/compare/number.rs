use std::fmt::Display;

use crate::search::wsi::wsi;
use nom::Parser;
use nom::{IResult, branch::alt, bytes::complete::tag, combinator::value};

use super::parse_equal;

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum NumberOperator {
    GreaterThan,
    GreaterOrEqual,
    LowerThan,
    LowerOrEqual,
    Equal,
    NotEqual,
}

impl Display for NumberOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::GreaterThan => write!(f, ">"),
            Self::GreaterOrEqual => write!(f, ">="),
            Self::LowerThan => write!(f, "<"),
            Self::LowerOrEqual => write!(f, "<="),
            Self::Equal => write!(f, "=="),
            Self::NotEqual => write!(f, "!="),
        }
    }
}

pub fn parse_number_operator(input: &str) -> IResult<&str, NumberOperator> {
    alt((
        value(NumberOperator::GreaterOrEqual, wsi(tag(">="))),
        value(NumberOperator::LowerOrEqual, wsi(tag("<="))),
        value(NumberOperator::GreaterThan, wsi(tag(">"))),
        value(NumberOperator::LowerThan, wsi(tag("<"))),
        value(NumberOperator::Equal, parse_equal),
        value(NumberOperator::NotEqual, wsi(tag("!="))),
    ))
    .parse(input)
}
