//! This library contains all the glue code with [Ratatui](https://github.com/ratatui/ratatui).

mod action;
mod component;
pub mod error;
mod records_buffer;
mod schema_detail;
pub mod theme;
mod tui;
pub use action::Action;
pub use action::Notification;

pub use component::State;
pub use component::Ui;
pub use error::TuiError;
pub use theme::Theme;
