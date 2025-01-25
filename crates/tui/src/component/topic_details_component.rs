//! Component showing information regarding a given topic: partitions, consumer groups, replicas ...
use std::collections::HashSet;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use itertools::Itertools;
use lib::{ConsumerGroupDetail, ConsumerGroupState, TopicDetail};
use ratatui::{
    layout::{Alignment, Constraint, Margin, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, Padding, Paragraph, Row, Table, TableState,
    },
    Frame,
};
use thousands::Separable;
use tokio::sync::mpsc::UnboundedSender;

use crate::{error::TuiError, Action, Notification};

use super::{Component, ComponentName, State, WithHeight};

#[derive(Default)]
pub struct TopicDetailsComponent {
    pub details: Vec<TopicDetail>,
    pub action_tx: Option<UnboundedSender<Action>>,
    pub state: TableState,
    pub refreshing_data: bool,
    throbber_state: throbber_widgets_tui::ThrobberState,
}

impl WithHeight for TopicDetailsComponent {
    fn content_height(&self) -> usize {
        self.details
            .iter()
            .map(|e| e.consumer_groups.len())
            .sum::<usize>()
    }
}

impl Component for TopicDetailsComponent {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<(), TuiError> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn id(&self) -> ComponentName {
        ComponentName::TopicDetails
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>, TuiError> {
        match key.code {
            KeyCode::Char('k') | KeyCode::Down => {
                self.next();
                //self.scroll.scroll_to_next_line();
            }
            KeyCode::Char('j') | KeyCode::Up => {
                self.previous();
                //self.scroll.scroll_to_previous_line();
            }
            KeyCode::Char('[') => {
                self.first();
            }
            KeyCode::Char(']') => {
                self.last();
            }
            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let mut h = HashSet::default();
                h.extend(self.details.iter().map(|d| d.name.clone()));
                self.refreshing_data = true;
                self.action_tx
                    .as_ref()
                    .unwrap()
                    .send(Action::Notification(Notification::new(
                        log::Level::Info,
                        "Refreshing data".to_string(),
                    )))
                    .unwrap();
                self.action_tx
                    .as_ref()
                    .unwrap()
                    .send(Action::RequestTopicDetails(h))
                    .unwrap();
            }
            _ => (),
        }
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>, TuiError> {
        match action {
            Action::Tick => self.throbber_state.calc_next(),
            Action::TopicDetails(details) => {
                self.refreshing_data = false;
                self.details = details;
            }
            Action::RequestTopicDetails(_details) => {
                if !self.details.is_empty() {
                    self.refreshing_data = true;
                }
            }
            _ => (),
        };
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect, state: &State) -> Result<(), TuiError> {
        let block = Block::new()
            .borders(Borders::ALL)
            .border_style(Style::default())
            .title(" Topic details ")
            .padding(Padding::proportional(3))
            .border_type(BorderType::Rounded);
        let block = self.make_block_focused_with_state(state, block);

        if self.details.is_empty() {
            f.render_widget(Clear, rect);
            let full = throbber_widgets_tui::Throbber::default()
                .label("Fetching data...")
                .style(Style::default())
                .throbber_style(Style::default().add_modifier(Modifier::BOLD))
                .throbber_set(throbber_widgets_tui::BRAILLE_DOUBLE)
                .use_type(throbber_widgets_tui::WhichUse::Spin);
            f.render_widget(block, rect);
            f.render_stateful_widget(
                full,
                rect.inner(Margin::new(6, 4)),
                &mut self.throbber_state,
            );
            return Ok(());
        }

        if self.refreshing_data {
            let full = throbber_widgets_tui::Throbber::default()
                .label("Refreshing data...")
                .style(Style::default())
                .throbber_style(Style::default().add_modifier(Modifier::BOLD))
                .throbber_set(throbber_widgets_tui::BRAILLE_DOUBLE)
                .use_type(throbber_widgets_tui::WhichUse::Spin);
            f.render_widget(&block, rect);
            f.render_stateful_widget(
                full,
                rect.inner(Margin::new(7, 3)),
                &mut self.throbber_state,
            );
        }

        if !self.details.is_empty() {
            let header_cells = vec![
                Cell::new(Text::from("")),
                Cell::new(Text::from("Name")),
                Cell::new(Text::from("State")),
                Cell::new(Text::from("Partitions").alignment(Alignment::Right)),
                Cell::new(Text::from("Members").alignment(Alignment::Right)),
                Cell::new(Text::from("Lag").alignment(Alignment::Right)),
            ];

            let header = Row::new(header_cells).bold().height(1);
            let mut rows = vec![];

            for detail in &self.details {
                let consumers_groups = detail.consumer_groups.clone();
                rows.extend(
                    consumers_groups
                        .into_iter()
                        .sorted_by(|a, b| a.name.cmp(&b.name))
                        .enumerate()
                        .map(|item| {
                            Row::new(vec![
                                Cell::new(
                                    match item.1.state {
                                        ConsumerGroupState::Unknown => {
                                            Span::styled("⊘", Style::default().fg(state.theme.red))
                                        }
                                        ConsumerGroupState::Empty => {
                                            Span::styled("◯", Style::default().fg(state.theme.red))
                                        }
                                        ConsumerGroupState::Dead => {
                                            Span::styled("⊗", Style::default().fg(state.theme.red))
                                        }
                                        ConsumerGroupState::Stable => Span::styled(
                                            "⏺︎",
                                            Style::default().fg(state.theme.green),
                                        ),
                                        ConsumerGroupState::PreparingRebalance => Span::styled(
                                            "⦿",
                                            Style::default().fg(state.theme.yellow),
                                        ),
                                        ConsumerGroupState::CompletingRebalance => Span::styled(
                                            "⦿",
                                            Style::default().fg(state.theme.yellow),
                                        ),
                                        ConsumerGroupState::Rebalancing => Span::styled(
                                            "⦿",
                                            Style::default().fg(state.theme.yellow),
                                        ),
                                        ConsumerGroupState::UnknownRebalance => Span::styled(
                                            "⊘",
                                            Style::default().fg(state.theme.black),
                                        ),
                                    }
                                    .into_right_aligned_line(),
                                ),
                                Cell::new(Span::styled(item.1.name.clone(), Style::default())),
                                Cell::new(Span::styled(item.1.state.to_string(), Style::default())),
                                Cell::new(
                                    Span::styled(
                                        item.1.members.len().to_string(),
                                        Style::default(),
                                    )
                                    .into_right_aligned_line(),
                                ),
                                Cell::new(
                                    Span::styled("1", Style::default()).into_right_aligned_line(),
                                ),
                                Cell::new(
                                    Span::styled("?", Style::default()).into_right_aligned_line(),
                                ),
                            ])
                            .height(1_u16)
                        }),
                );
            }

            let focused = state.is_focused(self.id());
            let table = Table::new(
                rows,
                [
                    Constraint::Length(1),
                    Constraint::Length(42),
                    Constraint::Length(24),
                    Constraint::Length(10),
                    Constraint::Length(32),
                    Constraint::Length(6),
                ],
            )
            .column_spacing(2)
            .header(header.clone())
            .row_highlight_style(match focused {
                true => Style::default()
                    .bg(state.theme.bg_focused_selected)
                    .fg(state.theme.fg_focused_selected)
                    .bold(),
                false => Style::default()
                    .bg(state.theme.bg_unfocused_selected)
                    .fg(state.theme.fg_unfocused_selected),
            });

            let table_area = block.inner(rect);

            let detail = self.details.first().unwrap();

            let text = vec![
                Line::from(detail.name.clone()).style(Style::default().bold()),
                Line::from(format!(
                    "{} partitions, {} replicas",
                    detail.partitions, detail.replicas
                ))
                .style(Style::default()),
                Line::from(format!(
                    "{} records, {} consumer groups",
                    detail.count.separate_with_underscores(),
                    detail.consumer_groups.len()
                )),
                Line::from(""),
            ];

            let block_experimental = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default())
                .padding(Padding::horizontal(1))
                .border_type(BorderType::Rounded);

            f.render_widget(
                Paragraph::new(
                    "🔬 The following list of consumer members is experimental, use it with caution.",
                )
                .block(block_experimental),
                Rect {
                    x: 0,
                    y: 9,
                    width: rect.width + 3,
                    height: 3,
                }
                .inner(Margin::new(8, 0)),
            );

            f.render_stateful_widget(
                table,
                Rect {
                    x: table_area.x,
                    y: table_area.y + 7,
                    width: table_area.width,
                    height: table_area.height.saturating_sub(4),
                },
                &mut self.state.clone(), //.with_offset((self.scroll.value() + table_area.y + 10).into()),
            );

            f.render_widget(
                Paragraph::new(text)
                    .style(Style::default())
                    .block(block.clone()),
                rect,
            );

            //f.render_widget(widget, area);
            //self.scroll.draw(f, rect, self.content_height());

            //
            //            let mut text: Vec<Line<'_>> = vec![];
            //            for d in &self.details {
            //                text.push(Line::from(format!(
            //                    "{} - {} {}",
            //                    d.0,
            //                    d.1,
            //                    match d.1 > 1 {
            //                        true => "partitions",
            //                        false => "partition",
            //                    }
            //                )));
            //                for (k, v) in &d.2 {
            //                    text.push(Line::from(format!("{}: lag of {}", k, v)));
            //                }
            //            }
            //
        }

        Ok(())
    }
}

impl TopicDetailsComponent {
    fn all_consumer_members(&self) -> Vec<&ConsumerGroupDetail> {
        self.details
            .iter()
            .flat_map(|e| &e.consumer_groups)
            .collect()
    }

    fn next(&mut self) {
        let consumer_members = self.all_consumer_members();
        if consumer_members.is_empty() {
            self.state.select(None);
            return;
        }

        let consumer_members = self.all_consumer_members();
        let i = match self.state.selected() {
            Some(i) => {
                if i >= consumer_members.len() - 1 {
                    i
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let consumer_members = self.all_consumer_members();
        if consumer_members.is_empty() {
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
        match self.all_consumer_members().is_empty() {
            true => self.state.select(None),
            false => self.state.select(Some(0)),
        }
    }

    fn last(&mut self) {
        let consumer_members = self.all_consumer_members();
        match consumer_members.is_empty() {
            true => self.state.select(None),
            false => self.state.select(Some(consumer_members.len() - 1)),
        }
    }
}
