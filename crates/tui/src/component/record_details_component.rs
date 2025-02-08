//! Component showing all the details of a given kafka record.
use bytesize::ByteSize;
use crossterm::event::{KeyCode, KeyEvent};

use itertools::Itertools;
use lib::{ExportedKafkaRecord, KafkaRecord};
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Padding, Paragraph, Wrap},
    Frame,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{error::TuiError, Action};

use super::{scroll_state::ScrollState, Component, ComponentName, Shortcut, State};

#[derive(Default)]
pub(crate) struct RecordDetailsComponent<'a> {
    record: Option<KafkaRecord>,
    lines: Vec<Line<'a>>,
    search_query: String,
    scroll: ScrollState,
    action_tx: Option<UnboundedSender<Action>>,
}

impl RecordDetailsComponent<'_> {
    pub fn new(_state: &State) -> Self {
        Self {
            ..Default::default()
        }
    }

    fn generate_span(key: &str, value: String) -> Line<'_> {
        Line::from(vec![
            Span::styled(
                format!("{:>12}: ", key.to_string()),
                Style::default().bold(),
            ),
            Span::styled(value.to_string(), Style::default()),
        ])
    }

    fn show_schema(&mut self) -> Result<(), TuiError> {
        if self.record.as_ref().map(|r| r.has_schemas()) == Some(false) {
            return Ok(());
        };

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

        let record = self.record.as_ref().unwrap();
        let mut to_render = vec![
            Line::default(),
            Self::generate_span("Topic", record.topic.clone()),
            Self::generate_span(
                "Timestamp",
                self.record
                    .as_ref()
                    .unwrap()
                    .timestamp
                    .unwrap_or(0)
                    .to_string(),
            ),
            Self::generate_span(
                "DateTime",
                self.record
                    .as_ref()
                    .unwrap()
                    .timestamp_as_local_date_time()
                    .map(|e| e.to_rfc3339_opts(chrono::SecondsFormat::Millis, false))
                    .unwrap_or("".to_string()),
            ),
            Self::generate_span("Offset", record.offset.to_string()),
            Self::generate_span("Partition", record.partition.to_string()),
            Self::generate_span("Size", ByteSize(record.size as u64).to_string()),
            Self::generate_span("Headers", "".to_string()),
        ];

        let longest_header_key = self
            .record
            .as_ref()
            .unwrap()
            .headers
            .keys()
            .map(|e| e.len())
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
            };
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
                    format!("{} - {}", s.id, t),
                )),
                None => to_render.push(Self::generate_span("Key schema", s.id.to_string())),
            }
        }
        if let Some(s) = &record.value_schema {
            match &s.schema_type {
                Some(t) => to_render.push(Self::generate_span(
                    "Value schema",
                    format!("{} - {}", s.id, t),
                )),
                None => to_render.push(Self::generate_span("Value schema", s.id.to_string())),
            }
        }

        to_render.extend(vec![
            Self::generate_span("Key", record.key_as_string.clone()),
            Self::generate_span("Value", "".to_string()),
        ]);

        //let syntax = SYNTAX_SET.find_syntax_by_extension("json").unwrap();
        //let mut h = HighlightLines::new(syntax, &self.theme);
        //
        //let mut payload_lines = vec![];
        //for line in LinesWithEndings::from(&value_s) {
        //    let ranges: Vec<(syntect::highlighting::Style, &str)> =
        //        h.highlight_line(line, &SYNTAX_SET).unwrap();
        //    let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
        //    let output = escaped.into_text().unwrap();
        //    payload_lines.extend(output.lines);
        //}
        let text = Text::from(record.value.to_string_pretty());
        to_render.extend(text.lines);

        self.lines = to_render;
        self.scroll.reset();
    }
}

impl Component for RecordDetailsComponent<'_> {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<(), TuiError> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn id(&self) -> ComponentName {
        ComponentName::RecordDetails
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>, TuiError> {
        match key.code {
            KeyCode::Char('k') => {
                self.scroll.scroll_to_next_line();
            }
            KeyCode::Char('j') => {
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
        };
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
            .map(|r| r.key_schema.is_some() || r.value_schema.is_some())
            .unwrap_or(false)
        {
            shortcuts.push(Shortcut::new("S", "Schemas"))
        }

        shortcuts
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect, state: &State) -> Result<(), TuiError> {
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
