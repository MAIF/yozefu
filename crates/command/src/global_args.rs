use app::configuration::{GlobalConfig, Workspace};
use clap::Args;
use std::path::PathBuf;
use tracing::debug;

#[derive(Args, Clone, Debug, Default)]
pub struct GlobalArgs {
    #[arg(long, global = true)]
    /// Use a specific config file
    pub config_file: Option<PathBuf>,
    #[arg(long, env = "YOZEFU_CONFIG_DIR", global = true)]
    /// Use a specific config directory to store the configuration, logs, search filters.
    pub config_dir: Option<PathBuf>,
    #[arg(long, env = "YOZEFU_LOG_FILE", global = true)]
    /// Append logs to a specific log file
    pub log_file: Option<PathBuf>,
}

impl GlobalArgs {
    pub fn workspace(&self) -> Workspace {
        let default_workspace = Workspace::default();

        let (workspace_dir, config_file) = match (&self.config_dir, &self.config_file) {
            (Some(dir), Some(file)) => (dir, file),
            (Some(dir), None) => (dir, &dir.join(Workspace::CONFIG_FILENAME)),
            (None, Some(file)) => (&default_workspace.path, file),
            (None, None) => (&default_workspace.path, &default_workspace.config_file()),
        };

        let config = GlobalConfig::read(config_file).unwrap_or(GlobalConfig::new(config_file));

        let log_file = match &self.log_file {
            Some(log_file) => log_file.clone(),
            None => config
                .log_file
                .as_ref()
                .unwrap_or(&workspace_dir.join(Workspace::LOGS_FILENAME))
                .clone(),
        };

        let workspace = Workspace::new(workspace_dir, config, log_file);
        debug!("Using config directory: {}", workspace.path.display());
        workspace
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_config_dir_both_set() {
        let args = GlobalArgs {
            config_dir: Some(PathBuf::from("/tmp/config_dir")),
            config_file: Some(PathBuf::from("/tmp/config_dir/config.json")),
            log_file: None,
        };
        let ws = args.workspace();
        assert_eq!(ws.path, PathBuf::from("/tmp/config_dir"));
        assert_eq!(
            ws.config_file(),
            PathBuf::from("/tmp/config_dir/config.json")
        );
    }

    #[test]
    fn test_config_dir_only_dir() {
        let args = GlobalArgs {
            config_dir: Some(PathBuf::from("/tmp/config_dir")),
            config_file: None,
            log_file: None,
        };
        let ws = args.workspace();
        assert_eq!(ws.path, PathBuf::from("/tmp/config_dir"));
        // The config file should be the default for this workspace
        assert_eq!(
            ws.config_file(),
            Workspace::new(
                &PathBuf::from("/tmp/config_dir"),
                GlobalConfig::new(
                    &args
                        .config_dir
                        .as_ref()
                        .unwrap()
                        .join(Workspace::CONFIG_FILENAME)
                ),
                PathBuf::from("/tmp/config_dir/application.log")
            )
            .config_file()
        );
    }

    #[test]
    fn test_config_dir_only_file() {
        let default_ws = Workspace::default();
        let args = GlobalArgs {
            config_dir: None,
            config_file: Some(PathBuf::from("/tmp/config_dir/config.json")),
            log_file: None,
        };
        let ws = args.workspace();
        assert_eq!(ws.path, default_ws.path);
        assert_eq!(
            ws.config_file(),
            PathBuf::from("/tmp/config_dir/config.json")
        );
    }

    #[test]
    fn test_default() {
        let default_ws = Workspace::default();
        let args = GlobalArgs {
            config_dir: None,
            config_file: None,
            log_file: None,
        };
        let ws = args.workspace();
        assert_eq!(ws.path, default_ws.path);
        assert_eq!(ws.config_file(), default_ws.config_file());
    }

    #[test]
    fn test_config_file_method() {
        let args = GlobalArgs {
            config_dir: Some(PathBuf::from("/tmp/config_dir")),
            config_file: Some(PathBuf::from("/tmp/config_dir/config.json")),
            log_file: None,
        };
        assert_eq!(
            args.workspace().config_file(),
            PathBuf::from("/tmp/config_dir/config.json")
        );
    }
}
