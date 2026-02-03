//! Module gathering the code to run the terminal user interface.

use app::App;
use app::search::{Search, SearchContext};
use chrono::DateTime;
use crossterm::event::KeyEvent;
use futures::{StreamExt, future};
use futures_batch::TryChunksTimeoutStreamExt;
use itertools::Itertools;
use lib::KafkaRecord;
use ratatui::prelude::Rect;
use rdkafka::Message;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::OwnedMessage;
use std::collections::HashSet;
use std::time::Duration;
use tokio::select;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, trace_span, warn};

use crate::action::{Action, Level, Notification};
use crate::component::{Component, RootComponent};
use crate::error::TuiError;
use crate::records_buffer::RecordsAndStats;
use crate::schema_detail::SchemaDetail;
use crate::tui;

use super::{RecordsSender, State};

pub struct Ui {
    app: App,
    should_quit: bool,
    root: RootComponent,
    worker: CancellationToken,
    topics: Vec<String>,
    last_tick_key_events: Vec<KeyEvent>,
    records_sender: RecordsSender,
}

impl Ui {
    pub fn new(app: App, query: &str, selected_topics: Vec<String>, state: State) -> Self {
        let (records_sender, records_receiver) = tokio::sync::mpsc::unbounded_channel();
        info!("hello from tui ui");
        Self {
            should_quit: false,
            worker: CancellationToken::new(),
            app,
            topics: vec![],
            root: RootComponent::new(query, selected_topics, records_receiver, state),
            records_sender,
            last_tick_key_events: Vec::new(),
        }
    }

    pub(crate) fn create_consumer(
        app: &App,
        topics: Vec<String>,
        tx: UnboundedSender<Action>,
    ) -> Result<StreamConsumer, TuiError> {
        match app.create_consumer(&topics) {
            Ok(c) => Ok(c),
            Err(e) => {
                tx.send(Action::Notification(Notification::new(
                    Level::Error,
                    e.to_string(),
                )))?;
                error!("Something went wrong when trying to consume topics: {e}");
                Err(e.into())
            }
        }
    }

    pub(crate) fn consume_topics(&mut self, tx: UnboundedSender<Action>) -> Result<(), TuiError> {
        self.worker.cancel();
        // records state is now channel-driven, no global lock-reset needed
        // If records need reset, propagate an action or recreate records_receiver

        if self.topics.is_empty() {
            tx.send(Action::StopConsuming())?;
            return Ok(());
        }

        let message = match self.app.search_query.is_empty() {
            true => "Waiting for new events".to_string(),
            false => "Searching".to_string(),
        };

        tx.send(Action::Notification(Notification::new(
            Level::Info,
            message,
        )))?;
        self.worker = CancellationToken::new();

        let query = self.app.search_query.query().clone();
        let order_by = query.order_by.clone();
        tx.send(Action::OrderBy(order_by.clone()))?;
        tx.send(Action::NewConsumer())?;
        tx.send(Action::Consuming)?;

        let _token = self.worker.clone();
        let token = self.worker.clone();
        let search_query = self.app.search_query.query().clone();
        let app = self.app.clone();
        let txx = tx.clone();
        let topics = self.topics.clone();

        let (tx_dd, mut rx_dd) = mpsc::unbounded_channel::<OwnedMessage>();
        let mut schema_registry = app.schema_registry().clone();
        let token_cloned = token.clone();

        let filters_directory = self.app.config.workspace().filters_dir();
        let records_sender = self.records_sender.clone();
        tokio::task::Builder::new()
            .name("search-engine")
        .spawn(async move {
            let (mut read, mut matched) = (0, 0);
            loop {
                select! {
                    _ = token_cloned.cancelled() => {
                        return;
                     },
                    Some(message) = rx_dd.recv() => {
                        let record = KafkaRecord::parse(message, &mut schema_registry).await;
                        let context = SearchContext::new(&record, &filters_directory);
                        let span = trace_span!("matching", offset = %record.offset, partition = %record.partition, topic = %record.topic);
                        let search_span = span.enter();
                        let matches = search_query.matches(&context);
                        drop(search_span);
                        read += 1;
                        // Pushing to a locked buffer replaced by sending over channel.
                        if matches {
                            matched += 1;
                            records_sender.send(RecordsAndStats {
                                records: vec![record],
                                read
                            }).unwrap();
                        }

                        if !matches && read % 200 == 0 {
                            // Send stats update even if no match found to update the UI
                            records_sender.send(RecordsAndStats {
                                records: vec![],
                                read
                            }).unwrap();
                        }


                        if let Some(limit) = query.limit {
                            if Some(matched) >= Some(limit) {
                                token_cloned.cancel();
                            }
                        }
                    }
                }
            }
        }).unwrap();

        let consumer_config = self.app.consumer_config();
        tokio::task::Builder::new()
            .name("kafka-consumer")
            .spawn(async move {
                let _ = tx.send(Action::Consuming);
                let consumer = match Self::create_consumer(&app, topics.clone(), txx.clone()) {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = tx.send(Action::StopConsuming());
                        warn!("I was not able to create a consumer: {e}");
                        return Err("I was not able to create a consumer after 5 attempts...");
                    }
                };
                let _ = tx.send(Action::Consuming);
                let assignments = consumer.assignment().unwrap();
                let txx = tx.clone();
                tokio::task::Builder::new()
                    .name("records-to-read")
                    .spawn(async move {
                        let count = app
                            .estimate_number_of_records_to_read(&assignments)
                            .unwrap_or(0);
                        let _ = txx.send(Action::RecordsToRead(count as usize));
                    })
                    .unwrap();
                let mut current_time = Instant::now();

                let _ = consumer
                    .stream()
                    .take_until(token.cancelled())
                    .try_chunks_timeout(
                        consumer_config.buffer_capacity,
                        Duration::from_millis(consumer_config.timeout_in_ms),
                    )
                    .for_each(|bulk_of_records| {
                        let bulk_of_records = bulk_of_records.unwrap();
                        info!("Received a bulk of {} records", bulk_of_records.len());
                        let timestamp = bulk_of_records
                            .last()
                            .and_then(|r| r.timestamp().to_millis())
                            .unwrap_or(0);
                        for record in bulk_of_records {
                            if tx_dd.send(record.detach()).is_err() {
                                token.cancel();
                                break;
                            }
                        }
                        if current_time.elapsed() > Duration::from_secs(13) {
                            current_time = Instant::now();

                            tx.send(Action::Notification(Notification::new(
                                Level::Info,
                                format!(
                                    "Checkpoint: {}",
                                    DateTime::from_timestamp_millis(timestamp).unwrap()
                                ),
                            )))
                            .unwrap();
                        }
                        future::ready(())
                    })
                    .await;
                consumer.unassign().unwrap();
                info!("Consumer is terminated");
                token.cancel();
                let _ = tx.send(Action::StopConsuming());
                Ok(())
            })
            .unwrap();
        Ok(())
    }

    pub(crate) fn topics_details(
        &mut self,
        topics: HashSet<String>,
        action_tx: UnboundedSender<Action>,
    ) {
        let app = self.app.clone();
        tokio::task::Builder::new()
            .name("topics-details")
            .spawn(async move {
                match app.topic_details(topics) {
                    Ok(mut details) => {
                        for detail in &mut details.iter_mut() {
                            detail.config = app.topic_config_of(&detail.name).await.ok().flatten();
                        }
                        action_tx.send(Action::TopicDetails(details)).unwrap();
                    }
                    Err(e) => action_tx
                        .send(Action::Notification(Notification::new(
                            Level::Error,
                            e.to_string(),
                        )))
                        .unwrap(),
                }
            })
            .unwrap();
    }

    pub(crate) fn export_record(
        &mut self,
        record: &KafkaRecord,
        action_tx: &UnboundedSender<Action>,
    ) -> Result<(), TuiError> {
        self.app.export_record(record)?;
        action_tx.send(Action::Notification(Notification::new(
            Level::Info,
            "Record exported to the file".to_string(),
        )))?;
        Ok(())
    }

    pub(crate) fn load_topics(&mut self, action_tx: UnboundedSender<Action>) {
        let app = self.app.clone();
        tokio::task::Builder::new()
            .name("topics-loader")
            .spawn(async move {
                info!("Listing topics from the cluster");
                match app.list_topics() {
                    Ok(topics) => {
                        action_tx.send(Action::Topics(topics)).unwrap();
                    }
                    Err(e) => {
                        if action_tx
                            .send(Action::Notification(Notification::new(
                                Level::Error,
                                e.to_string(),
                            )))
                            .is_err()
                        {
                            error!("Cannot notify the TUI: {e:?}");
                        }
                        error!("Something went wrong when trying to list topics: {e}");
                    }
                }
            })
            .unwrap();
    }

    pub async fn run(&mut self, topics: Vec<String>, state: State) -> Result<(), TuiError> {
        let (action_tx, mut action_rx) = mpsc::unbounded_channel();
        // No need to track a global receiver here; records are passed directly to the components that consume them.
        self.load_topics(action_tx.clone());
        let mut tui: tui::Tui = tui::Tui::new()?;
        tui.enter()?;
        self.root.register_action_handler(action_tx.clone());
        self.root.init()?;
        if !topics.is_empty() {
            action_tx.send(Action::SelectedTopics(topics))?;
        }

        let mut schema_registry = self.app.schema_registry();
        loop {
            if let Some(e) = tui.next().await {
                match e {
                    tui::Event::Quit => action_tx.send(Action::Quit)?,
                    tui::Event::Tick => action_tx.send(Action::Tick)?,
                    tui::Event::Render => action_tx.send(Action::Render)?,
                    tui::Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
                    _ => {}
                }

                if let Some(action) = self.root.handle_events(Some(e.clone()))? {
                    action_tx.send(action)?;
                }
            }
            while let Ok(action) = action_rx.try_recv() {
                // Possible place to poll or forward any pending records from channel, if needed
                match action {
                    Action::NewSearchPrompt(ref prompt) => {
                        self.app.config.push_history(prompt);
                        self.app.config.save_config()?;
                    }
                    Action::RequestTopicDetails(ref topics) => {
                        self.topics_details(topics.clone(), action_tx.clone());
                    }
                    Action::Tick => {
                        self.last_tick_key_events.drain(..);
                    }
                    Action::Refresh => {
                        self.load_topics(action_tx.clone());
                        action_tx.send(Action::Notification(Notification::new(
                            Level::Info,
                            "Refreshing topics".to_string(),
                        )))?;
                    }
                    Action::Quit => {
                        self.worker.cancel();
                        self.should_quit = true;
                    }
                    Action::Open(ref record) => {
                        let url = self
                            .app
                            .config
                            .url_template_of(&state.cluster)
                            .replace("{topic}", &record.topic)
                            .replace("{partition}", &record.partition.to_string())
                            .replace("{offset}", &record.offset.to_string());

                        if let Err(e) = open::that(&url) {
                            action_tx.send(Action::Notification(Notification::new(
                                Level::Info,
                                "this action is not available right now".to_string(),
                            )))?;
                            warn!("Cannot open the URL '{url}': {e}");
                        }
                    }
                    Action::Resize(w, h) => {
                        tui.resize(Rect::new(0, 0, w, h))?;
                        tui.draw(|f| {
                            let _ = self.root.draw(f, f.area(), &state);
                        })?;
                    }
                    Action::Export(ref record) => {
                        self.export_record(record, &action_tx)?;
                    }
                    Action::RequestSchemasOf(ref key, ref value) => {
                        action_tx.send(Action::Schemas(
                            SchemaDetail::from(&mut schema_registry, key.as_ref()).await,
                            SchemaDetail::from(&mut schema_registry, value.as_ref()).await,
                        ))?;
                    }
                    Action::Render => {
                        let span = tracing::span!(tracing::Level::TRACE, "render");
                        let _ = span.enter();
                        tui.draw(|f| {
                            let _ = self.root.draw(f, f.area(), &state);
                        })?;
                    }
                    Action::SelectedTopics(ref topics) => {
                        self.topics = topics.iter().map(Into::into).collect_vec();
                        self.consume_topics(action_tx.clone())?;
                    }
                    Action::Search(ref search) => {
                        if self.topics.is_empty() {
                            action_tx.send(Action::Notification(Notification::new(
                                Level::Info,
                                "No topics selected".to_string(),
                            )))?;
                        }
                        self.app.search_query = search.clone();
                        self.consume_topics(action_tx.clone())?;
                    }
                    _ => {}
                }

                if let Some(action) = self.root.update(action.clone())? {
                    action_tx.send(action.clone())?;
                }
            }
            if self.should_quit {
                tui.stop();
                break;
            }
        }
        tui.exit()?;
        Ok(())
    }
}
