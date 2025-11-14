//! Module gathering code for the headless mode.

use app::App;
use app::search::Search;
use app::search::SearchContext;
use chrono::DateTime;
use futures_batch::TryChunksTimeoutStreamExt;
use rdkafka::Message;
use rdkafka::message::OwnedMessage;
use std::time::Duration;
use std::time::Instant;
use thousands::Separable;
use tokio::select;
use tokio::sync::mpsc;
use tracing::info;

use futures::{StreamExt, TryStreamExt};
use indicatif::ProgressBar;
use lib::Error;
use lib::KafkaRecord;
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
            return Err(Error::Error("Please specify topics to consume".into()));
        }
        info!("Creating consumer for topics [{}]", self.topics.join(", "));
        let consumer = self.app.create_consumer(&self.topics)?;
        let mut records_channel = mpsc::unbounded_channel::<KafkaRecord>();
        let search_query = self.app.search_query.clone();
        let token = CancellationToken::new();
        let progress = self.progress.clone();
        progress.enable_steady_tick(Duration::from_secs(10));
        let count = self
            .app
            .estimate_number_of_records_to_read(&consumer.assignment()?)?;
        progress.set_length(count as u64);

        let (tx_dd, mut rx_dd) = mpsc::unbounded_channel::<OwnedMessage>();
        let mut schema_registry = self.app.schema_registry().clone();
        let token_cloned = token.clone();

        let filters_directory = self.app.config.workspace().filters_dir();
        tokio::task::Builder::new()
            .name("headless-search-engine")
            .spawn(async move {
                loop {
                    let mut limit = 0;
                    select! {
                        () = token_cloned.cancelled() => {
                            return;
                         },
                        Some(message) = rx_dd.recv() => {
                            let record = KafkaRecord::parse(message, &mut schema_registry).await;
                            let context = SearchContext::new(&record, &filters_directory);
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
            })
            .unwrap();

        let consumer_config = self.app.consumer_config();
        tokio::task::Builder::new()
            .name("headless-kafka-consumer")
            .spawn(async move {
                let mut current_time = Instant::now();
                let mut consumed = 0;
                let mut total_consumed = 0;
                let task = consumer
                    .stream()
                    .take_until(token.cancelled())
                    .try_chunks_timeout(consumer_config.buffer_capacity, Duration::from_micros(consumer_config.timeout_in_ms))
                    .try_for_each(|messages| {
                        let timestamp = messages
                            .last()
                            .and_then(|r| r.timestamp().to_millis())
                            .unwrap_or_default();
                        for message in messages {
                            consumed += 1;
                            total_consumed += 1;
                            let message = message.detach();
                            tx_dd.send(message).unwrap();
                        }

                        let elapsed = current_time.elapsed();

                        if elapsed > Duration::from_secs(5) {
                            current_time = Instant::now();
                            info!("Checkpoint: {} records read in {}ms ({} rec/s). Total read is {}. The last record timestamp read is {}",
                            consumed.separate_with_underscores(),
                            elapsed.as_millis().separate_with_underscores(), (consumed / elapsed.as_secs()).separate_with_underscores(),
                            total_consumed.separate_with_underscores(),
                            DateTime::from_timestamp_millis(timestamp).map(|e| e.to_rfc3339()).unwrap_or_default());
                            consumed = 0;
                        }
                        progress.inc(1);
                        futures::future::ok(())
                    })
                    .await;
                info!("Consumer is terminated");
                task
            })
            .unwrap();

        while let Some(record) = records_channel.1.recv().await {
            println!("{}", self.formatter.fmt(&record));
            if self.export_records {
                self.app.export_record(&record)?;
            }
        }
        Ok(())
    }
}
