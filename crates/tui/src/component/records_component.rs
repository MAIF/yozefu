//! Component showing in real time incoming kafka records.

use app::{configuration::TimestampFormat, search::ValidSearchQuery};
use copypasta::{ClipboardContext, ClipboardProvider};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lib::ExportedKafkaRecord;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Span, Text},
    widgets::{Block, BorderType, Borders, Cell, Row, Table, TableState},
};
use thousands::Separable;
use throbber_widgets_tui::ThrobberState;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    Action,
    action::{Level, Notification},
    component::RecordsReceiver,
    error::TuiError,
    records_buffer::{BUFFER_SIZE, RecordsBuffer},
};

use super::{Component, ComponentName, Shortcut, State, styles};

pub(crate) struct RecordsComponent {
    records: RecordsBuffer,
    state: TableState,
    status: ThrobberState,
    search_query: ValidSearchQuery,
    consuming: bool,
    receiver: RecordsReceiver,
    follow: bool,
    action_tx: Option<UnboundedSender<Action>>,
    selected_topics: usize,
    key_events_buffer: Vec<KeyEvent>,
    column_size: u16,
    timestamp_format: TimestampFormat,
}

impl RecordsComponent {
    pub fn new(receiver: RecordsReceiver, timestamp_format: TimestampFormat) -> Self {
        Self {
            records: RecordsBuffer::new(),
            receiver,
            state: TableState::default(),
            status: ThrobberState::default(),
            search_query: ValidSearchQuery::default(),
            consuming: false,
            follow: false,
            action_tx: None,
            selected_topics: 0,
            key_events_buffer: Vec::default(),
            column_size: 0,
            timestamp_format,
        }
    }

    fn buffer_is_empty(&self) -> bool {
        self.records.is_empty()
    }

    fn buffer_len(&self) -> usize {
        self.records.len()
    }

    pub fn poll_new_records(&mut self) {
        for _i in 0..500 {
            match self.receiver.try_recv() {
                Err(_) => break,
                Ok(record) => {
                    self.records.extend(record);
                    self.on_new_record();
                }
            }
        }

        self.records.sort(&self.search_query.query().order_by);
    }

    fn next(&mut self) {
        if self.buffer_is_empty() {
            self.state.select(None);
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.buffer_len() - 1 {
                    i
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn show_details(&mut self) -> Result<(), TuiError> {
        if self.state.selected().is_some() {
            self.action_tx
                .as_ref()
                .unwrap()
                .send(Action::NewView(ComponentName::RecordDetails))?;
            self.set_event_dialog()?;
        }
        Ok(())
    }

    fn set_event_dialog(&mut self) -> Result<(), TuiError> {
        if let Some(s) = self.state.selected() {
            if let Some(record) = self.records.get(s) {
                self.action_tx
                    .as_ref()
                    .unwrap()
                    .send(Action::ShowRecord(record.clone()))?;
            };
        }
        Ok(())
    }

    fn previous(&mut self) {
        if self.buffer_is_empty() {
            self.state.select(None);
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn first(&mut self) {
        match self.buffer_is_empty() {
            true => self.state.select(None),
            false => self.state.select(Some(0)),
        }
    }

    fn last(&mut self) {
        match self.buffer_is_empty() {
            true => self.state.select(None),
            false => self.state.select(Some(self.buffer_len() - 1)),
        }
    }

    pub fn on_new_record(&mut self) {
        let stats = self.records.stats();
        let length = stats.buffer_size;
        let empty_buffer = length == 0;
        if self.follow && !empty_buffer {
            self.state.select(Some(length - 1));
        }
        if self.state.selected().is_none() && !empty_buffer {
            self.state.select(Some(0));
        }
        if let Some(s) = self.state.selected() {
            if s >= length {
                let ii = match empty_buffer {
                    true => 0,
                    false => length - 1,
                };
                self.state.select(Some(ii));
            }
        }
    }

    fn buffer_key_event(&mut self, e: KeyEvent) -> Result<(), TuiError> {
        self.key_events_buffer.push(e);
        if self.key_events_buffer.len() < 2 {
            return Ok(());
        }
        self.key_events_buffer.dedup();
        if self.key_events_buffer.len() == 1 {
            match self.key_events_buffer.first().unwrap().code {
                KeyCode::Char('g') => {
                    self.follow(false)?;
                    self.first();
                }
                KeyCode::Char('G') => {
                    self.follow(false)?;
                    self.last();
                }
                _ => (),
            }
        }
        self.key_events_buffer.clear();
        Ok(())
    }

    fn follow(&mut self, follow: bool) -> Result<(), TuiError> {
        self.follow = follow;
        if self.follow {
            self.state.select(match self.buffer_len() {
                0 => None,
                i => Some(i - 1),
            });
        }
        self.action_tx
            .as_ref()
            .unwrap()
            .send(Action::RefreshShortcuts)?;
        Ok(())
    }

    fn truncate_value(value: &str, width: usize) -> String {
        match value.len() > width {
            true => value.chars().take(width).collect(),
            false => value.to_string(),
        }
    }
}

impl Component for RecordsComponent {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) {
        self.action_tx = Some(tx.clone());
    }

    fn id(&self) -> ComponentName {
        ComponentName::Records
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>, TuiError> {
        match key.code {
            KeyCode::Char('c') => {
                if let Some(s) = self.state.selected() {
                    let record = self.records.get(s).unwrap();
                    let mut ctx = ClipboardContext::new().unwrap();
                    let exported_record: ExportedKafkaRecord = record.into();
                    self.action_tx.as_ref().unwrap().send(Action::Notification(
                        Notification::new(Level::Info, "Copied to clipboard".to_string()),
                    ))?;
                    ctx.set_contents(serde_json::to_string_pretty(&exported_record)?)
                        .unwrap();
                }
            }
            KeyCode::Char('f') => self.follow(!self.follow)?,
            KeyCode::Char('v') | KeyCode::Enter => {
                self.show_details()?;
            }
            KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                for record in self.records.iter() {
                    self.action_tx
                        .as_ref()
                        .unwrap()
                        .send(Action::Export(record.clone()))?;
                }
            }
            KeyCode::Char('e') => {
                if let Some(s) = self.state.selected() {
                    let record = self.records.get(s).unwrap();
                    self.action_tx
                        .as_ref()
                        .unwrap()
                        .send(Action::Export(record.clone()))?;
                }
            }
            KeyCode::Char('o') => {
                if let Some(s) = self.state.selected() {
                    let record = self.records.get(s).unwrap();
                    self.action_tx
                        .as_ref()
                        .unwrap()
                        .send(Action::Open(record.clone()))?;
                }
            }
            KeyCode::Char('t') => {
                self.timestamp_format = match self.timestamp_format {
                    TimestampFormat::Ago => TimestampFormat::DateTime,
                    TimestampFormat::DateTime => TimestampFormat::Ago,
                };
            }
            KeyCode::Char('[') => {
                self.follow(false)?;
                self.first();
            }
            KeyCode::Char(']') => {
                self.follow(false)?;
                self.last();
            }
            KeyCode::Down => {
                self.follow(false)?;
                self.next();
                self.set_event_dialog()?;
            }
            KeyCode::Up => {
                self.follow(false)?;
                self.previous();
                self.set_event_dialog()?;
            }
            KeyCode::Left => {
                self.column_size = self.column_size.saturating_sub(1);
            }
            KeyCode::Right => {
                self.column_size = self.column_size.saturating_add(1).min(100);
            }
            KeyCode::Char('g' | 'G') => self.buffer_key_event(key)?,
            _ => (),
        }
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>, TuiError> {
        match action.clone() {
            Action::Tick => self.status.calc_next(),
            Action::SelectedTopics(topics) => self.selected_topics = topics.len(),
            Action::Consuming => {
                self.consuming = true;
                self.records.reset();
            }
            Action::StopConsuming() => {
                self.consuming = false;
                self.records.reset();
            }
            Action::Search(search_query) => {
                self.state.select(None);
                self.search_query = search_query;
            }
            _ => (),
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect, state: &State) -> Result<(), TuiError> {
        self.poll_new_records();
        let focused = state.is_focused(&self.id());
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Records ");

        let block = self.make_block_focused_with_state(state, block);

        let constraints = [
            Constraint::Length(match self.timestamp_format {
                TimestampFormat::Ago => 15,
                TimestampFormat::DateTime => 29,
            }),
            Constraint::Min(19 + self.column_size),
            Constraint::Length(7),
            Constraint::Length(10 + self.column_size),
            Constraint::Percentage(100),
        ];

        let lll = Layout::horizontal(constraints).split(rect);

        let normal_style = Style::default();
        let header_cells = vec![
            Cell::new(Text::from("Timestamp")).bold(),
            Cell::new(Text::from("Topic").alignment(Alignment::Right)).bold(),
            Cell::new(Text::from("Offset").alignment(Alignment::Right)).bold(),
            Cell::new(Text::from("Key").alignment(Alignment::Right)).bold(),
            Cell::new(Text::from("Value")).bold(),
        ];
        let header = Row::new(header_cells)
            .style(normal_style)
            .height(1)
            .bottom_margin(1);

        let mut records = Vec::with_capacity(BUFFER_SIZE);
        records.extend(self.records.iter().cloned());

        // TODO render only records in the viewport
        let rows = records.iter().enumerate().map(|(index, item)| {
            if let Some(s) = self.state.selected() {
                let is_visible = (s + rect.height as usize) > index
                    && s.saturating_sub(rect.height as usize) <= index;
                if !is_visible {
                    return Row::new(Vec::<Cell>::new()).height(1_u16);
                }
            }

            let cells = vec![
                Cell::new(
                    styles::colorize_timestamp(item, &state.theme, &self.timestamp_format)
                        .alignment(match self.timestamp_format {
                            TimestampFormat::Ago => Alignment::Right,
                            TimestampFormat::DateTime => Alignment::Left,
                        }),
                ),
                Cell::new(
                    Text::from(styles::colorize_and_shorten_topic(
                        &item.topic,
                        item.partition,
                        &state.theme,
                        lll[1].width as usize,
                    ))
                    .alignment(Alignment::Right),
                ),
                Cell::new(Text::from(item.offset.to_string()).alignment(Alignment::Right)),
                Cell::new(
                    styles::colorize_key(&item.key_as_string, &state.theme, lll[3].width as usize)
                        .alignment(Alignment::Right),
                ),
                Cell::new(Text::from(Self::truncate_value(
                    &item.value_as_string,
                    lll[4].width as usize,
                ))),
            ];
            Row::new(cells).height(1_u16)
        });

        let table = Table::new(rows, constraints)
            .header(header)
            .column_spacing(2)
            .row_highlight_style(match focused {
                true => Style::default()
                    .bg(state.theme.bg_focused_selected)
                    .fg(state.theme.fg_focused_selected)
                    .bold(),
                false => Style::default()
                    .bg(state.theme.bg_unfocused_selected)
                    .fg(state.theme.fg_unfocused_selected),
            });

        let metrics = Span::styled(
            format!(
                " {} / {}",
                self.records.stats().matched.separate_with_underscores(),
                self.records.stats().read.separate_with_underscores(),
                //self.stats.read
                //    .checked_div(self.stats.read)
                //    .unwrap_or(0)
                //    .checked_mul(100)
                //    .unwrap_or(0)
                //    .checked_div(self.stats.total_to_read)
                //    .unwrap_or(0),
            ),
            Style::default(),
        );
        let inner = block.inner(rect);
        f.render_widget(block, rect);

        f.render_stateful_widget(table, inner, &mut self.state);
        let metrics_area = Rect::new(
            inner
                .right()
                .checked_sub(u16::try_from(metrics.width())?)
                .unwrap_or(1000)
                .checked_sub(11)
                .unwrap_or(1000),
            inner.y,
            metrics.width() as u16,
            1,
        );

        if self.records.stats().read != 0 {
            f.render_widget(metrics, metrics_area);
        }
        if self.consuming {
            let simple = throbber_widgets_tui::Throbber::default();
            let ss = Span::styled(
                " Live    ",
                Style::default()
                    .fg(state.theme.white)
                    .bg(state.theme.orange),
            )
            .bold();
            f.render_widget(
                ss,
                Rect::new(inner.right().saturating_sub(9), inner.y, 8, 1),
            );
            f.render_stateful_widget(
                simple,
                Rect::new(inner.right().saturating_sub(3), inner.y, 2, 1),
                &mut self.status,
            );
        }
        Ok(())
    }

    fn shortcuts(&self) -> Vec<Shortcut> {
        let shortcuts = vec![
            Shortcut::new("C", "Copy"),
            Shortcut::new("O", "Open"),
            // Shortcut::new("[", "First record"),
            // Shortcut::new("]", "Last record"),
            Shortcut::new("E", "Export"),
            Shortcut::new(
                "F",
                match self.follow {
                    true => "Unfollow",
                    false => "Follow",
                },
            ),
            Shortcut::new("T", "Timestamp format"),
            Shortcut::new("⇄", "Resize"),
        ];

        shortcuts
    }
}
