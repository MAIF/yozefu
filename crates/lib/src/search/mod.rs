//! This module defines the parsing functions of search query.
//! The grammar of the syntax is the following:
//!
//! ```bnf
//! search-query      ::= clause+
//! clause            ::= or-expression | limit-clause | from-clause | order-clause
//! or-expression     ::= And-expression | and-expression 'or' and-expression
//! and-expression    ::= atom | atom 'and' atom
//! term              ::= atom | '!' atom
//! atom              ::= comparison  | filter | '(' expression ')'
//! number-symbol     ::= 'offset' | 'partition' | 'size'
//! string-symbol     ::= 'topic' | 'key' | 'timestamp' | 'value'
//! symbol            ::= number-symbol | string-symbol
//! comparison        ::= number-comparison | string-comparison | time-comparison
//! number-comparison ::= number-symbol number-operator number
//! string-comparison ::= string-symbol string-operator string
//! time-comparison   ::= 'between' string 'and' string
//! number-operator   ::=  '==' | '!=' | '>' | '<' | '>=' | '<='
//! string-operator   ::= 'starts with' | '==' | '!=' | '=~' | 'contains' | 'contain' | 'includes' | 'include'
//! filter            ::= .+ '('filter-parameters')'
//! filter-parameter  ::= string | number
//! filter-parameters ::= filter-parameter  (',' filter-parameter)*
//! limit-clause      ::= 'limit' number
//! order-clause      ::= 'order by' symbol order-keyword
//! order-keyword     ::= 'asc' | 'desc'
//! from-clause       ::= 'from' offset
//! offset            ::= 'beginning' | 'begin' | 'end' | 'end' '-' number | string | number
//! number            ::= [0-9_]+
//! string            ::= '"' [^"]+ '"' | "'" [^']+ "'"
//! ```
//! You can use <https://www.bottlecaps.de/rr/ui> to visualize it.

#[cfg(feature = "native")]
pub mod atom;
#[cfg(feature = "native")]
pub mod clause;
#[cfg(feature = "native")]
pub mod expression;
#[cfg(feature = "native")]
pub mod filter;
#[cfg(feature = "native")]
pub mod number;
#[cfg(feature = "native")]
pub mod offset;
#[cfg(feature = "native")]
pub mod order;
#[cfg(feature = "native")]
pub mod search_query;
#[cfg(feature = "native")]
pub mod string;
#[cfg(feature = "native")]
pub mod symbol;
#[cfg(feature = "native")]
pub mod term;
#[cfg(feature = "native")]
pub mod timestamp;
#[cfg(feature = "native")]
pub mod wsi;

pub mod compare;

#[cfg(feature = "native")]
pub use order::Order;
#[cfg(feature = "native")]
pub use order::OrderBy;
#[cfg(feature = "native")]
pub use search_query::SearchQuery;
#[cfg(feature = "native")]
pub use search_query::parse_search_query;
use serde::Deserialize;
use serde::Serialize;

#[cfg(test)]
pub mod expression_test;
#[cfg(test)]
pub mod filter_test;
#[cfg(test)]
pub mod number_test;
#[cfg(test)]
pub mod offset_test;

/// Result of a search filter evaluation.
#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub struct FilterResult {
    /// `true` if the kafka record matched the search filter condition, `false` otherwise.
    pub r#match: bool,
}

impl FilterResult {
    pub fn new(r#match: bool) -> Self {
        Self { r#match }
    }
}

impl From<bool> for FilterResult {
    fn from(r#match: bool) -> Self {
        Self { r#match }
    }
}

#[test]
fn generate_json_schema_filter_result() {
    use schemars::schema_for;
    let mut schema = schema_for!(FilterResult);
    schema.insert("$id".into(), "https://raw.githubusercontent.com/MAIF/yozefu/refs/heads/main/docs/json-schemas/filter-result.json".into());
    std::fs::write(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("docs")
            .join("json-schemas")
            .join("filter-result.json"),
        serde_json::to_string_pretty(&schema).unwrap(),
    )
    .unwrap();
}
