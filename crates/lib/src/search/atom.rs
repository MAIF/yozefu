//! Atoms are the smallest unit of an expression. They can be a symbol, a comparison, a filter or a parenthesized expression.
use std::fmt::Display;

use nom::{
    IResult, Parser, branch::alt, bytes::complete::tag, combinator::map, sequence::delimited,
};

use super::{
    compare::{CompareExpression, parse_compare},
    expression::{Expression, parse_or_expression},
    filter::{Filter, parse_filter},
    symbol::Symbol,
    wsi::wsi,
};

#[derive(Debug, PartialEq, Clone)]
pub enum Atom {
    Symbol(Symbol),
    Compare(CompareExpression),
    Filter(Filter),
    Parenthesis(Box<Expression>),
}

pub(crate) fn parse_atom(input: &str) -> IResult<&str, Atom> {
    alt((
        map(wsi(parse_filter), Atom::Filter),
        map(wsi(parse_compare), Atom::Compare),
        map(
            delimited(wsi(tag("(")), parse_or_expression, wsi(tag(")"))),
            |expr: Expression| Atom::Parenthesis(Box::new(expr)),
        ),
        //map(parse_symbol, Atom::Symbol),
    ))
    .parse(input)
}

impl Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Atom::Symbol(a) => write!(f, "{}", a),
            Atom::Compare(a) => write!(f, "{}", a),
            Atom::Parenthesis(a) => write!(f, "{}", a),
            Atom::Filter(a) => write!(f, "{}", a),
        }
    }
}
