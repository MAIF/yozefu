//! The command line argument Parser struct
use crate::cluster::Cluster;
use crate::command::{Command, MainCommand, UtilityCommands};
use crate::theme::init_themes_file;
use crate::version::VERSION_MESSAGE;
use app::configuration::{
    ClusterConfig, GlobalConfig, SchemaRegistryConfig, Workspace, YozefuConfig,
};
use app::{APPLICATION_NAME, BINARY_NAME};
use clap::command;
use lib::Error;
use reqwest::Url;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tui::error::TuiError;

pub use clap::Parser;
use indexmap::IndexMap;

// https://github.com/clap-rs/clap/issues/975
/// CLI parser
#[derive(Parser)]
#[command(author,
    version = VERSION_MESSAGE,
    about = "A terminal user interface to navigate Kafka topics and search for Kafka records.", 
    name = APPLICATION_NAME,
    bin_name = BINARY_NAME,
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
        self.init_files().await?;
        match &self.subcommands {
            Some(c) => c.execute().await.map_err(std::convert::Into::into),
            None => {
                // Load the config from the yozefu config file
                let yozefu_config = match yozefu_config {
                    None => self.default_command.yozefu_config()?,
                    Some(c) => c,
                };
                let mut command = self.default_command.clone();
                command.logs_file.clone_from(&self.logs_file);
                command.execute(yozefu_config).await
            }
        }
    }

    /// Initializes a default configuration file if it does not exist.
    /// The default cluster is `localhost`.
    async fn init_files(&self) -> Result<(), Error> {
        let workspace = match &self.subcommands {
            Some(UtilityCommands::Config(c)) => &c.global,
            _ => &self.default_command.global,
        }
        .workspace();

        init_config_file(&workspace)?;
        init_themes_file(&workspace).await?;
        Ok(())
    }
}

/// Initializes a default configuration file if it does not exist.
/// The default cluster is `localhost`.
fn init_config_file(workspace: &Workspace) -> Result<PathBuf, Error> {
    let path = workspace.config_file();
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
                headers: HashMap::default(),
            }),
            ..Default::default()
        },
    );

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            std::io::Error::new(
                e.kind(),
                format!("Failed to create the configuration directory '{}': {}. Check you have write permissions.", parent.display(), e)
            )
        })?;
        fs::create_dir_all(workspace.filters_dir()).map_err(|e| {
            std::io::Error::new(
                e.kind(),
                format!("Failed to create the filters directory '{}': {}. Check you have write permissions.", workspace.filters_dir().display(), e)
            )
        })?;
    }
    fs::write(&path, serde_json::to_string_pretty(&config).unwrap())
        .map_err(|e| {
        std::io::Error::new(
            e.kind(),
            format!("Failed to initialize the configuration file '{}': {}. Check you have write permissions.", path.display(), e)
        )
    })?;

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

#[test]
fn initialize_config_file_on_readonly_root_partition() {
    let workspace = Workspace::new(
        &PathBuf::from("/tmp/yozefu-readonly"),
        &PathBuf::from("/tmp/yozefu-readonly/config.json"),
    );
    let directory = &workspace.path;

    let _ = fs::remove_dir_all(directory);
    fs::create_dir_all(directory).unwrap();
    // change permissions of temp_dir to read-only
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms: fs::Permissions = fs::metadata(directory).unwrap().permissions();
        perms.set_mode(0o000);
        let _ = fs::set_permissions(directory, perms);
        // suppose to return an error
        let result = init_config_file(&workspace);
        assert!(result.is_err());
    }
}
