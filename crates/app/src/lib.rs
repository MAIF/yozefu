//! The core of the tool:
//! - List topics, and consume records,
//! - Fetch information about a given topic,
//! - Consume records.
mod app;
pub mod configuration;
pub mod search;

pub use app::App;
/// Name of the application
pub const APPLICATION_NAME: &str = "yozefu";

/// Name of the application
pub const BINARY_NAME: &str = "yozf";
