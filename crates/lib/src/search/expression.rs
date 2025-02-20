/// Expressions represent booleans expression such as
/// ```js
/// offset == 23 and key == "my-key"
/// offset == 23 && key == "my-key"
///
/// key starts with "1234-" or offset < 100
/// key starts with "1234-" || offset < 100
/// ```
use std::fmt::Display;

use nom::Parser;
use nom::bytes::complete::tag_no_case;
use nom::{
    IResult, branch::alt, bytes::complete::tag, combinator::map, multi::many0, sequence::preceded,
};

use super::term::{Term, parse_term};
use super::wsi::wsi;

// https://stackoverflow.com/questions/9509048/antlr-parser-for-and-or-logic-how-to-get-expressions-between-logic-operators
pub type Expression = OrExpression;
#[derive(Debug, PartialEq, Clone)]
pub enum AndExpression {
    AndTerm(Term),
    AndExpression(Vec<Term>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum OrExpression {
    OrTerm(AndExpression),
    OrExpression(Vec<AndExpression>),
}

impl Display for AndExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AndExpression::AndTerm(t) => write!(f, "{}", t),
            AndExpression::AndExpression(e) => write!(
                f,
                "{}",
                e.iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
                    .join(" && ")
            ),
        }
    }
}

impl Display for OrExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrExpression::OrTerm(t) => write!(f, "{}", t),
            OrExpression::OrExpression(t) => write!(
                f,
                "{}",
                t.iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
                    .join(" || ")
            ),
        }
    }
}

impl OrExpression {
    pub(crate) fn is_empty(&self) -> bool {
        match self {
            OrExpression::OrTerm(_) => false,
            OrExpression::OrExpression(v) => v.is_empty(),
        }
    }
}

/// Parses an or expression, operator is `||` or `or`
pub(crate) fn parse_or_expression(input: &str) -> IResult<&str, OrExpression> {
    if input.trim().is_empty() {
        return Ok(("", Expression::OrExpression(vec![])));
    }
    map(
        (
            wsi(parse_and_expression),
            many0(preceded(
                wsi(alt((tag("||"), tag("or")))),
                wsi(parse_and_expression),
            )),
        ),
        |(l, ee)| {
            if ee.is_empty() {
                OrExpression::OrTerm(l)
            } else {
                let mut ll = vec![l];
                ll.extend(ee);
                OrExpression::OrExpression(ll)
            }
        },
    )
    .parse(input)
}

/// Parses an or expression, operator is `&&` or `and`
pub(crate) fn parse_and_expression(input: &str) -> IResult<&str, AndExpression> {
    map(
        (
            wsi(parse_term),
            many0(preceded(
                wsi(alt((tag("&&"), tag_no_case("and")))),
                wsi(parse_term),
            )),
        ),
        |(l, ee)| {
            if ee.is_empty() {
                AndExpression::AndTerm(l)
            } else {
                let mut ll = vec![l];
                ll.extend(ee);
                AndExpression::AndExpression(ll)
            }
        },
    )
    .parse(input)
}
