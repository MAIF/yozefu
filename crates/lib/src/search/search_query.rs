use itertools::Itertools;
use nom::{
    Parser,
    branch::alt,
    combinator::{eof, map},
    multi::many_till,
};

use crate::error::SearchError;

use super::{
    clause::{
        SearchClause, parse_expression, parse_from_offset_clause, parse_limit, parse_order_by,
    },
    expression::Expression,
    offset::FromOffset,
    order::{Order, OrderBy, OrderKeyword},
    wsi::wsi,
};

/// A `SearchQuery` is a combination of an expression, a limit, an offset and an order by clause.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchQuery {
    pub expression: Expression,
    pub limit: Option<usize>,
    pub from: Option<FromOffset>,
    pub order_by: OrderBy,
    //pub group_by_key: bool,
}

impl SearchQuery {
    pub fn is_empty(&self) -> bool {
        self.limit.is_none() && self.from.is_none() && self.expression.is_empty()
    }
}

impl std::fmt::Display for SearchQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut clauses = vec![];

        let from = match &self.from {
            Some(f) => format!("from {f}"),
            None => String::new(),
        };
        let limit = match self.limit {
            Some(i) => format!("limit {i}"),
            None => String::new(),
        };
        clauses.push(from.to_string());
        clauses.push(format!("{}", self.expression));
        clauses.push(format!("{}", self.order_by));
        clauses.push(limit.to_string());
        let clauses = clauses.into_iter().filter(|e| !e.is_empty()).collect_vec();
        write!(f, "{}", clauses.join(" "))
    }
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            expression: Expression::OrExpression(vec![]),
            limit: None,
            from: None,
            order_by: OrderBy::new(Order::Timestamp, OrderKeyword::Asc),
            //group_by_key: false,
        }
    }
}

pub fn parse_search_query(input: &str) -> Result<(&str, SearchQuery), SearchError> {
    map(
        many_till(
            alt((
                parse_from_offset_clause,
                parse_limit,
                parse_expression,
                parse_order_by,
            )),
            wsi(eof),
        ),
        |clauses| {
            let mut s = SearchQuery::default();
            for c in clauses.0 {
                match c {
                    SearchClause::Limit(i) => s.limit = Some(i),
                    SearchClause::From(f) => s.from = Some(f),
                    SearchClause::Expression(u) => s.expression = u,
                    SearchClause::OrderBy(order, k) => {
                        s.order_by = OrderBy::new(order, k.unwrap_or(OrderKeyword::Asc));
                    } //SearchClause::GroupByKey => s.group_by_key = true,
                }
            }
            s
        },
    )
    .parse(input)
    .map_err(|e| {
        let remaining = match e {
            nom::Err::Incomplete(_) => input.to_string(),
            nom::Err::Error(s) => s.input.to_string(),
            nom::Err::Failure(s) => s.input.to_string(),
        };
        SearchError::Parse(remaining)
    })
}

#[test]
fn test_parse_search_query() {
    assert!(parse_search_query(r#"   from end - 10"#).is_ok());
}

#[test]
fn test_parse_search_query_with_json_path() {
    assert!(parse_search_query(r#"from end - 10 value.sequenceNum == "115568969""#).is_ok());
}
