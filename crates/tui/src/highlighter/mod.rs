//! A syntax highlighter using syntect,
//! used to highlight JSON payloads in `RecordDetailsComponent`.
use lib::DataType;
use ratatui::text::{Line, Span, Text};
use resolve_path::PathResolveExt;
use std::{path::PathBuf, sync::LazyLock};
use syntect::{
    easy::HighlightLines,
    highlighting::{self, Theme, ThemeSet},
    parsing::{SyntaxReference, SyntaxSet},
    util::LinesWithEndings,
};
use tracing::warn;

static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
static THEMES: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);
pub const HIGHLIGHTER_DEFAULT_THEME: &str = "base16-ocean.dark";

pub const HIGHLIGHTER_THEMES: [&str; 7] = [
    HIGHLIGHTER_DEFAULT_THEME,
    "base16-eighties.dark",
    "base16-mocha.dark",
    "base16-ocean.light",
    "InspiredGitHub",
    "Solarized (dark)",
    "Solarized (light)",
];

#[derive(Debug, Clone)]
pub struct Highlighter {
    syntax: SyntaxReference,
    theme: highlighting::Theme,
    enabled: bool,
}

impl Default for Highlighter {
    fn default() -> Self {
        Self::new(Some(THEMES.themes[HIGHLIGHTER_DEFAULT_THEME].clone()))
    }
}

impl<'a> Highlighter {
    /// `name` and `fallback` could be a absolute file path to a `.tmTheme` file or a syntect theme name.
    pub fn theme(name: Option<&str>, fallback: Option<&str>) -> Option<highlighting::Theme> {
        match name.or(fallback) {
            Some(theme) => Self::try_to_load(name)
                .or(Self::try_to_load(fallback))
                .or_else(|| {
                    warn!("Warning: Theme '{theme}' not found");
                    None
                }),
            None => None,
        }
    }

    pub fn try_to_load(name: Option<&str>) -> Option<Theme> {
        let name = name?;
        let name = PathBuf::from(&name)
            .resolve()
            .canonicalize()
            .map(|d| d.display().to_string())
            .unwrap_or(name.to_string());

        ThemeSet::get_theme(&name)
            .ok()
            .or(THEMES.themes.get(&name).cloned())
    }

    pub fn new(theme: Option<highlighting::Theme>) -> Self {
        match theme {
            Some(t) => Self {
                theme: t,
                syntax: SYNTAX_SET.find_syntax_by_extension("json").unwrap().clone(),
                enabled: true,
            },
            None => Self::disabled(),
        }
    }

    fn disabled() -> Self {
        Self {
            theme: THEMES
                .themes
                .get(HIGHLIGHTER_DEFAULT_THEME)
                .expect("Cannot load the default theme")
                .clone(),
            syntax: SYNTAX_SET.find_syntax_by_extension("json").unwrap().clone(),
            enabled: false,
        }
    }

    pub fn highlight(&self, content: &str) -> Text<'a> {
        if !self.enabled || content.len() > 100_000 {
            return Text::from(content[0..99_000].to_string());
        }

        let mut h = HighlightLines::new(&self.syntax, &self.theme);
        let mut payload_lines = vec![];
        for line in LinesWithEndings::from(content) {
            let regions = h.highlight_line(line, &SYNTAX_SET).unwrap();
            payload_lines.push(Self::to_line(regions));
        }
        Text::from(payload_lines)
    }

    pub fn highlight_data_type(&self, value: &DataType) -> Text<'a> {
        self.highlight(&value.to_string_pretty())
    }

    fn to_line(regions: Vec<(highlighting::Style, &str)>) -> Line<'static> {
        let spans: Vec<_> = regions
            .into_iter()
            .map(|(style, s)| {
                let mut modifier = ratatui::style::Modifier::empty();
                if style.font_style.contains(highlighting::FontStyle::BOLD) {
                    modifier |= ratatui::style::Modifier::BOLD;
                }
                if style.font_style.contains(highlighting::FontStyle::ITALIC) {
                    modifier |= ratatui::style::Modifier::ITALIC;
                }
                if style
                    .font_style
                    .contains(highlighting::FontStyle::UNDERLINE)
                {
                    modifier |= ratatui::style::Modifier::UNDERLINED;
                }

                Span {
                    content: s.to_string().into(),
                    style: ratatui::style::Style {
                        fg: Self::to_ansi_color(style.foreground),
                        add_modifier: modifier,
                        ..Default::default()
                    },
                }
            })
            .collect();

        Line::from(spans)
    }

    // Copy from https://github.com/sharkdp/bat/blob/master/src/terminal.rs
    fn to_ansi_color(color: highlighting::Color) -> Option<ratatui::style::Color> {
        if color.a == 0 {
            // Themes can specify one of the user-configurable terminal colors by
            // encoding them as #RRGGBBAA with AA set to 00 (transparent) and RR set
            // to the 8-bit color palette number. The built-in themes ansi, base16,
            // and base16-256 use this.
            Some(match color.r {
                // For the first 8 colors, use the Color enum to produce ANSI escape
                // sequences using codes 30-37 (foreground) and 40-47 (background).
                // For example, red foreground is \x1b[31m. This works on terminals
                // without 256-color support.
                0x00 => ratatui::style::Color::Black,
                0x01 => ratatui::style::Color::Red,
                0x02 => ratatui::style::Color::Green,
                0x03 => ratatui::style::Color::Yellow,
                0x04 => ratatui::style::Color::Blue,
                0x05 => ratatui::style::Color::Magenta,
                0x06 => ratatui::style::Color::Cyan,
                0x07 => ratatui::style::Color::White,
                // For all other colors, use Fixed to produce escape sequences using
                // codes 38;5 (foreground) and 48;5 (background). For example,
                // bright red foreground is \x1b[38;5;9m. This only works on
                // terminals with 256-color support.
                //
                // TODO: When ansi_term adds support for bright variants using codes
                // 90-97 (foreground) and 100-107 (background), we should use those
                // for values 0x08 to 0x0f and only use Fixed for 0x10 to 0xff.
                n => ratatui::style::Color::Indexed(n),
            })
        } else if color.a == 1 {
            // Themes can specify the terminal's default foreground/background color
            // (i.e. no escape sequence) using the encoding #RRGGBBAA with AA set to
            // 01. The built-in theme ansi uses this.
            None
        } else {
            Some(ratatui::style::Color::Rgb(color.r, color.g, color.b))
        }
    }
}

#[cfg(test)]
pub mod mod_test;
