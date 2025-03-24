//! Main command for the CLI.
//!
//! To execute the commande, you need:
//! 1. To call `with_client` with a `ClientConfig` to get a `MainCommandWithClient`.
//! 2. To call `execute` on the `MainCommandWithClient`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fs, io};

use app::configuration::{
    ClusterConfig, Configuration, GlobalConfig, InternalConfig, YozefuConfig,
};
use app::search::ValidSearchQuery;

use app::App;
use clap::error::ErrorKind;
use clap::{CommandFactory, Parser};
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use itertools::Itertools;
use lib::Error;
use log::{debug, info, warn};
use rdkafka::consumer::BaseConsumer;
use strum::{Display, EnumString};
use tui::Theme;
use tui::error::TuiError;
use tui::{State, Ui};

use crate::headless::Headless;
use crate::headless::formatter::{
    JsonFormatter, KafkaFormatter, PlainFormatter, SimpleFormatter, TransposeFormatter,
};
use crate::log::{init_logging_file, init_logging_stderr};
use crate::theme::update_themes;
use crate::{APPLICATION_NAME, Cli, Cluster};

fn parse_cluster<T>(s: &str) -> Result<T, Error>
where
    T: Cluster,
{
    s.parse()
        .map_err(|e: <T as FromStr>::Err| Error::Error(e.to_string()))
}

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None, propagate_version = true)]
pub struct MainCommand<T>
where
    T: Cluster,
{
    #[clap(short, long)]
    /// Log level set to 'debug'
    pub debug: bool,
    #[clap(short = 'c', short_alias='e', alias="environment", long, value_parser = parse_cluster::<T>, default_value_t, hide_default_value=true)]
    /// The cluster to use
    cluster: T,
    #[clap(long)]
    /// Topics to consume
    #[clap(
        short,
        long,
        alias = "topic",
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
    #[clap(skip)]
    pub(crate) logs_file: Option<PathBuf>,
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
    T: Cluster,
{
    pub async fn execute(self, mut yozefu_config: YozefuConfig) -> Result<(), TuiError> {
        for property in &self.properties {
            match property.split_once('=') {
                Some((key, value)) => {
                    yozefu_config.set_kafka_property(key, value);
                }
                None => {
                    return Err(TuiError::from(Error::Error(format!(
                        "Invalid kafka property '{}', expected a '=' symbol to separate the property and its value.",
                        property
                    ))));
                }
            }
        }

        match self.headless {
            true => {
                let _ = init_logging_stderr(self.debug);
                self.headless(&yozefu_config).await.map_err(|e| e.into())
            }
            false => {
                // Ignore the result, we just want to make sure the logger is initialized
                self.tui(&yozefu_config).await
            }
        }
    }

    pub(crate) fn cluster(&self) -> T {
        self.cluster.clone()
    }

    /// Returns the kafka client config
    pub fn yozefu_config(&self) -> Result<YozefuConfig, Error> {
        let cluster_config = self.cluster_config(&self.cluster)?;
        Ok(YozefuConfig::new(cluster_config))
    }

    /// Returns the search query to use.
    fn query(&self, config: &GlobalConfig) -> Result<String, Error> {
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

    fn config(&self, yozefu_config: &YozefuConfig) -> Result<GlobalConfig, Error> {
        let path = self.config.clone().unwrap_or(GlobalConfig::path()?);
        let mut config = GlobalConfig::read(&path)?;
        config.logs = yozefu_config.logs_file.clone();
        Ok(config)
    }

    fn themes(file: &Path) -> Result<HashMap<String, Theme>, Error> {
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

    // Validate the cluster name provided by the user.
    // If the cluster name is not provided (`self.cluster` is an empty string), it will return an error.
    // If the cluster name is not found in the configuration file, it will return an error.
    fn cluster_config(&self, cluster: &T) -> Result<ClusterConfig, TuiError> {
        let config = self.read_config()?;
        let available_clusters = config.clusters.keys().collect_vec().into_iter().join(", ");
        match self.cluster().to_string().is_empty() {
            true => {
                let mut cmd = Cli::<T>::command();
                cmd.error(
                    ErrorKind::MissingRequiredArgument,
                    format!(
                        "Argument '--cluster' was not provided. Possible clusters: [{}]",
                        available_clusters
                    ),
                )
                .exit();
            }
            false => {
                if !config.clusters.contains_key(&cluster.to_string()) {
                    return Err(Error::Error(format!(
                        "Unknown cluster '{}'. Possible clusters: [{}].",
                        cluster, available_clusters
                    ))
                    .into());
                }
            }
        };
        Ok(config.clusters.get(&cluster.to_string()).unwrap().clone())
    }

    fn read_config(&self) -> Result<GlobalConfig, Error> {
        match GlobalConfig::read(&GlobalConfig::path()?) {
            Ok(mut config) => {
                config.logs = self.logs_file.clone();
                Ok(config)
            }
            Err(e) => Err(e),
        }
    }

    async fn load_theme(file: &Path, name: &str) -> Result<Theme, Error> {
        let mut themes = Self::themes(file)?;

        if !themes.contains_key(name) {
            info!("Theme '{}' not found. About to update theme file.", name);
            let _ = update_themes().await;
            themes = Self::themes(file)?;
        }

        let theme = match themes.get(name) {
            Some(theme) => theme,
            None => {
                update_themes().await?;
                warn!(
                    "Theme '{}' not found. Available themes are [{}]. Make sure it is defined in '{}'",
                    name,
                    themes.keys().join(", "),
                    file.display()
                );

                let theme = themes.iter().next().unwrap().1;
                info!(
                    "Since the theme was not found, I'm going to use the first available theme '{}'",
                    theme.name
                );
                theme
            }
        };
        Ok(theme.clone())
    }

    /// Starts the app in TUI mode
    async fn tui(&self, yozefu_config: &YozefuConfig) -> Result<(), TuiError> {
        let cluster = self.cluster();
        let config = self.config(yozefu_config)?;
        let query = self.query(&config)?;

        let _ = init_logging_file(self.debug, &config.logs_file());
        let theme_name = self.theme.clone().unwrap_or(config.theme.clone());
        let color_palette = Self::load_theme(&config.themes_file(), &theme_name).await?;
        let state = State::new(&cluster.to_string(), color_palette, &config);
        let mut ui = Ui::new(
            self.app(&query, yozefu_config)?,
            query,
            self.topics.clone(),
            state.clone(),
        )
        .await?;

        self.check_connection(yozefu_config)?;
        ui.run(self.topics.clone(), state).await
    }

    fn check_connection(&self, yozefu_config: &YozefuConfig) -> Result<(), Error> {
        let _ = yozefu_config.create_kafka_consumer::<BaseConsumer>()?;
        Ok(())
    }

    /// Creates the App
    fn app(&self, query: &str, yozefu_config: &YozefuConfig) -> Result<App, Error> {
        debug!("{:?}", yozefu_config);
        let config = self.config(yozefu_config)?;
        let search_query = ValidSearchQuery::from(query, &config.filters_dir())?;

        let internal_config = InternalConfig::new(yozefu_config.clone(), config);
        //let output_file = internal_config.output_file();
        Ok(App::new(
            self.cluster().to_string(),
            internal_config,
            search_query,
        ))
    }

    /// Starts the app in headless mode
    async fn headless(&self, yozefu_config: &YozefuConfig) -> Result<(), Error> {
        let config = self.config(yozefu_config)?;
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
        let topics = self.topics(yozefu_config)?;
        if !self.disable_progress {
            progress.set_draw_target(ProgressDrawTarget::stderr());
        }
        let app = Headless::new(
            self.app(&query, yozefu_config)?,
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
                cluster,
                topics.join(","),
                query
            )
        }
    }

    /// Lists available topics when the user didn't provide any
    fn topics(&self, yozefu_config: &YozefuConfig) -> Result<Vec<String>, Error> {
        if !self.topics.is_empty() {
            return Ok(self.topics.clone());
        }
        let items = App::list_topics_from_client(yozefu_config)?;
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
}
