use reqwest::Client;
use std::{fs, path::PathBuf};

use app::configuration::Workspace;
use indexmap::IndexMap;
use lib::Error;
use tracing::{info, warn};
use tui::Theme;

const THEMES_URL: &str =
    "https://raw.githubusercontent.com/MAIF/yozefu/refs/heads/main/crates/command/themes.json";

/// Initializes a default configuration file if it does not exist.
/// The default cluster is `localhost`.
pub(crate) async fn init_themes_file(workspace: &Workspace) -> Result<PathBuf, Error> {
    let path = workspace.themes_file();
    if fs::metadata(&path).is_ok() {
        return Ok(path);
    }

    let themes = get_themes_from_github().await;
    fs::write(&path, &serde_json::to_string_pretty(&themes)?)?;
    Ok(path)
}

/// Update the themes file with the latest themes from the repository.
pub(crate) async fn update_themes(workspace: &Workspace) -> Result<PathBuf, Error> {
    let path = workspace.themes_file();
    if fs::metadata(&path).is_err() {
        return init_themes_file(workspace).await;
    }

    let content = fs::read_to_string(&path)?;
    let mut local_themes: IndexMap<String, Theme> = serde_json::from_str(&content)?;

    info!("Updating themes file from {THEMES_URL}");
    let new_themes = get_themes_from_github().await;
    for (name, theme) in new_themes {
        if !local_themes.contains_key(&name) {
            info!("Theme '{name}' added");
            local_themes.insert(name, theme);
        }
    }

    fs::write(&path, &serde_json::to_string_pretty(&local_themes)?)?;
    Ok(path)
}

async fn get_themes_from_github() -> IndexMap<String, Theme> {
    let default_theme = Theme::light();
    let mut default_themes = IndexMap::new();
    default_themes.insert(default_theme.name.clone(), default_theme);

    let http_client = Client::new();
    let content = match http_client
        .get(THEMES_URL)
        .timeout(std::time::Duration::from_millis(1_000))
        .send()
        .await
    {
        Ok(response) => match response.status().is_success() {
            true => response.text().await.unwrap(),
            false => {
                warn!("HTTP {} when downloading theme file", response.status());
                serde_json::to_string_pretty(&default_themes).unwrap_or_default()
            }
        },
        Err(e) => {
            warn!("Error while downloading the theme file: {e}");
            serde_json::to_string_pretty(&default_themes).unwrap_or_default()
        }
    };

    let themes: IndexMap<String, Theme> = match serde_json::from_str(&content) {
        Ok(themes) => themes,
        Err(e) => {
            warn!("Error while parsing themes from GitHub: {e}");
            default_themes
        }
    };
    themes
}

#[test]
fn test_valid_themes() {
    use std::collections::HashMap;
    use tui::Theme;

    let content = include_str!("../themes.json");
    let themes: HashMap<String, Theme> = serde_json::from_str(content).unwrap();
    assert!(themes.keys().len() >= 3)
}
