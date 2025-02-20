//! Component showing in real time incoming kafka records.
use app::search::ValidSearchQuery;
use copypasta::{ClipboardContext, ClipboardProvider};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use itertools::Itertools;
use lib::ExportedKafkaRecord;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Rect},
    style::{Style, Stylize},
    text::{Span, Text},
    widgets::{Block, BorderType, Borders, Cell, Row, Table, TableState},
};
use thousands::Separable;
use throbber_widgets_tui::ThrobberState;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::watch::Receiver;

use crate::{Action, action::Notification, error::TuiError, records_buffer::BufferAction};

use super::{Component, ComponentName, ConcurrentRecordsBuffer, Shortcut, State};

pub(crate) struct RecordsComponent<'a> {
    records: &'a ConcurrentRecordsBuffer,
    state: TableState,
    status: ThrobberState,
    search_query: ValidSearchQuery,
    consuming: bool,
    count: (usize, usize, usize),
    follow: bool,
    action_tx: Option<UnboundedSender<Action>>,
    buffer_tx: Receiver<BufferAction>,
    selected_topics: usize,
    key_events_buffer: Vec<KeyEvent>,
}

impl<'a> RecordsComponent<'a> {
    pub fn new(records: &'a ConcurrentRecordsBuffer) -> Self {
        let buffer_tx = records.lock().map(|e| e.channels.clone().1).ok().unwrap();

        Self {
            records,
            state: Default::default(),
            status: Default::default(),
            search_query: Default::default(),
            consuming: Default::default(),
            count: Default::default(),
            follow: Default::default(),
            action_tx: Default::default(),
            buffer_tx,
            selected_topics: Default::default(),
            key_events_buffer: Default::default(),
        }
    }

    fn buffer_is_empty(&self) -> bool {
        self.count.2 == 0
    }

    fn buffer_len(&self) -> usize {
        self.count.2
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

    fn shorten_topic(topic: &str) -> String {
        let t = topic.replace('_', "");
        let parts = t.split('.');
        let e = parts.map(|e| e.chars().next().unwrap_or('_')).join(".");
        e
    }

    fn show_details(&mut self) -> Result<(), TuiError> {
        if self.state.selected().is_some() {
            self.action_tx
                .as_ref()
                .unwrap()
                .send(Action::NewView(ComponentName::RecordDetails))?;
            self.set_event_dialog()?
        }
        Ok(())
    }

    fn set_event_dialog(&mut self) -> Result<(), TuiError> {
        if let Some(s) = self.state.selected() {
            let record = self.records.lock().unwrap().get(s).unwrap().clone();
            self.action_tx
                .as_ref()
                .unwrap()
                .send(Action::ShowRecord(record))?;
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

    pub fn on_new_record(&mut self, count: (usize, usize, usize)) -> Result<(), TuiError> {
        self.count = count;
        let length = count.2;
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
        Ok(())
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

    fn truncate_value(value: &str, rect: &Rect) -> String {
        let split_at = (rect.width.checked_sub(70)).unwrap_or(3) as usize;
        match value.len() > split_at {
            true => value.chars().take(split_at).collect(),
            false => value.to_string(),
        }
    }
}

impl Component for RecordsComponent<'_> {
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
                    let r = self.records.lock().unwrap();
                    let record = r.get(s).unwrap();
                    let mut ctx = ClipboardContext::new().unwrap();
                    let exported_record: ExportedKafkaRecord = record.into();
                    self.action_tx.as_ref().unwrap().send(Action::Notification(
                        Notification::new(log::Level::Info, "Copied to clipboard".to_string()),
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
                let records = self.records.lock().unwrap();
                for record in records.iter() {
                    self.action_tx
                        .as_ref()
                        .unwrap()
                        .send(Action::Export(record.clone()))?;
                }
            }
            KeyCode::Char('e') => {
                if let Some(s) = self.state.selected() {
                    let r = self.records.lock().unwrap();
                    let record = r.get(s).unwrap();
                    self.action_tx
                        .as_ref()
                        .unwrap()
                        .send(Action::Export(record.clone()))?;
                }
            }
            KeyCode::Char('o') => {
                if let Some(s) = self.state.selected() {
                    let r = self.records.lock().unwrap();
                    let record = r.get(s).unwrap();
                    self.action_tx
                        .as_ref()
                        .unwrap()
                        .send(Action::Open(record.clone()))?;
                }
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
            KeyCode::Char('g') | KeyCode::Char('G') => self.buffer_key_event(key)?,
            _ => (),
        }
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>, TuiError> {
        let mut a = self.buffer_tx.clone();
        let BufferAction::Count(count) = *a.borrow_and_update();
        let _ = self.on_new_record(count);
        match action.clone() {
            Action::NewConsumer() => {
                self.count = (0, 0, 0);
            }
            Action::Tick => self.status.calc_next(),
            Action::SelectedTopics(topics) => self.selected_topics = topics.len(),
            Action::Consuming => self.consuming = true,
            Action::StopConsuming() => {
                self.consuming = false;
                self.count = (0, 0, 0);
            }
            Action::Search(search_query) => {
                self.state.select(None);
                self.search_query = search_query;
            }
            _ => (),
        };
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect, state: &State) -> Result<(), TuiError> {
        let focused = state.is_focused(self.id());
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Records ");

        let block = self.make_block_focused_with_state(state, block);

        let normal_style = Style::default();
        let header_cells = vec![
            Cell::new(Text::from("Timestamp")),
            Cell::new(Text::from("Offset").alignment(Alignment::Right)),
            Cell::new(Text::from("Partition").alignment(Alignment::Right)),
            Cell::new(Text::from("Topic").alignment(Alignment::Right)),
            Cell::new(Text::from("Key").alignment(Alignment::Right)),
            Cell::new(Text::from("Value")),
        ];
        let header = Row::new(header_cells)
            .style(normal_style)
            .height(1)
            .bottom_margin(1);
        let r = self.records.lock().unwrap();

        // TODO render only records in the viewport
        let rows = r.iter().enumerate().map(|(index, item)| {
            if let Some(s) = self.state.selected() {
                let is_visible = (s + rect.height as usize) > index
                    && s.saturating_sub(rect.height as usize) <= index;
                if !is_visible {
                    return Row::new(Vec::<Cell>::new()).height(1_u16);
                }
            }
            let cells = vec![
                Cell::new(Text::from(
                    item.timestamp_as_local_date_time()
                        .map(|e| e.to_rfc3339_opts(chrono::SecondsFormat::Millis, false))
                        .unwrap_or("".to_string()),
                )),
                Cell::new(Text::from(item.offset.to_string()).alignment(Alignment::Right)),
                Cell::new(Text::from(item.partition.to_string()).alignment(Alignment::Right)),
                Cell::new(Text::from(Self::shorten_topic(&item.topic)).alignment(Alignment::Right)),
                Cell::new(Text::from(item.key_as_string.to_string()).alignment(Alignment::Right)),
                Cell::new(Text::from(Self::truncate_value(
                    &item.value_as_string,
                    &rect,
                ))),
            ];
            Row::new(cells).height(1_u16)
        });
        let table = Table::new(
            rows,
            [
                Constraint::Min(30),
                Constraint::Min(10),
                Constraint::Min(10),
                Constraint::Min(11),
                Constraint::Min(12),
                Constraint::Percentage(100),
            ],
        )
        .header(header)
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
                " {} / {} ",
                self.count.0.separate_with_underscores(),
                self.count.1.separate_with_underscores()
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
                .checked_sub(10)
                .unwrap_or(1000),
            inner.y,
            metrics.width() as u16,
            1,
        );

        if self.consuming && self.count.1 != 0 {
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
            //Shortcut::new("↑↓", "Scroll"),
        ];

        shortcuts
    }
}
