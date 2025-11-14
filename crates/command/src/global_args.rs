use app::configuration::Workspace;
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
}

impl GlobalArgs {
    pub fn workspace(&self) -> Workspace {
        let default_workspace = Workspace::default();
        let workspace = match (&self.config_dir, &self.config_file) {
            (Some(dir), Some(file)) => Workspace::new(dir, file),
            (Some(dir), None) => Workspace::new(dir, &dir.join(Workspace::CONFIG_FILENAME)),
            (None, Some(file)) => Workspace::new(&default_workspace.path, file),
            (None, None) => default_workspace,
        };

        debug!("Using config directory: {}", workspace.path.display());
        workspace
    }

    //pub fn config_file(&self) -> PathBuf {
    //    self.config_dir().global_config()
    //}
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
        };
        let ws = args.workspace();
        assert_eq!(ws.path, PathBuf::from("/tmp/config_dir"));
        // The config file should be the default for this workspace
        assert_eq!(
            ws.config_file(),
            Workspace::new(
                &PathBuf::from("/tmp/config_dir"),
                args.config_dir
                    .as_ref()
                    .unwrap()
                    .join(Workspace::CONFIG_FILENAME)
                    .as_path(),
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
        };
        assert_eq!(
            args.workspace().config_file(),
            PathBuf::from("/tmp/config_dir/config.json")
        );
    }
}
