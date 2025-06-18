//! Formatters for displaying kafka records to stdout.

mod json_formatter;
mod plain_formatter;
mod simple_formatter;
mod transpose_formatter;

pub use json_formatter::JsonFormatter;
pub use plain_formatter::PlainFormatter;
pub use simple_formatter::SimpleFormatter;
pub use transpose_formatter::TransposeFormatter;

use lib::KafkaRecord;

/// A kafka formatter displays a kafka record to stdout.
pub trait KafkaFormatter: Sync + Send {
    fn fmt(&self, record: &KafkaRecord) -> String;
}

#[cfg(test)]
pub mod plain_formatter_test;
