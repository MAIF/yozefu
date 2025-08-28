use std::path::Path;

use crate::highlighter::Highlighter;

#[test]
pub fn test_theme() {
    let theme = Highlighter::theme(
        &Some("unknown-theme".to_string()),
        Some("unknown-theme-again"),
    );
    assert_eq!(theme.and_then(|t| t.name), None);
}

#[test]
pub fn test_disable() {
    let theme = Highlighter::theme(&None, None);
    assert_eq!(theme, None);
}

#[test]
pub fn test_theme_from_file() {
    let theme_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("theme.tmTheme");

    let theme = Highlighter::theme(
        &Some(theme_file.display().to_string()),
        Some("unknown-theme"),
    );
    assert_eq!(theme.and_then(|t| t.name), Some("Burgundy".to_string()));
}

#[test]
pub fn test_theme_yozefu_default_theme() {
    let theme = Highlighter::theme(&Some("unknown-theme".into()), Some("InspiredGitHub"));
    assert_eq!(theme.and_then(|t| t.name), Some("GitHub".to_string()));
}
