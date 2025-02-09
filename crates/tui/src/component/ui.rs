//! Module gathering the code to run the terminal user interface.

use app::search::{Search, SearchContext};
use app::App;
use chrono::DateTime;
use crossterm::event::KeyEvent;
use futures::{StreamExt, TryStreamExt};
use itertools::Itertools;
use lib::KafkaRecord;
use log::{error, info, warn};
use ratatui::prelude::Rect;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::OwnedMessage;
use rdkafka::Message;
use std::collections::HashSet;
use std::fs;
use std::time::Duration;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::time::Instant;
use tokio::{select, time};
use tokio_util::sync::CancellationToken;

use crate::action::{Action, Notification};
use crate::component::{Component, RootComponent};
use crate::error::TuiError;
use crate::schema_detail::SchemaDetail;
use crate::tui;

use super::{ConcurrentRecordsBuffer, State, BUFFER};

pub struct Ui {
    app: App,
    should_quit: bool,
    root: RootComponent,
    worker: CancellationToken,
    topics: Vec<String>,
    last_tick_key_events: Vec<KeyEvent>,
    records_sender: Option<UnboundedSender<KafkaRecord>>,
    records: &'static ConcurrentRecordsBuffer,
}

impl Ui {
    pub async fn new(
        app: App,
        query: String,
        selected_topics: Vec<String>,
        state: State,
    ) -> Result<Self, TuiError> {
        let config = app.config.clone();
        Ok(Self {
            should_quit: false,
            worker: CancellationToken::new(),
            app,
            records: &BUFFER,
            topics: vec![],
            root: RootComponent::new(query, selected_topics, &config.global, &BUFFER, state),
            records_sender: None,
            last_tick_key_events: Vec::new(),
        })
    }

    pub fn save_config(&self) -> Result<(), TuiError> {
        let mut config = self.app.config.clone();
        if config.global.history.len() > 1000 {
            config.global.history = config.global.history.into_iter().skip(500).collect();
        }
        fs::write(
            &self.app.config.global.path,
            serde_json::to_string_pretty(&self.app.config.global)?,
        )?;
        Ok(())
    }

    pub(crate) async fn create_consumer(
        app: &App,
        topics: Vec<String>,
        tx: UnboundedSender<Action>,
    ) -> Result<StreamConsumer, TuiError> {
        match app.create_consumer(&topics) {
            Ok(c) => Ok(c),
            Err(e) => {
                tx.send(Action::Notification(Notification::new(
                    log::Level::Error,
                    e.to_string(),
                )))?;
                error!("Something went wrong when trying to consume topics: {}", e);
                Err(e.into())
            }
        }
    }

    pub(crate) async fn consume_topics(
        &mut self,
        tx: UnboundedSender<Action>,
    ) -> Result<(), TuiError> {
        self.worker.cancel();
        self.records.lock().unwrap().reset();
        if self.topics.is_empty() {
            tx.send(Action::StopConsuming())?;
            return Ok(());
        }

        let message = match self.app.search_query.is_empty() {
            true => "Waiting for new events".to_string(),
            false => "Searching".to_string(),
        };

        tx.send(Action::Notification(Notification::new(
            log::Level::Info,
            message,
        )))?;
        self.worker = CancellationToken::new();

        let query = self.app.search_query.query().clone();
        let order_by = query.order_by.clone();
        tx.send(Action::OrderBy(order_by.clone()))?;
        tx.send(Action::NewConsumer())?;
        tx.send(Action::Consuming)?;
        let r = self.records;
        let token = self.worker.clone();
        tokio::spawn(async move {
            while !token.is_cancelled() {
                r.lock().unwrap().sort(&order_by);
                let mut interval = time::interval(Duration::from_secs(1));
                interval.tick().await;
            }
        });
        let r = self.records;
        let token = self.worker.clone();
        let search_query = self.app.search_query.query().clone();
        let app = self.app.clone();
        let txx = tx.clone();
        let topics = self.topics.clone();

        let (tx_dd, mut rx_dd) = mpsc::unbounded_channel::<OwnedMessage>();
        let mut schema_registry = app.schema_registry().clone();
        let token_cloned = token.clone();
        tokio::spawn(async move {
            loop {
                select! {
                    _ = token_cloned.cancelled() => {
                        info!("Consumer is about to be cancelled");
                        return;
                     },
                    Some(message) = rx_dd.recv() => {
                        let record = KafkaRecord::parse(message, &mut schema_registry).await;
                        let context = SearchContext::new(&record);
                        let mut ll = r.lock().unwrap();
                        ll.new_record_read();
                        if search_query.matches(&context) {
                            ll.push(record);
                        }
                        ll.dispatch_metrics();
                        if let Some(limit) = query.limit {
                            if Some(ll.matched_and_read().0) >= Some(limit) {
                                token_cloned.cancel();
                            }
                        }
                    }
                }
            }
        });

        tokio::spawn(async move {
            let _ = tx.send(Action::Consuming);
            let consumer = match Self::create_consumer(&app, topics.clone(), txx.clone()).await {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx.send(Action::StopConsuming());
                    warn!("I was not able to create a consumer: {}", e);
                    return Err("I was not able to create a consumer after 5 attempts...");
                }
            };
            let _ = tx.send(Action::Consuming);
            let assignments = consumer.assignment().unwrap();
            let txx = tx.clone();
            tokio::spawn(async move {
                let count = app
                    .estimate_number_of_records_to_read(assignments)
                    .unwrap_or(0);
                let _ = txx.send(Action::RecordsToRead(count as usize));
            });
            let mut current_time = Instant::now();
            let _ = consumer
                .stream()
                .take_until(token.cancelled())
                .try_for_each(|message| {
                    let message = message.detach();
                    let timestamp = message.timestamp().to_millis().unwrap_or_default();
                    tx_dd.send(message).unwrap();
                    if current_time.elapsed() > Duration::from_secs(13) {
                        current_time = Instant::now();

                        tx.send(Action::Notification(Notification::new(
                            log::Level::Info,
                            format!(
                                "Checkpoint: {}",
                                DateTime::from_timestamp_millis(timestamp).unwrap()
                            ),
                        )))
                        .unwrap();
                    }
                    futures::future::ok(())
                })
                .await;
            consumer.unassign().unwrap();
            info!("Consumer is terminated");
            token.cancel();
            r.lock().unwrap().sort(&query.order_by);
            let _ = tx.send(Action::StopConsuming());
            Ok(())
        });
        Ok(())
    }

    pub(crate) fn topics_details(
        &mut self,
        topics: HashSet<String>,
        action_tx: UnboundedSender<Action>,
    ) -> Result<(), TuiError> {
        let app = self.app.clone();
        tokio::spawn(async move {
            match app.topic_details(topics) {
                Ok(details) => action_tx.send(Action::TopicDetails(details)).unwrap(),
                Err(e) => action_tx
                    .send(Action::Notification(Notification::new(
                        log::Level::Error,
                        e.to_string(),
                    )))
                    .unwrap(),
            }
        });
        Ok(())
    }

    pub(crate) fn export_record(
        &mut self,
        record: &KafkaRecord,
        action_tx: UnboundedSender<Action>,
    ) -> Result<(), TuiError> {
        self.app.export_record(record)?;
        action_tx.send(Action::Notification(Notification::new(
            log::Level::Info,
            "Record exported to the file".to_string(),
        )))?;
        Ok(())
    }

    pub(crate) fn load_topics(
        &mut self,
        action_tx: UnboundedSender<Action>,
    ) -> Result<(), TuiError> {
        let app = self.app.clone();
        tokio::spawn(async move {
            info!("Loading topics");
            match app.list_topics() {
                Ok(topics) => {
                    action_tx.send(Action::Topics(topics)).unwrap();
                }
                Err(e) => {
                    if action_tx
                        .send(Action::Notification(Notification::new(
                            log::Level::Error,
                            e.to_string(),
                        )))
                        .is_err()
                    {
                        error!("Cannot notify the TUI: {:?}", e);
                    }
                    error!("Something went wrong when trying to list topics: {}", e)
                }
            }
        });
        Ok(())
    }

    pub async fn run(&mut self, topics: Vec<String>, state: State) -> Result<(), TuiError> {
        let (action_tx, mut action_rx) = mpsc::unbounded_channel();
        let records_channel = mpsc::unbounded_channel::<KafkaRecord>();
        self.records_sender = Some(records_channel.0);
        self.load_topics(action_tx.clone())?;
        let mut tui = tui::Tui::new()?;
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
                };

                if let Some(action) = self.root.handle_events(Some(e.clone()))? {
                    action_tx.send(action)?;
                }
            }
            while let Ok(action) = action_rx.try_recv() {
                match action {
                    Action::NewSearchPrompt(ref prompt) => {
                        self.app.config.global.history.push(prompt.to_string());
                        self.app.config.global.history.dedup();
                        self.save_config()?;
                    }
                    Action::RequestTopicDetails(ref topics) => {
                        self.topics_details(topics.clone(), action_tx.clone())?;
                    }
                    Action::Tick => {
                        self.last_tick_key_events.drain(..);
                    }
                    Action::Refresh => {
                        self.load_topics(action_tx.clone())?;
                        action_tx.send(Action::Notification(Notification::new(
                            log::Level::Info,
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
                                log::Level::Info,
                                "this action is not available right now".to_string(),
                            )))?;
                            warn!("Cannot open the URL '{}': {}", url, e)
                        }
                    }
                    Action::Resize(w, h) => {
                        tui.resize(Rect::new(0, 0, w, h))?;
                        tui.draw(|f| {
                            let _ = self.root.draw(f, f.area(), &state);
                        })?;
                    }
                    Action::Export(ref record) => {
                        self.export_record(record, action_tx.clone())?;
                    }
                    Action::RequestSchemasOf(ref key, ref value) => {
                        action_tx.send(Action::Schemas(
                            SchemaDetail::from(&mut schema_registry, key).await,
                            SchemaDetail::from(&mut schema_registry, value).await,
                        ))?;
                    }
                    Action::Render => {
                        tui.draw(|f| {
                            let _ = self.root.draw(f, f.area(), &state);
                        })?;
                    }
                    Action::SelectedTopics(ref topics) => {
                        self.topics = topics.iter().map(|t| t.into()).collect_vec();
                        self.consume_topics(action_tx.clone()).await?;
                    }
                    Action::Search(ref search) => {
                        if self.topics.is_empty() {
                            action_tx.send(Action::Notification(Notification::new(
                                log::Level::Info,
                                "No topics selected".to_string(),
                            )))?;
                        }
                        self.app.search_query = search.clone();
                        self.consume_topics(action_tx.clone()).await?;
                    }
                    _ => {}
                }

                if let Some(action) = self.root.update(action.clone())? {
                    action_tx.send(action.clone())?
                };
            }
            if self.should_quit {
                tui.stop()?;
                break;
            }
        }
        tui.exit()?;
        Ok(())
    }
}
