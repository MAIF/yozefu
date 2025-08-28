//! Theme for the TUI

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// A `Theme` contains all the colors
/// to make the TUI pretty.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub fg: Color,
    pub bg: Color,
    pub black: Color,
    pub red: Color,
    pub green: Color,
    pub yellow: Color,
    pub blue: Color,
    pub magenta: Color,
    pub cyan: Color,
    pub white: Color,
    pub orange: Color,
    pub focused_border: Color,
    pub bg_focused_selected: Color,
    pub fg_focused_selected: Color,
    pub bg_unfocused_selected: Color,
    pub fg_unfocused_selected: Color,
    pub bg_disabled: Color,
    pub fg_disabled: Color,
    pub bg_active: Color,
    pub fg_active: Color,
    pub dialog_border: Color,
    pub autocomplete: Color,
    pub shortcuts: Option<Color>,
    pub highlighter_theme: Option<String>,
}

impl Theme {
    pub fn light() -> Self {
        Self {
            name: "light".to_string(),
            fg: Color::Black,
            bg: Color::White,
            black: Color::Black,
            red: Color::Red,
            green: Color::Green,
            yellow: Color::Yellow,
            blue: Color::Blue,
            magenta: Color::Magenta,
            cyan: Color::Cyan,
            white: Color::White,
            orange: Color::LightRed,
            focused_border: Color::Blue,
            bg_focused_selected: Color::Black,
            fg_focused_selected: Color::White,
            bg_unfocused_selected: Color::White,
            fg_unfocused_selected: Color::default(),
            dialog_border: Color::Yellow,
            autocomplete: Color::Rgb(100, 100, 100),
            bg_disabled: Color::Indexed(7),
            fg_disabled: Color::Black,
            bg_active: Color::Green,
            fg_active: Color::Black,
            shortcuts: Some(Color::DarkGray),
            highlighter_theme: Some("base16-ocean.light".to_string()),
        }
    }
}
