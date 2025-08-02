//! Command to create a new wasm filter.
use std::{
    fs,
    path::{Path, PathBuf},
    process::Stdio,
};

use chrono::Local;
use clap::Args;
use lib::Error;
use strum::{Display, EnumIter, EnumString};
use tracing::{error, info};

use crate::{
    APPLICATION_NAME,
    command::{Command, default_editor},
};
use std::process::Command as ProcessCommand;

#[derive(Debug, Clone, Args)]
pub(crate) struct CreateFilterCommand {
    #[clap(short, long)]
    /// The programming language used to build the WebAssembly module
    language: SupportedLanguages,
    #[clap(long)]
    /// Location of the search filter repository
    directory: Option<PathBuf>,
    /// Name of the search filter
    name: String,
}

#[derive(Debug, Clone, EnumString, EnumIter, Display)]
#[strum(serialize_all = "lowercase")]
pub enum SupportedLanguages {
    Rust,
    Golang,
}

impl Command for CreateFilterCommand {
    async fn execute(&self) -> Result<(), Error> {
        let repo_dir = match &self.directory {
            Some(d) => d.clone(),
            None => std::env::temp_dir().join(format!(
                "{}-filter-{}-{}",
                APPLICATION_NAME,
                self.name.clone(),
                Local::now().timestamp()
            )),
        };

        let editor = default_editor(&None);

        match self.clone_repository(&repo_dir) {
            Ok(()) => {
                self.prepare_git_repository(&repo_dir)?;

                info!(
                    "The filter repository has been initialized: '{}'",
                    repo_dir.display()
                );
                info!(
                    "You can now implement your wasm filter in the repository: '{}'",
                    repo_dir.display()
                );
            }
            Err(e) => {
                error!("I was not able to clone the repository: {e}. Please download it manually.");
                eprintln!("    mkdir -p '{}'", repo_dir.parent().unwrap().display());
                eprintln!(
                    "    curl -L 'https://github.com/MAIF/yozefu/archive/refs/heads/main.zip'"
                );
                eprintln!("    unzip yozefu-main.zip -d .");
                eprintln!("    mv 'yozefu-main' {}", repo_dir.display());
            }
        }

        println!("    {} '{}'", editor, repo_dir.display());
        println!("    make -C '{}' build", repo_dir.display());
        let binary = std::env::current_exe()?;
        println!(
            "    {} import-filter '{}' --name '{}'",
            binary.file_name().unwrap().to_str().unwrap(),
            repo_dir.join("module.wasm").display(),
            self.name
        );
        println!("    rm '{}'", repo_dir.display());
        Ok(())
    }
}

impl CreateFilterCommand {
    /// Clones the [`wasm-blueprints`](https://github.com/MAIF/yozefu/tree/main/crates/wasm-blueprints) repository
    /// and reorganizes directories to keep only the programming language selected by the user.
    fn prepare_git_repository(&self, repo_dir: &Path) -> Result<(), Error> {
        let source = repo_dir
            .join("crates")
            .join("wasm-blueprints")
            .join(self.language.to_string());
        let temp = repo_dir.parent().unwrap().join(
            repo_dir
                .file_name()
                .map(|e| format!("{}-temp", e.to_str().unwrap()))
                .unwrap(),
        );

        fs::rename(source, &temp)?;
        fs::remove_dir_all(repo_dir)?;
        fs::rename(&temp, repo_dir)?;
        info!("Preparing the repository");

        Ok(())
    }

    /// try the clone the filter repository from GitHub with HTTPS.
    fn clone_repository(&self, repo_dir: &Path) -> Result<(), Error> {
        info!("Cloning the filter repository to '{}'", repo_dir.display());
        let output = ProcessCommand::new("git")
            .arg("clone")
            .arg("https://github.com/MAIF/yozefu.git")
            .arg("--depth")
            .arg("1")
            .arg(repo_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?
            .wait_with_output()?;

        match output.status.success() {
            true => Ok(()),
            false => Err(Error::Error(format!(
                "Failed to clone the repository: {}",
                String::from_utf8(output.stderr).unwrap()
            ))),
        }
    }
}

#[tokio::test]
async fn test_success() {
    use itertools::Itertools;

    tracing_subscriber::fmt().init();

    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    let command = CreateFilterCommand {
        language: SupportedLanguages::Rust,
        directory: Some(temp_dir.path().to_path_buf()),
        name: "sha256".to_string(),
    };

    assert!(command.execute().await.is_ok());
    assert!(
        !fs::read_dir(temp_dir.path())
            .unwrap()
            .collect_vec()
            .is_empty(),
        "Running this command should create a directory containing the code for the search filter"
    )
}
