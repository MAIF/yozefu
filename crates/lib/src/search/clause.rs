/// Clauses are similar to clauses in the SQL language.
use nom::Parser;
use nom::bytes::complete::tag_no_case;
use nom::{
    IResult,
    branch::alt,
    combinator::{map, opt},
    sequence::{pair, preceded},
};

use super::expression::{Expression, parse_or_expression};
use super::number::parse_unsigned_number;
use super::offset::{FromOffset, parse_from_offset};
use super::order::{Order, OrderKeyword, parse_order, parse_order_keyword};
use super::wsi::wsi;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SearchClause {
    /// Clause that Limits the number of kafka records to consume
    Limit(usize),
    /// Clause containing the search expression
    Expression(Expression),
    /// Clause for telling the consumer where to start consuming from
    From(FromOffset),
    /// Clause defining how to sort the kafka records in the UI
    OrderBy(Order, Option<OrderKeyword>),
}

pub(crate) fn parse_expression(input: &str) -> IResult<&str, SearchClause> {
    map(
        (opt(wsi(tag_no_case("where"))), wsi(parse_or_expression)),
        |(_, e)| SearchClause::Expression(e),
    )
    .parse(input)
}

//pub(crate) fn parse_group_by_key(input: &str) -> IResult<&str, SearchClause> {
//    value(
//        SearchClause::GroupByKey,
//        (
//            wsi(tag_no_case("group")),
//            wsi(tag_no_case("by")),
//            wsi(tag_no_case("key")),
//        ),
//    )
//    .parse(input)
//}

pub(crate) fn parse_from_offset_clause(input: &str) -> IResult<&str, SearchClause> {
    map(parse_from_offset, SearchClause::From).parse(input)
}

pub(crate) fn parse_limit(input: &str) -> IResult<&str, SearchClause> {
    map(
        preceded(wsi(tag_no_case("limit")), wsi(parse_unsigned_number)),
        SearchClause::Limit,
    )
    .parse(input)
}

pub(crate) fn parse_order_by(input: &str) -> IResult<&str, SearchClause> {
    map(
        preceded(
            pair(
                wsi(alt((tag_no_case("order"), tag_no_case("sort")))),
                wsi(tag_no_case("by")),
            ),
            pair(parse_order, opt(parse_order_keyword)),
        ),
        |(o, oo)| SearchClause::OrderBy(o, oo),
    )
    .parse(input)
}

#[test]
fn test_parse_offset_clause() {
    assert_eq!(
        parse_from_offset_clause(r#"from end - 10"#),
        Ok(("", SearchClause::From(FromOffset::OffsetTail(10))))
    );
}
