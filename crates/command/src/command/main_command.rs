//! Main command for the CLI.
//!
//! To execute the commande, you need:
//! 1. To call `with_client` with a `ClientConfig` to get a `MainCommandWithClient`.
//! 2. To call `execute` on the `MainCommandWithClient`.

use std::collections::HashMap;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fs, io};

use app::search::ValidSearchQuery;
use chrono::Local;

use app::{App, Config};
use clap::Parser;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use itertools::Itertools;
use lib::Error;
use log::{debug, info, warn};
use rdkafka::ClientConfig;
use strum::{Display, EnumString};
use tui::error::TuiError;
use tui::Theme;
use tui::{State, Ui};

use crate::cli::config_path;
use crate::headless::formatter::{
    JsonFormatter, KafkaFormatter, PlainFormatter, SimpleFormatter, TransposeFormatter,
};
use crate::headless::Headless;
use crate::log::init_logging_file;
use crate::APPLICATION_NAME;

use super::main_command_with_client::MainCommandWithClient;

fn parse_cluster<T>(s: &str) -> Result<T, Error>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    s.parse()
        .map_err(|e: <T as FromStr>::Err| Error::Error(e.to_string()))
}

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None, propagate_version = true)]
pub struct MainCommand<T>
where
    T: Display + Clone + Sync + Send + 'static + FromStr,
    <T as FromStr>::Err: Display,
{
    #[clap(short, long)]
    /// Log level set to 'debug'
    pub debug: bool,
    #[clap(short = 'c', short_alias='e', alias="environment", long, value_parser = parse_cluster::<T>, required_unless_present_any=&["version", "help"])]
    /// The cluster to use
    cluster: Option<T>,
    /// Topics to consume
    #[clap(
        short,
        long,
        group = "topic",
        use_value_delimiter = true,
        value_delimiter = ','
    )]
    pub topics: Vec<String>,
    /// Override kafka consumer properties, see <https://docs.confluent.io/platform/current/installation/configuration/consumer-configs.html>
    #[clap(short, long)]
    pub properties: Vec<String>,
    #[clap(long)]
    /// Disable the TUI, print results in stdout instead.
    pub headless: bool,
    /// The initial search query. If you start the query with the letter @, the rest should be a filename to read the data from, or - if you want yozefu to read the data from stdin.
    query: Vec<String>,
    #[clap(long)]
    /// Theme to use
    pub theme: Option<String>,
    #[clap(long, requires = "headless")]
    /// Specify the output format of kafka records
    pub format: Option<KafkaFormatterOption>,
    #[clap(long, requires = "headless")]
    /// Disable progress in stderr
    pub disable_progress: bool,
    #[clap(long, requires = "headless")]
    /// Export kafka records in the given file
    pub export: bool,
    #[clap(short, long)]
    /// Name of the file to export kafka records
    pub output: Option<PathBuf>,
    #[clap(long)]
    /// Use a specific config file
    pub config: Option<PathBuf>,
}

#[derive(Debug, Clone, EnumString, Display)]
#[strum(serialize_all = "lowercase")]
pub enum KafkaFormatterOption {
    Transpose,
    Simple,
    Plain,
    Human,
    Json,
    Log,
}

impl<T> MainCommand<T>
where
    T: Display + Clone + Sync + Send + 'static + FromStr,
    <T as FromStr>::Err: Display,
{
    /// Create a new `MainCommandWithClient` with a `ClientConfig`.
    pub fn with_client(
        self,
        client_config: ClientConfig,
    ) -> Result<MainCommandWithClient<T>, Error> {
        let kafka_properties = client_config.config_map().clone();
        let kafka_properties = self.override_kafka_config_properties(kafka_properties)?;
        let client_config = Self::kafka_client_config_from_properties(kafka_properties)?;
        Ok(MainCommandWithClient::new(self, client_config))
    }

    /// Returns the search query to use.
    pub(crate) fn query(&self, config: &Config) -> Result<String, Error> {
        App::load_config(config);
        let q = self.query.join(" ").trim().to_string();
        if q.is_empty() {
            return Ok(config.initial_query.clone());
        }

        if q == "-" {
            info!("Reading query from stdin");
            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer)?;
            return Ok(buffer);
        }

        match q.starts_with("@") {
            true => {
                let query_file = Path::new(&q[1..]);
                fs::read_to_string(query_file).map_err(|e| {
                    Error::Error(format!(
                        "Cannot read search query from file {:?}: {}",
                        query_file.display(),
                        e
                    ))
                })
            }
            false => Ok(q),
        }
    }

    pub(crate) fn config(&self) -> Result<Config, Error> {
        let path = self.config.clone().unwrap_or(config_path());
        Config::read(&path)
    }

    pub fn cluster(&self) -> T {
        self.cluster.as_ref().unwrap().clone()
    }

    pub fn themes(file: &Path) -> Result<HashMap<String, Theme>, Error> {
        let content = fs::read_to_string(file)?;
        let themes: HashMap<String, Theme> = serde_json::from_str(&content).map_err(|e| {
            Error::Error(format!(
                "Error while parsing themes file '{}': {}",
                file.display(),
                e
            ))
        })?;
        Ok(themes)
    }

    pub fn load_theme(file: &Path, name: &str) -> Result<Theme, Error> {
        let themes = Self::themes(file)?;
        let theme = match themes.get(name) {
            Some(theme) => theme,
            None => {
                warn!("Theme '{}' not found. Available themes are [{}]. Make sure it is defined in '{}'",

                name,
                themes.keys().join(", "),
                file.display());
                let theme = themes.iter().next().unwrap().1;
                info!("Since the theme was not found, I'm going to use the first available theme '{}'", theme.name);
                theme
            }
        };
        Ok(theme.clone())
    }

    /// Starts the app in TUI mode
    pub(crate) async fn tui(&self, kafka_client_config: ClientConfig) -> Result<(), TuiError> {
        let cluster = self.cluster();
        let config = self.config()?;
        let query = self.query(&config)?;

        let theme_name = self.theme.clone().unwrap_or(config.theme.clone());
        let color_palette = Self::load_theme(&config.themes_file(), &theme_name)?;

        let state = State::new(&cluster.to_string(), color_palette, &config);
        let mut app = Ui::new(
            self.app(&query, kafka_client_config)?,
            query,
            self.topics.clone(),
            state.clone(),
        )
        .await?;

        let _ = init_logging_file(self.debug, &config.logs_file());
        app.run(self.topics.clone(), state).await
    }

    /// Creates the App
    fn app(&self, query: &str, kafka_client_config: ClientConfig) -> Result<App, Error> {
        let config = self.config()?;
        let search_query = ValidSearchQuery::from_str(query)?;
        Ok(App::new(
            config,
            self.cluster().to_string(),
            kafka_client_config,
            search_query,
            self.output_file()?,
        ))
    }

    /// Returns the output file to use to store exported kafka records.
    pub(crate) fn output_file(&self) -> Result<PathBuf, Error> {
        let output = match &self.output {
            Some(o) => o.clone(),
            None => {
                let config = self.config()?;
                config.export_directory.join(format!(
                    "export-{}.json",
                    // Windows does not support ':' in filenames
                    Local::now()
                        .to_rfc3339_opts(chrono::SecondsFormat::Secs, false)
                        .replace(':', "-"),
                ))
            }
        };
        Ok(output)
    }

    /// Starts the app in headless mode
    pub(crate) async fn headless(&self, kafka_client_config: ClientConfig) -> Result<(), Error> {
        let config = self.config()?;
        let query = self.query(&config)?;

        let progress = ProgressBar::new(0);
        let date = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        progress.set_draw_target(ProgressDrawTarget::hidden());
        progress.set_style(
            ProgressStyle::with_template(&format!(
                "[{date} {{msg:.green}}  headless] {{pos}} records read {{per_sec}}"
            ))
            .map_err(|e| Error::Error(e.to_string()))?,
        );
        progress.set_message("INFO");

        let topics = self.topics(&kafka_client_config)?;
        if !self.disable_progress {
            progress.set_draw_target(ProgressDrawTarget::stderr());
        }
        let app = Headless::new(
            self.app(&query, kafka_client_config)?,
            &topics,
            self.formatter(),
            self.export,
            progress,
        );

        self.print_full_command(&self.cluster().to_string(), &topics, &query);

        app.run().await?;
        self.print_full_command(&self.cluster().to_string(), &topics, &query);
        Ok(())
    }

    fn print_full_command(&self, cluster: &str, topics: &[String], query: &str) {
        if self.topics.is_empty() {
            let binary = std::env::current_exe()
                .map(|f| f.file_name().unwrap().to_str().unwrap().to_string())
                .unwrap_or(APPLICATION_NAME.to_string());
            info!(
                "Executed command: {} -c {} --headless --topics {} '{}'",
                binary,
                cluster.to_string(),
                topics.join(","),
                query
            )
        }
    }

    /// Lists available topics when the user didn't provide any
    fn topics(&self, kafka_config: &ClientConfig) -> Result<Vec<String>, Error> {
        if !self.topics.is_empty() {
            return Ok(self.topics.clone());
        }
        let items = App::list_topics_from_client(kafka_config)?;
        println!(
            "Select topics to consume:\n {}",
            items.iter().take(20).join("\n ")
        );
        if items.len() > 20 {
            println!("... and {} more", items.len() - 20);
        }
        std::process::exit(1)
    }

    /// Creates a formatter for the headless mode
    fn formatter(&self) -> Box<dyn KafkaFormatter> {
        match &self.format {
            Some(d) => match d {
                KafkaFormatterOption::Transpose => Box::new(TransposeFormatter::new()),
                KafkaFormatterOption::Simple => Box::new(SimpleFormatter::new()),
                KafkaFormatterOption::Plain => Box::new(PlainFormatter::new()),
                KafkaFormatterOption::Json => Box::new(JsonFormatter::new()),
                KafkaFormatterOption::Human => Box::new(SimpleFormatter::new()),
                KafkaFormatterOption::Log => Box::new(PlainFormatter::new()),
            },
            None => Box::new(TransposeFormatter::new()),
        }
    }

    /// Overrides the kafka properties with the properties provided by the user
    fn override_kafka_config_properties(
        &self,
        mut config: HashMap<String, String>,
    ) -> Result<HashMap<String, String>, Error> {
        for property in &self.properties {
            match property.split_once('=') {
                Some((key, value)) => {
                    config.insert(key.trim().into(), value.into());
                }
                None => {
                    return Err(Error::Error(format!("Invalid kafka property '{}', expected a '=' symbol to separate the property and its value.", property)));
                }
            }
        }
        Ok(config)
    }

    /// Returns the kafka client config from the configuration file
    pub(crate) fn kafka_client_config(&self) -> Result<ClientConfig, Error> {
        let config = self.config()?;
        let mut kafka_properties = config.kafka_config_of(&self.cluster().to_string())?;
        kafka_properties = self.override_kafka_config_properties(kafka_properties)?;

        // Default properties
        for (key, value) in [
            ("group.id", APPLICATION_NAME),
            ("enable.auto.commit", "false"),
        ] {
            if !kafka_properties.contains_key(key) {
                kafka_properties.insert(key.into(), value.into());
            }
        }

        Self::kafka_client_config_from_properties(kafka_properties)
    }

    /// Returns the kafka client config from kafka properties
    pub fn kafka_client_config_from_properties(
        kafka_properties: HashMap<String, String>,
    ) -> Result<ClientConfig, Error> {
        let mut config = ClientConfig::new();
        config.set_log_level(rdkafka::config::RDKafkaLogLevel::Emerg);
        debug!(
            "Kafka properties: {:?}",
            kafka_properties
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .join(", ")
        );
        for (key, value) in kafka_properties {
            config.set(key, value);
        }

        Ok(config)
    }
}
