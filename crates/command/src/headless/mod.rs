//! Module gathering code for the headless mode.

use app::search::Search;
use app::search::SearchContext;
use app::App;
use rdkafka::message::OwnedMessage;
use rdkafka::Message;
use std::time::Duration;
use std::time::Instant;
use tokio::select;
use tokio::sync::mpsc;

use futures::{StreamExt, TryStreamExt};
use indicatif::ProgressBar;
use lib::Error;
use lib::KafkaRecord;
use log::info;
use rdkafka::consumer::Consumer;
use tokio_util::sync::CancellationToken;

use self::formatter::KafkaFormatter;
pub mod formatter;

pub struct Headless {
    app: App,
    pub(crate) topics: Vec<String>,
    pub(crate) formatter: Box<dyn KafkaFormatter>,
    progress: ProgressBar,
    export_records: bool,
}

impl Headless {
    pub fn new(
        app: App,
        topics: &[String],
        formatter: Box<dyn KafkaFormatter>,
        export_records: bool,
        progress: ProgressBar,
    ) -> Self {
        Self {
            app,
            topics: topics.to_owned(),
            formatter,
            progress,
            export_records,
        }
    }

    pub async fn run(&self) -> Result<(), Error> {
        if self.topics.is_empty() {
            return Err("Please specify topics to consume".into());
        }
        info!("Creating consumer for topics [{}]", self.topics.join(", "));
        let consumer = self.app.create_consumer(&self.topics)?;
        let mut records_channel = tokio::sync::mpsc::unbounded_channel::<KafkaRecord>();
        let search_query = self.app.search_query.clone();
        let token = CancellationToken::new();
        let progress = self.progress.clone();
        progress.enable_steady_tick(Duration::from_secs(10));
        let count = self
            .app
            .estimate_number_of_records_to_read(consumer.assignment()?)?;
        progress.set_length(count as u64);

        let (tx_dd, mut rx_dd) = mpsc::unbounded_channel::<OwnedMessage>();
        let mut schema_registry = self.app.schema_registry().clone();
        let token_cloned = token.clone();
        tokio::spawn(async move {
            loop {
                let mut limit = 0;
                select! {
                    _ = token_cloned.cancelled() => {
                        info!("Consumer is about to be cancelled");
                        return;
                     },
                    Some(message) = rx_dd.recv() => {
                        let record = KafkaRecord::parse(message, &mut schema_registry).await;
                        let context = SearchContext::new(&record);
                        if search_query.matches(&context) {
                            records_channel.0.send(record).unwrap();
                            limit += 1;
                        }
                        if let Some(query_limit) = search_query.limit() {
                            if limit >= query_limit {
                                token_cloned.cancel();
                            }
                        }
                    }
                }
            }
        });

        tokio::spawn(async move {
            let mut current_time = Instant::now();
            let task = consumer
                .stream()
                .take_until(token.cancelled())
                .try_for_each(|message| {
                    let message = message.detach();
                    let timestamp = message.timestamp().to_millis().unwrap_or_default();
                    tx_dd.send(message).unwrap();

                    if current_time.elapsed() > Duration::from_secs(10) {
                        current_time = Instant::now();
                        info!("Checkpoint: {}", timestamp);
                    }
                    progress.inc(1);
                    futures::future::ok(())
                })
                .await;
            info!("Consumer is terminated");
            task
        });

        while let Some(record) = records_channel.1.recv().await {
            println!("{}", self.formatter.fmt(&record));
            if self.export_records {
                self.app.export_record(&record)?;
            }
        }
        Ok(())
    }
}
