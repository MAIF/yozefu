use app::{search::ValidSearchQuery, Config};
use std::collections::HashSet;

use lib::{kafka::SchemaId, search::OrderBy, KafkaRecord, TopicDetail};

use crate::schema_detail::SchemaDetail;

use super::component::{ComponentName, Shortcut};

/// Actions that can be dispatched to the UI
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Tick,
    Render,
    /// Notify the UI that the terminal has been resized
    Resize(u16, u16),
    /// Notify the UI that the app is about to quit
    Quit,
    /// Request the app to export the given record into the file
    Export(KafkaRecord),
    /// Dispatch statistics about the number of processed records
    Count((usize, usize, usize)),
    /// Dispatch the new shortcuts to the UI
    Shortcuts(Vec<Shortcut>, bool),
    /// Request the app to clear the current notification
    ResetNotification(),
    /// Request the UI to show a new notification
    Notification(Notification),
    /// Request the UI to start searching for kafka records
    Search(ValidSearchQuery),
    ///  notification to the UI
    ShowRecord(KafkaRecord),
    /// Request the app to setup a new kafka consumer
    NewConsumer(),
    /// Request the app to start consuming
    Consuming,
    /// Request the app to refresh the UI
    Refresh,
    /// Request to refresh the shortcuts in the footer component
    RefreshShortcuts,
    /// Request to close the kafka consumer
    StopConsuming(),
    /// Request the app to fetch details (consumer groups, members...) of the given topics
    RequestTopicDetails(HashSet<String>),
    RequestSchemasOf(Option<SchemaId>, Option<SchemaId>),
    Schemas(Option<SchemaDetail>, Option<SchemaDetail>),
    /// Notify the UI the list of topics
    Topics(Vec<String>),
    /// Notify the UI that a new record has been polled
    NewRecord(KafkaRecord),
    /// Request the list of kafka records to be sorted in a specific way
    OrderBy(OrderBy),
    /// List of topics to consume
    SelectedTopics(Vec<String>),
    /// Dispatch the new configuration to the UI
    NewConfig(Config),
    /// Copy the given record to the clipboard
    CopyToClipboard(String),
    /// Notify the UI that a new component has been be displayed
    NewView(ComponentName),
    /// Notify the UI the visible components and their order in the stack view
    ViewStack((ComponentName, Vec<ComponentName>)),
    /// Request to open the web browser with the URL template (AKHQ, redpanda-console, etc.) pointing to the given record
    Open(KafkaRecord),
    /// Request the UI to close the specified component
    Close(ComponentName),
    /// Notify the UI some details (consumer groups, members...) of a given topic
    TopicDetails(Vec<TopicDetail>),
    /// Notify the UI that the user typed a new search query
    NewSearchPrompt(String),
    /// Notify the progress bar an estimate of the kafka records to consume in total according to the search query
    RecordsToRead(usize),
}

/// A notification is a message displayed at the bottom-right corner of the TUI.
#[derive(Debug, Clone, PartialEq)]
pub struct Notification {
    pub level: log::Level,
    pub message: String,
}

impl Notification {
    pub fn new(level: log::Level, message: String) -> Self {
        Self { level, message }
    }
}
