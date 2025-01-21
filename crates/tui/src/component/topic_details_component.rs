//! Component showing information regarding a given topic: partitions, consumer groups, replicas ...
use crossterm::event::{KeyCode, KeyEvent};

use itertools::Itertools;
use lib::{ConsumerGroupState, TopicDetail};
use ratatui::{
    layout::{Alignment, Constraint, Margin, Offset, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, Padding, Paragraph, Row, Table, TableState,
    },
    Frame,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{error::TuiError, Action};

use super::{scroll_state::ScrollState, Component, ComponentName, State, WithHeight};

#[derive(Default)]
pub struct TopicDetailsComponent {
    pub details: Vec<TopicDetail>,
    pub action_tx: Option<UnboundedSender<Action>>,
    pub state: TableState,
    pub scroll: ScrollState,
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
            _ => (),
        }
        Ok(None)
    }

    fn update(&mut self, action: Action) -> Result<Option<Action>, TuiError> {
        match action {
            Action::Tick => self.throbber_state.calc_next(),
            Action::TopicDetails(details) => {
                self.details = details;
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
                .label("This feature is not ready yet. Fetching data, please wait...")
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

        if !self.details.is_empty() {
            f.render_widget(Clear, rect);
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
            .header(header.clone());

            let table_area = block.inner(rect);

            let detail = self.details.first().unwrap();

            let text = vec![
                Line::from(detail.name.clone()).style(Style::default().bold()),
                Line::from(format!(
                    "{} partitions, {} replicas",
                    detail.partitions, detail.replicas
                ))
                .style(Style::default()),
                Line::from(format!("{} consumer groups", detail.consumer_groups.len()))
                    .style(Style::default()),
                Line::from(""),
                Line::from(""),
            ];

            f.render_stateful_widget(
                table,
                table_area.offset(Offset { x: 0, y: 5 }),
                &mut self
                    .state
                    .clone()
                    .with_offset((self.scroll.value() + table_area.y + 10).into()),
            );

            f.render_widget(
                Paragraph::new(text)
                    .style(Style::default())
                    .block(block.clone()),
                rect,
            );

            self.scroll.draw(f, rect, self.content_height());

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
