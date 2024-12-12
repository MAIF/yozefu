//! This component renders the search bar.
//! It comes with the following features:
//!  - all queries are stored into a history.
//!  - The component suggests queries based on your history.

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Padding, Paragraph, Wrap},
    Frame,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    error::TuiError,
    schema_detail::{ExportedSchemasDetails, SchemaDetail},
    Action,
};

use super::{Component, ComponentName, Shortcut, State};

#[derive(Default)]
pub struct SchemasComponent<'a> {
    key: Option<SchemaDetail>,
    value: Option<SchemaDetail>,
    text: Paragraph<'a>,
    pub action_tx: Option<UnboundedSender<Action>>,
}

impl SchemasComponent<'_> {
    pub fn new() -> Self {
        Self::default()
    }

    fn compute_schemas_rendering(&mut self) {
        let mut to_render = vec![];

        if let Some(s) = &self.key {
            to_render.push(Line::from(vec![
                Span::styled("Key schema URL:   ", Style::default().bold()),
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
                    .unwrap_or("Schema is unavailable".to_string()),
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
                    .unwrap_or("Schema is unavailable".to_string()),
            );
            to_render.push(Line::from(vec![Span::styled(
                "Value schema: ",
                Style::default().bold(),
            )]));
            to_render.extend(schema.lines);
        }

        let p = Paragraph::new(to_render).wrap(Wrap { trim: false });
        self.text = p;
    }
}

impl Component for SchemasComponent<'_> {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<(), TuiError> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn id(&self) -> ComponentName {
        ComponentName::Schemas
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>, TuiError> {
        if let Action::Schemas(key, value) = action {
            self.key = key;
            self.value = value;
            self.compute_schemas_rendering();
        };
        Ok(None)
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>, TuiError> {
        if key.code == KeyCode::Char('c') {
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
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect, state: &State) -> Result<(), TuiError> {
        //let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        //    .begin_symbol(Some("▲"))
        //    .end_symbol(Some("▼"));

        f.render_widget(Clear, rect);
        let block = Block::new()
            .borders(Borders::ALL)
            .padding(Padding::symmetric(4, 0))
            .title(" Schemas ");
        let block = self.make_block_focused_with_state(state, block);

        f.render_widget(self.text.clone().block(block), rect);

        Ok(())
    }

    fn shortcuts(&self) -> Vec<Shortcut> {
        vec![Shortcut::new("C", "Copy")]
    }
}
