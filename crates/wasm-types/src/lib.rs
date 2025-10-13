//! Crate containing the types used to define search filters.
pub use lib::FilterResult;
use lib::KafkaRecord;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A filter receives a struct composed of 2 fields:
///  - The consumed kafka record,
///  - The parameters passed to that filter, it can be numbers or strings
///
///
/// If the filter is `key-ends-with('1234')`, [`FilterInput`] serialized as json will be:
///
/// ```json
/// {
///     "record": {
///         "value": "Hello world",
///         "key": "f240ff26-66b3-40d0-a99e-861300c24753",
///         "topic": "my-topic",
///         "timestamp": 1717842091489,
///         "partition": 0,
///         "offset": 23,
///         "headers": {
///             "my-header": "my-value"
///         }
///     },
///     "params": [
///         "1234"
///     ]
/// }
/// ```
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub struct FilterInput {
    pub record: KafkaRecord,
    pub params: Vec<Value>,
}
