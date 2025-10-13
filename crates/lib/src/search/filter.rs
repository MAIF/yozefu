//! Filters allow developers to extend the search engine.
//!
//! Syntactically speaking, a filter looks like this a function call:
//! ```sql
//! from beginning offset > 50 && contains("rust")
//! ```
//!
//! In the example, the filter `contains` take 1 string parameter. Filters support string and number parameters for now.
//! Let's define this `contains` filter.
//!
//! this filter is not Rust code, nor assembly code but a wasm module. The wasm module must have the following requirements:
//!  - the name of the wasm file (`contains.wasm`) corresponds to the name of the filter.
//!  - the wasm module must implement 2 functions:
//!      - `fn matches(input: Input): bool` - this function returns `true` if the kafka record matches the condition.
//!      - `fn parse_parameters(params: Vec<Value>): bool` -  this function is optional: it returns `true` when the parameters are valid. Parameters are serialized to an JSON array.
//!
//! The library uses [Extism](https://extism.org/) to develop wasm modules.
//! You can also find the source code of  the `contains` WebAssembly module written in different supported programming languages.

use itertools::Itertools;
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, one_of},
    combinator::{map, recognize},
    multi::{many1, separated_list0},
    sequence::delimited,
};
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use std::fmt::Display;

use crate::KafkaRecord;

use super::{number::parse_number, string::parse_string, wsi::wsi};

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Filter {
    pub name: String,
    pub parameters: Vec<Parameter>,
}

impl Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}({})",
            self.name,
            self.parameters
                .iter()
                .map(std::string::ToString::to_string)
                .join(", ")
        )
    }
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub enum Parameter {
    Number(i64),
    String(String),
}

impl Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Parameter::Number(i) => write!(f, "{i}"),
            Parameter::String(i) => write!(f, "'{i}'"),
        }
    }
}

impl Parameter {
    pub fn json(&self) -> Value {
        match self {
            Parameter::Number(i) => Value::Number(Number::from(*i)),
            Parameter::String(i) => Value::String(i.to_string()),
        }
    }
}

fn parse_filter_name(input: &str) -> IResult<&str, String> {
    map(
        recognize(wsi(many1(alt((alphanumeric1, recognize(one_of("_-"))))))),
        |d: &str| d.to_string(),
    )
    .parse(input)
}

fn parse_parameter(input: &str) -> IResult<&str, Parameter> {
    wsi(alt((
        map(parse_number, Parameter::Number),
        map(parse_string, Parameter::String),
    )))
    .parse(input)
}

pub(crate) fn parse_filter(input: &str) -> IResult<&str, Filter> {
    let (remaining, (name, params)) = (
        parse_filter_name,
        delimited(
            wsi(tag("(")),
            separated_list0(wsi(tag(",")), parse_parameter),
            wsi(tag(")")),
        ),
    )
        .parse(input)?;
    Ok((
        remaining,
        Filter {
            name,
            parameters: params,
        },
    ))
}

/// Input for a search filter
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub struct FilterInput {
    /// The kafka record to evaluate against the filter.
    pub record: KafkaRecord,
    /// The user-provided parameters for the filter.
    pub params: Vec<Value>,
}

#[test]
fn generate_json_schema_for_filter_input() {
    use schemars::schema_for;
    let mut schema = schema_for!(FilterInput);
    schema.insert("$id".into(), "https://raw.githubusercontent.com/MAIF/yozefu/refs/heads/main/docs/json-schemas/filter-input.json".into());
    std::fs::write(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("docs")
            .join("json-schemas")
            .join("filter-input.json"),
        serde_json::to_string_pretty(&schema).unwrap(),
    )
    .unwrap();
}
