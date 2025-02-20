//! This component renders the search bar.
//! It comes with the following features:
//!  - all queries are stored into a history.
//!  - The component suggests queries based on your history.

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Padding, Paragraph, Wrap},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    Action,
    error::TuiError,
    schema_detail::{ExportedSchemasDetails, SchemaDetail},
};

use super::{Component, ComponentName, Shortcut, State, scroll_state::ScrollState};

#[derive(Default)]
pub(crate) struct SchemasComponent<'a> {
    key: Option<SchemaDetail>,
    value: Option<SchemaDetail>,
    lines: Vec<Line<'a>>,
    action_tx: Option<UnboundedSender<Action>>,
    scroll: ScrollState,
}

impl SchemasComponent<'_> {
    pub fn new() -> Self {
        Self::default()
    }

    fn compute_schemas_rendering(&mut self) {
        let mut to_render = vec![];

        if let Some(s) = &self.key {
            to_render.push(Line::from(vec![
                Span::styled("Key schema URL  : ", Style::default().bold()),
                Span::styled(s.url.to_string(), Style::default()),
            ]));
        }
        if let Some(s) = &self.value {
            to_render.push(Line::from(vec![
                Span::styled("Value schema URL: ", Style::default().bold()),
                Span::styled(s.url.to_string(), Style::default()),
            ]));
        }
        if let Some(s) = &self.key {
            to_render.push(Line::default());
            let schema = Text::from(
                s.response
                    .as_ref()
                    .map(|r| r.schema_to_string_pretty())
                    .unwrap_or(
                        format!("The Schema {} is unavailable. Please make sure you configured Yozefu to use the schema registry.", s.id),
                    ),
            );
            to_render.push(Line::from(vec![Span::styled(
                "Key schema: ",
                Style::default().bold(),
            )]));
            to_render.extend(schema.lines);
        }

        if let Some(s) = &self.value {
            to_render.push(Line::default());
            let schema = Text::from(
                s.response
                    .as_ref()
                    .map(|r| r.schema_to_string_pretty())
                    .unwrap_or(
                        format!("The Schema {} is unavailable. Please make sure you configured Yozefu to use the schema registry.", s.id),
                    ),
            );
            to_render.push(Line::from(vec![Span::styled(
                "Value schema: ",
                Style::default().bold(),
            )]));
            to_render.extend(schema.lines);
        }
        self.lines = to_render;
    }
}

impl Component for SchemasComponent<'_> {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) {
        self.action_tx = Some(tx);
    }

    fn id(&self) -> ComponentName {
        ComponentName::Schemas
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>, TuiError> {
        if let Action::Schemas(key, value) = action {
            self.key = key;
            self.value = value;
            self.compute_schemas_rendering();
            self.scroll.reset();
        };
        Ok(None)
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>, TuiError> {
        match key.code {
            KeyCode::Char('k') | KeyCode::Down => {
                self.scroll.scroll_to_next_line();
            }
            KeyCode::Char('j') | KeyCode::Up => {
                self.scroll.scroll_to_previous_line();
            }
            KeyCode::Char('[') => {
                self.scroll.scroll_to_top();
            }
            KeyCode::Char(']') => {
                self.scroll.scroll_to_bottom();
            }
            KeyCode::Char('c') => {
                let exported_schemas = ExportedSchemasDetails {
                    key: self.key.clone(),
                    value: self.value.clone(),
                };
                self.action_tx
                    .as_ref()
                    .unwrap()
                    .send(Action::CopyToClipboard(
                        serde_json::to_string_pretty(&exported_schemas)
                            .expect("Unable to serialize schemas"),
                    ))?;
            }
            _ => (),
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect, state: &State) -> Result<(), TuiError> {
        f.render_widget(Clear, rect);
        let block = Block::new()
            .borders(Borders::ALL)
            .padding(Padding::symmetric(4, 0))
            .title(" Schemas ");

        let paragraph = Paragraph::new(self.lines.clone())
            .wrap(Wrap { trim: false })
            .scroll((self.scroll.value(), 0));

        let block = self.make_block_focused_with_state(state, block);
        f.render_widget(paragraph.block(block), rect);

        self.scroll.draw(f, rect, self.lines.len() + 2);
        Ok(())
    }

    fn shortcuts(&self) -> Vec<Shortcut> {
        vec![Shortcut::new("C", "Copy")]
    }
}
