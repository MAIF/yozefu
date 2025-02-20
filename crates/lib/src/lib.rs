//! This crate contains the core structs and enums for the tool.
//! It heavily relies on the [`rdkafka` crate](https://github.com/fede1024/rust-rdkafka).

#[cfg(feature = "native")]
pub mod error;

#[cfg(feature = "native")]
pub use {
    error::Error, kafka::ExportedKafkaRecord, kafka::topic::*, search::SearchQuery,
    search::parse_search_query,
};

pub mod kafka;
pub mod search;
pub use kafka::Comparable;
pub use kafka::DataType;
pub use kafka::KafkaRecord;
pub use search::FilterResult;
pub use search::compare::StringOperator;
