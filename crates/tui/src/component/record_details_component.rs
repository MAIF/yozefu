//! Component showing all the details of a given kafka record.
use core::time;

use app::configuration::TimestampFormat;
use bytesize::ByteSize;
use chrono::{DateTime, Utc};
use crossterm::event::{KeyCode, KeyEvent};

use itertools::Itertools;
use lib::{ExportedKafkaRecord, KafkaRecord};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Padding, Paragraph, Wrap},
};
use tokio::sync::mpsc::UnboundedSender;

use super::{Component, ComponentName, Shortcut, State, scroll_state::ScrollState, styles};
use crate::{Action, Theme, error::TuiError, highlighter::Highlighter};

#[derive(Default)]
pub(crate) struct RecordDetailsComponent<'a> {
    record: Option<KafkaRecord>,
    lines: Vec<Line<'a>>,
    search_query: String,
    scroll: ScrollState,
    theme: Option<Theme>,
    action_tx: Option<UnboundedSender<Action>>,
    highlighter: Highlighter,
}

impl<'a> RecordDetailsComponent<'a> {
    pub fn new(highlighter: Highlighter) -> Self {
        Self {
            highlighter,
            ..Default::default()
        }
    }

    fn generate_span(key: &str, value: Line<'a>) -> Line<'a> {
        let mut spans = vec![Span::styled(
            format!("{:>12}: ", key.to_string()),
            Style::default().bold(),
        )];
        spans.extend(value.spans);

        Line::from(spans)
    }

    fn show_schema(&mut self) -> Result<(), TuiError> {
        if self.record.as_ref().is_some_and(|r| !r.has_schemas()) {
            return Ok(());
        }

        let r = self.record.as_ref().unwrap();

        self.action_tx
            .as_ref()
            .unwrap()
            .send(Action::RequestSchemasOf(
                r.key_schema.as_ref().map(|s| s.id.clone()),
                r.value_schema.as_ref().map(|s| s.id.clone()),
            ))?;

        self.action_tx
            .as_ref()
            .unwrap()
            .send(Action::NewView(ComponentName::Schemas))?;
        Ok(())
    }

    fn compute_record_rendering(&mut self) {
        if self.record.is_none() {
            self.record = Some(KafkaRecord::default());
        }

        let theme = self.theme.clone().unwrap_or(Theme::light());
        let record: &KafkaRecord = self.record.as_ref().unwrap();
        let ago_formatter = timeago::Formatter::new();
        let timestamp_in_millis = record.timestamp.unwrap_or(0);

        let published_at = DateTime::from_timestamp_millis(timestamp_in_millis).unwrap();
        let duration = (Utc::now() - published_at)
            .to_std()
            .unwrap_or(time::Duration::ZERO);

        let mut to_render = vec![
            Line::default(),
            Self::generate_span("Topic", record.topic.clone().into()),
            Self::generate_span("Timestamp", format!("{timestamp_in_millis} ms").into()),
            Self::generate_span(
                "DateTime",
                styles::colorize_timestamp(record, &theme, &TimestampFormat::DateTime),
            ),
            Self::generate_span("Published", ago_formatter.convert(duration).into()),
            Self::generate_span("Offset", record.offset.to_string().into()),
            Self::generate_span(
                "Partition",
                record.partition.to_string().fg(theme.yellow).into(),
            ),
            Self::generate_span("Size", ByteSize(record.size as u64).to_string().into()),
            //Self::generate_span("Headers", "".to_string().into()),
        ];

        let longest_header_key = self
            .record
            .as_ref()
            .unwrap()
            .headers
            .keys()
            .map(String::len)
            .max()
            .unwrap_or(0);

        let mut formatted_headers = vec![];
        for entry in self
            .record
            .as_ref()
            .unwrap()
            .headers
            .iter()
            .sorted_by(|a, b| a.0.cmp(b.0))
            .enumerate()
        {
            let e = entry.1;
            match entry.0 {
                0 => formatted_headers.push(Span::styled(
                    format!("{: <width$}", e.0, width = longest_header_key),
                    Style::default().italic(),
                )),
                _ => formatted_headers.push(Span::styled(
                    format!("              {: <width$}", e.0, width = longest_header_key),
                    Style::default().italic(),
                )),
            }
            formatted_headers.push(Span::styled(" : ", Style::default()));
            formatted_headers.push(Span::styled(e.1.to_string(), Style::default()));
        }

        if !formatted_headers.is_empty() {
            let first = formatted_headers.iter().take(3).collect_vec();
            let line: &mut Line<'_> = to_render.last_mut().unwrap();
            line.spans.remove(1);
            for span in first {
                line.push_span(span.clone());
            }
        }

        for ppp in &formatted_headers.into_iter().skip(3).chunks(3) {
            to_render.push(Line::from(ppp.collect_vec()));
        }

        if let Some(s) = &record.key_schema {
            match &s.schema_type {
                Some(t) => to_render.push(Self::generate_span(
                    "Key schema",
                    format!("{} - {}", s.id, t).into(),
                )),
                None => to_render.push(Self::generate_span("Key schema", s.id.to_string().into())),
            }
        }
        if let Some(s) = &record.value_schema {
            match &s.schema_type {
                Some(t) => to_render.push(Self::generate_span(
                    "Value schema",
                    format!("{} - {}", s.id, t).into(),
                )),
                None => {
                    to_render.push(Self::generate_span("Value schema", s.id.to_string().into()));
                }
            }
        }

        to_render.extend(vec![
            Self::generate_span("Key", record.key_as_string.clone().fg(theme.green).into()),
            Self::generate_span("Value", "".into()),
        ]);
        let value = &record.value;
        let highlighted = self.highlighter.highlight_data_type(value);
        to_render.extend(highlighted.lines);
        self.lines = to_render;

        self.scroll.reset();
    }
}

impl Component for RecordDetailsComponent<'_> {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) {
        self.action_tx = Some(tx);
    }

    fn id(&self) -> ComponentName {
        ComponentName::RecordDetails
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>, TuiError> {
        match key.code {
            KeyCode::Char('j') => {
                self.scroll.scroll_to_next_line();
            }
            KeyCode::Char('k') => {
                self.scroll.scroll_to_previous_line();
            }
            KeyCode::Char('[') => {
                self.scroll.scroll_to_top();
            }
            KeyCode::Char(']') => {
                self.scroll.scroll_to_bottom();
            }
            KeyCode::Char('o') => {
                if let Some(record) = &self.record {
                    self.action_tx
                        .as_ref()
                        .unwrap()
                        .send(Action::Open(record.clone()))?;
                }
            }
            KeyCode::Char('s') => self.show_schema()?,
            KeyCode::Char('c') => {
                if let Some(record) = &self.record {
                    let mut exported_record: ExportedKafkaRecord = record.into();
                    exported_record.search_query = self.search_query.to_string();
                    self.action_tx
                        .as_ref()
                        .unwrap()
                        .send(Action::CopyToClipboard(
                            serde_json::to_string_pretty(&exported_record)
                                .expect("Unable to serialize record as json for the clipboard"),
                        ))?;
                }
            }
            KeyCode::Char('e') => {
                if let Some(record) = &self.record {
                    self.action_tx
                        .as_ref()
                        .unwrap()
                        .send(Action::Export(record.clone()))?;
                }
            }
            _ => (),
        }
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>, TuiError> {
        match action {
            Action::ShowRecord(record) => {
                self.record = Some(record);
                self.compute_record_rendering();
            }
            Action::Search(e) => self.search_query = e.query().to_string(),
            _ => {}
        }
        Ok(None)
    }

    fn shortcuts(&self) -> Vec<Shortcut> {
        let mut shortcuts = vec![
            Shortcut::new("J/K", "Scroll"),
            Shortcut::new("↑↓", "Prev/next record"),
        ];

        if self
            .record
            .as_ref()
            .is_some_and(|r| r.key_schema.is_some() || r.value_schema.is_some())
        {
            shortcuts.push(Shortcut::new("S", "Schemas"));
        }

        shortcuts
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect, state: &State) -> Result<(), TuiError> {
        if self.theme.is_none() {
            self.theme = Some(state.theme.clone());
        }
        let p = Paragraph::new(self.lines.clone())
            .wrap(Wrap { trim: false })
            .scroll((self.scroll.value(), 0));

        f.render_widget(Clear, rect);
        let block = Block::new()
            .borders(Borders::ALL)
            .padding(Padding::symmetric(4, 0))
            .title(" Record details ");
        let block = self.make_block_focused_with_state(state, block);

        f.render_widget(p.block(block), rect);
        self.scroll.draw(f, rect, self.lines.len() + 2);
        Ok(())
    }
}
