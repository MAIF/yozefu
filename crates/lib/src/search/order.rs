use nom::Parser;
use nom::{IResult, branch::alt, bytes::complete::tag, combinator::value};

use super::symbol::{
    Symbol, parse_key, parse_offset, parse_partition, parse_size, parse_timestamp_symbol,
    parse_topic, parse_value,
};
use super::wsi::wsi;

/// This struct is only used when you start the TUI.
/// You can order kafka records in the terminal as you could do with SQL.
///
/// ```sql
/// order by key desc
/// sort by partition asc
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OrderBy {
    pub order: Order,
    pub keyword: OrderKeyword,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderKeyword {
    Desc,
    Asc,
}

impl Default for Order {
    fn default() -> Self {
        Self::Timestamp
    }
}

impl Default for OrderKeyword {
    fn default() -> Self {
        Self::Asc
    }
}

/// You can order kafka records by the following fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Order {
    Timestamp,
    Key,
    Value,
    Partition,
    Offset,
    Size,
    Topic,
}

impl OrderBy {
    pub fn new(order: Order, keyword: OrderKeyword) -> Self {
        Self { order, keyword }
    }

    pub fn is_descending(&self) -> bool {
        self.keyword == OrderKeyword::Desc
    }
}

pub(crate) fn parse_order(input: &str) -> IResult<&str, Order> {
    let t = wsi(alt((
        parse_size,
        parse_timestamp_symbol,
        parse_offset,
        parse_key,
        parse_value,
        parse_topic,
        parse_partition,
    )))
    .parse(input)?;

    let o = match t.1 {
        Symbol::Offset => Order::Offset,
        Symbol::Key => Order::Key,
        Symbol::Topic => Order::Topic,
        Symbol::Value(_) => Order::Value,
        Symbol::Partition => Order::Partition,
        Symbol::OffsetTail => unreachable!("nope"),
        Symbol::Size => Order::Size,
        Symbol::Timestamp => Order::Timestamp,
        Symbol::Header(_) => unreachable!("nope"),
    };
    Ok((t.0, o))
}

pub(crate) fn parse_order_keyword(input: &str) -> IResult<&str, OrderKeyword> {
    alt((
        value(OrderKeyword::Asc, wsi(tag("asc"))),
        value(OrderKeyword::Desc, wsi(tag("desc"))),
    ))
    .parse(input)
}

impl std::fmt::Display for OrderBy {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "order by {} {}", self.order, self.keyword)
    }
}

impl std::fmt::Display for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let order = match self {
            Order::Timestamp => "timestamp",
            Order::Key => "key",
            Order::Value => "value",
            Order::Partition => "partition",
            Order::Offset => "offset",
            Order::Size => "size",
            Order::Topic => "topic",
        };
        write!(f, "{order}")
    }
}

impl std::fmt::Display for OrderKeyword {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let keyword = match self {
            OrderKeyword::Desc => "desc",
            OrderKeyword::Asc => "asc",
        };
        write!(f, "{keyword}")
    }
}

#[test]
fn test_parse_order() {
    assert_eq!(parse_order(r#"partition"#), Ok(("", Order::Partition)));
    assert!(parse_order(r#"!value"#).is_err());
}

#[test]
fn test_parse_order_keyword() {
    assert_eq!(parse_order_keyword(r#"asc"#), Ok(("", OrderKeyword::Asc)));
    assert_eq!(parse_order_keyword(r#"desc"#), Ok(("", OrderKeyword::Desc)));
}

#[test]
fn test_fmt() {
    assert_eq!(
        format!("{}", OrderBy::new(Order::Key, OrderKeyword::Asc)),
        "order by key asc"
    );
    assert_eq!(
        format!("{}", OrderBy::new(Order::Value, OrderKeyword::Desc)),
        "order by value desc"
    );
}

#[test]
fn test_is_descending() {
    assert!(OrderBy::new(Order::Key, OrderKeyword::Desc).is_descending());
    assert!(!OrderBy::new(Order::Partition, OrderKeyword::Asc).is_descending());
}
