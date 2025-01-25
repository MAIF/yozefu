use std::{fs, path::PathBuf};

use app::configuration::GlobalConfig;
use indexmap::IndexMap;
use lib::Error;
use log::{info, warn};
use tui::Theme;

const THEMES_URL: &str =
    "https://raw.githubusercontent.com/MAIF/yozefu/refs/heads/main/crates/command/themes.json";

/// Initializes a default configuration file if it does not exist.
/// The default cluster is `localhost`.
pub(crate) async fn init_themes_file() -> Result<PathBuf, Error> {
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

/// Update the themes file with the latest themes from the repository.
pub(crate) async fn update_themes() -> Result<PathBuf, Error> {
    let path = GlobalConfig::path()?;
    let config = GlobalConfig::read(&path)?;
    let path = config.themes_file();
    if fs::metadata(&path).is_err() {
        return init_themes_file().await;
    }

    let content = fs::read_to_string(&path)?;
    let mut local_themes: IndexMap<String, Theme> = serde_json::from_str(&content)?;

    info!("Updating themes file from {}", THEMES_URL);
    let content = match reqwest::get(THEMES_URL).await {
        Ok(response) => match response.status().is_success() {
            true => response.text().await.unwrap(),
            false => {
                warn!("HTTP {} when downloading theme file", response.status());
                "{}".to_string()
            }
        },
        Err(e) => {
            warn!("Error while downloading theme file: {}", e);
            "{}".to_string()
        }
    };

    let new_themes = serde_json::from_str::<IndexMap<String, Theme>>(&content)?;

    for (name, theme) in new_themes {
        if !local_themes.contains_key(&name) {
            info!("Theme '{}' added", name);
            local_themes.insert(name, theme);
        }
    }

    fs::write(&path, &serde_json::to_string_pretty(&local_themes)?)?;
    Ok(path)
}

#[test]
fn test_valid_themes() {
    use std::collections::HashMap;
    use tui::Theme;

    let content = include_str!("../themes.json");
    let themes: HashMap<String, Theme> = serde_json::from_str(content).unwrap();
    assert!(themes.keys().len() >= 3)
}
