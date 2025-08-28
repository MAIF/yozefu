use std::path::Path;

use crate::highlighter::Highlighter;

#[test]
pub fn test_theme() {
    let theme = Highlighter::theme(&Some("unknown-theme".to_string()), "unknown-theme-again");
    assert_eq!(theme.name, Some("Base16 Ocean Dark".to_string()));
}

#[test]
pub fn test_theme_from_file() {
    let theme_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("theme.tmTheme");

    let theme = Highlighter::theme(&Some(theme_file.display().to_string()), "unknown-theme");
    assert_eq!(theme.name, Some("Burgundy".to_string()));
}

#[test]
pub fn test_theme_yozefu_default_theme() {
    let theme = Highlighter::theme(&Some("unknown-theme".into()), "InspiredGitHub");
    assert_eq!(theme.name, Some("GitHub".to_string()));
}
