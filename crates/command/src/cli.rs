//! The command line argument Parser struct
use crate::command::{Command, MainCommand, UtilityCommands};
use app::configuration::{ClusterConfig, GlobalConfig, SchemaRegistryConfig, YozefuConfig};
use app::search::filter::FILTERS_DIR;
use app::APPLICATION_NAME;
use clap::command;
use lib::Error;
use log::warn;
use reqwest::Url;
use std::fmt::Debug;
use std::fs;
use std::{fmt::Display, path::PathBuf, str::FromStr};
use tui::error::TuiError;
use tui::Theme;

pub use clap::Parser;
use indexmap::IndexMap;

/// CLI parser
#[derive(Parser)]
#[command(author, version, about = "A terminal user interface to navigate Kafka topics and search for Kafka records.", name = APPLICATION_NAME, bin_name = APPLICATION_NAME, display_name = APPLICATION_NAME, long_about = None, propagate_version = true, args_conflicts_with_subcommands = true)]
pub struct Cli<T>
where
    T: Display + Debug + Clone + Sync + Send + 'static + FromStr,
    <T as FromStr>::Err: Display,
{
    #[command(subcommand)]
    pub subcommands: Option<UtilityCommands>,
    #[command(flatten)]
    pub default_command: MainCommand<T>,
    #[clap(skip)]
    logs_file: Option<PathBuf>,
}

impl<T> Cli<T>
where
    T: Display + Debug + Clone + Sync + Send + 'static + FromStr,
    <T as FromStr>::Err: Display,
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

    /// The targeted cluster
    pub fn cluster(&self) -> Option<T> {
        match self.subcommands.is_some() {
            true => None,
            false => Some(self.default_command.cluster()),
        }
    }

    /// Changes the default logs file path
    pub fn logs_file(&mut self, logs: PathBuf) -> &mut Self {
        self.logs_file = Some(logs);
        self
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

    async fn run(&self, yozefu_config: Option<YozefuConfig>) -> Result<(), TuiError> {
        init_files().await?;
        let filters_dir = self.read_config()?.filters_dir();
        // TODO this sucks
        *FILTERS_DIR.lock().unwrap() = filters_dir;
        match &self.subcommands {
            Some(c) => c.execute().await.map_err(|e| e.into()),
            None => {
                // Load the config from the yozefu config file
                let yozefu_config = match yozefu_config {
                    None => self.yozefu_config_of(self.default_command.cluster())?,
                    Some(c) => c,
                };
                let command = self.default_command.clone();
                command.execute(yozefu_config).await
            }
        }
    }

    /// Returns the kafka client config
    fn yozefu_config_of(&self, cluster: T) -> Result<YozefuConfig, Error> {
        let config = self.read_config()?;
        match config.clusters.get(&cluster.to_string()) {
            Some(c) => Ok(YozefuConfig::new(c.clone())),
            None => Err(Error::Error(format!("Unknown cluster '{}'. Make sure you have defined a configuration for this cluster name.", cluster))),
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

/// Initializes a default configuration file if it does not exist.
/// The default cluster is `localhost`.
async fn init_themes_file() -> Result<PathBuf, Error> {
    let path = GlobalConfig::path()?;
    let config = GlobalConfig::read(&path)?;
    let path = config.themes_file();
    if fs::metadata(&path).is_ok() {
        return Ok(path);
    }

    let default_theme = Theme::light();
    let mut default_themes = IndexMap::new();
    default_themes.insert(default_theme.name.clone(), default_theme);

    let content = match reqwest::get(
        "https://raw.githubusercontent.com/MAIF/yozefu/refs/heads/main/crates/command/themes.json",
    )
    .await
    {
        Ok(response) => match response.status().is_success() {
            true => response.text().await.unwrap(),
            false => {
                warn!("HTTP {} when downloading theme file", response.status());
                serde_json::to_string_pretty(&default_themes).unwrap()
            }
        },
        Err(e) => {
            warn!("Error while downloading theme file: {}", e);
            serde_json::to_string_pretty(&default_themes).unwrap()
        }
    };

    let e: IndexMap<String, Theme> = match serde_json::from_str(&content) {
        Ok(themes) => themes,
        Err(_) => default_themes,
    };

    fs::write(&path, &serde_json::to_string_pretty(&e)?)?;
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
