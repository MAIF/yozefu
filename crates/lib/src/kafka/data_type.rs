//! As you you, a kafka record is just a bunch of bytes. The key and the value of a record can be of different types.
//! This module defines the different data types supported.
//! More details about the bytes format when using a schema: <https://docs.confluent.io/platform/current/schema-registry/fundamentals/serdes-develop/index.html#wire-format>
use std::fmt::Display;

use serde::Deserialize;
use serde::Serialize;

use crate::search::compare::StringOperator;

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq)]
#[serde(untagged)]
pub enum DataType {
    Json(serde_json::Value),
    String(String),
}

impl From<DataType> for serde_json::Value {
    fn from(val: DataType) -> Self {
        match val {
            DataType::Json(value) => value,
            DataType::String(s) => serde_json::Value::String(s),
        }
    }
}

impl Default for DataType {
    fn default() -> Self {
        Self::String("".to_string())
    }
}

pub trait Comparable {
    fn compare(
        &self,
        json_pointer: &Option<String>,
        operator: &StringOperator,
        right: &str,
    ) -> bool;
}

impl Comparable for DataType {
    fn compare(
        &self,
        json_pointer: &Option<String>,
        operator: &StringOperator,
        right: &str,
    ) -> bool {
        match &self {
            DataType::Json(value) => Self::compare_json(value, json_pointer, operator, right),
            DataType::String(value) => Self::compare_string(value, operator, right),
        }
    }
}

impl DataType {
    fn compare_json(
        value: &serde_json::Value,
        json_pointer: &Option<String>,
        operator: &StringOperator,
        right: &str,
    ) -> bool {
        let v = match json_pointer {
            Some(path) => {
                let path = path.replace(['.', '['], "/").replace(']', "");
                match value.pointer(&path) {
                    Some(d) => match d {
                        serde_json::Value::Null => "null".to_string(),
                        serde_json::Value::Bool(v) => v.to_string(),
                        serde_json::Value::Number(v) => v.to_string(),
                        serde_json::Value::String(v) => v.to_string(),
                        serde_json::Value::Array(_) => return false,
                        serde_json::Value::Object(_) => return false,
                    },
                    None => {
                        return false;
                    }
                }
            }
            None => serde_json::to_string(value).unwrap(),
        };
        match operator {
            StringOperator::Contain => v.contains(right),
            StringOperator::Equal => v == right,
            StringOperator::StartWith => v.starts_with(right),
            StringOperator::NotEqual => v != right,
        }
    }

    fn compare_string(value: &str, operator: &StringOperator, right: &str) -> bool {
        match operator {
            StringOperator::Contain => value.contains(right),
            StringOperator::Equal => value == right,
            StringOperator::StartWith => value.starts_with(right),
            StringOperator::NotEqual => value != right,
        }
    }

    pub fn raw(&self) -> String {
        match &self {
            DataType::Json(value) => match value {
                serde_json::Value::Null => "null".to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Number(number) => number.to_string(),
                serde_json::Value::String(s) => s.to_string(),
                serde_json::Value::Array(vec) => serde_json::to_string(vec).unwrap_or_default(),
                serde_json::Value::Object(map) => serde_json::to_string(map).unwrap_or_default(),
            },
            DataType::String(s) => s.clone(),
        }
    }

    pub fn to_string_pretty(&self) -> String {
        match &self {
            DataType::Json(value) => serde_json::to_string_pretty(value).unwrap_or_default(),
            DataType::String(s) => s.clone(),
        }
    }
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            DataType::Json(value) => {
                write!(f, "{}", serde_json::to_string(value).unwrap_or_default())
            }
            DataType::String(s) => write!(f, "{s}"),
        }
    }
}

#[test]
fn test_compare_string() {
    let data_type = DataType::String("hello world".to_string());
    assert!(data_type.compare(&None, &StringOperator::Contain, "world"));
    assert!(data_type.compare(&None, &StringOperator::Equal, "hello world"));
    assert!(data_type.compare(&None, &StringOperator::StartWith, "hello"));
    assert!(!data_type.compare(&None, &StringOperator::NotEqual, "hello world"));
    assert!(data_type.compare(&None, &StringOperator::Equal, "hello world"));
}

#[test]
fn test_compare_json() {
    use serde_json::json;
    let data_type = DataType::Json(json!({"hello": "world"}));
    assert!(data_type.compare(&None, &StringOperator::Contain, "world"));
    assert!(data_type.compare(&Some("/hello".into()), &StringOperator::Equal, "world"));
    assert!(data_type.compare(&Some("/hello".into()), &StringOperator::StartWith, "world"));
    assert!(!data_type.compare(&Some("/hello".into()), &StringOperator::NotEqual, "world"));
    assert!(!data_type.compare(&None, &StringOperator::Equal, "goodbye"));
}
