//! The command line argument Parser struct
use crate::cluster::Cluster;
use crate::command::{Command, MainCommand, UtilityCommands};
use crate::theme::init_themes_file;
use app::APPLICATION_NAME;
use app::configuration::{ClusterConfig, GlobalConfig, SchemaRegistryConfig, YozefuConfig};
use clap::command;
use lib::Error;
use reqwest::Url;
use std::fs;
use std::path::PathBuf;
use tui::error::TuiError;

pub use clap::Parser;
use indexmap::IndexMap;

const VERSION_MESSAGE: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "\n",
    env!("CARGO_PKG_REPOSITORY"),
    "\n",
    env!("CARGO_PKG_AUTHORS")
);

// https://github.com/clap-rs/clap/issues/975
/// CLI parser
#[derive(Parser)]
#[command(author,
    version = VERSION_MESSAGE,
    about = "A terminal user interface to navigate Kafka topics and search for Kafka records.", 
    name = APPLICATION_NAME,
    bin_name = APPLICATION_NAME,
    display_name = APPLICATION_NAME,
    long_about = None,
    propagate_version = true,
    args_conflicts_with_subcommands = true
)]
pub struct Cli<T>
where
    T: Cluster,
{
    #[command(subcommand)]
    subcommands: Option<UtilityCommands>,
    #[command(flatten)]
    default_command: MainCommand<T>,
    #[clap(skip)]
    logs_file: Option<PathBuf>,
}

impl<T> Cli<T>
where
    T: Cluster,
{
    /// Executes the CLI.
    /// The config will be loaded from the default config file.
    pub async fn execute(&self) -> Result<(), TuiError> {
        self.run(None).await
    }

    /// Executes the CLI with a specified kafka config client
    pub async fn execute_with(&self, yozefu_config: YozefuConfig) -> Result<(), TuiError> {
        self.run(Some(yozefu_config)).await
    }

    /// This function returns `Some(T)` when the user starts the TUI.
    /// Otherwise, for subcommands commands that such `config` or `new-filter`, it returns `None`.
    pub fn cluster(&self) -> Option<T> {
        match self.subcommands.is_some() {
            true => None,
            false => Some(self.default_command.cluster()),
        }
    }

    pub fn is_main_command(&self) -> bool {
        self.cluster().is_some()
    }

    /// Changes the default logs file path
    pub fn logs_file(&mut self, logs: PathBuf) -> &mut Self {
        self.logs_file = Some(logs);
        self
    }

    async fn run(&self, yozefu_config: Option<YozefuConfig>) -> Result<(), TuiError> {
        init_files().await?;
        match &self.subcommands {
            Some(c) => c.execute().await.map_err(|e| e.into()),
            None => {
                // Load the config from the yozefu config file
                let yozefu_config = match yozefu_config {
                    None => self.default_command.yozefu_config()?,
                    Some(c) => c,
                };
                let mut command = self.default_command.clone();
                command.logs_file = self.logs_file.clone();
                command.execute(yozefu_config).await
            }
        }
    }
}

/// Initializes a default configuration file if it does not exist.
/// The default cluster is `localhost`.
async fn init_files() -> Result<(), Error> {
    init_config_file()?;
    init_themes_file().await?;
    Ok(())
}

/// Initializes a default configuration file if it does not exist.
/// The default cluster is `localhost`.
fn init_config_file() -> Result<PathBuf, Error> {
    let path = GlobalConfig::path()?;
    if fs::metadata(&path).is_ok() {
        return Ok(path);
    }
    let mut config = GlobalConfig::try_from(&path)?;
    let mut localhost_config = IndexMap::new();
    localhost_config.insert(
        "bootstrap.servers".to_string(),
        "localhost:9092".to_string(),
    );
    localhost_config.insert("security.protocol".to_string(), "plaintext".to_string());
    localhost_config.insert("broker.address.family".to_string(), "v4".to_string());
    config
        .default_kafka_config
        .insert("fetch.min.bytes".to_string(), "10000".to_string());

    config.clusters.insert(
        "localhost".into(),
        ClusterConfig {
            kafka: localhost_config,
            schema_registry: Some(SchemaRegistryConfig {
                url: Url::parse("http://localhost:8081").unwrap(),
                headers: Default::default(),
            }),
            ..Default::default()
        },
    );

    fs::create_dir_all(config.filters_dir())?;
    fs::write(&path, serde_json::to_string_pretty(&config).unwrap()).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms: fs::Permissions = fs::metadata(&path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&path, perms)?;
    }

    Ok(path)
}

#[test]
pub fn test_conflicts() {
    use clap::CommandFactory;
    Cli::<String>::command().debug_assert();
}
#[test]
fn test_valid_themes() {
    use std::collections::HashMap;
    use tui::Theme;

    let content = include_str!("../themes.json");
    let themes: HashMap<String, Theme> = serde_json::from_str(content).unwrap();
    assert!(themes.keys().len() >= 3)
}
