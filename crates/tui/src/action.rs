use app::search::ValidSearchQuery;
use std::collections::HashSet;

use lib::{KafkaRecord, TopicDetail, kafka::SchemaId, search::OrderBy};

use crate::schema_detail::SchemaDetail;

use super::component::{ComponentName, Shortcut};

/// Actions that can be dispatched to the UI
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Action {
    Tick,
    Render,
    /// Notify the UI that the terminal has been resized
    Resize(u16, u16),
    /// Notify the UI that the app is about to quit
    Quit,
    /// Request the app to export the given record into the file
    Export(KafkaRecord),
    /// Dispatch the new shortcuts to the UI
    Shortcuts(Vec<Shortcut>, bool),
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
    /// Request the list of kafka records to be sorted in a specific way
    OrderBy(OrderBy),
    /// List of topics to consume
    SelectedTopics(Vec<String>),
    /// Copy the given record to the clipboard
    CopyToClipboard(String),
    /// Notify the UI that a new component has been be displayed
    NewView(ComponentName),
    /// Notify the UI the visible components and their order in the stack view
    ViewStack((ComponentName, Vec<ComponentName>)),
    /// Request to open the web browser with the URL template (AKHQ, redpanda-console, etc.) pointing to the given record
    Open(KafkaRecord),
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
