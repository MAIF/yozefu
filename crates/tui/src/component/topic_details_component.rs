//! Component showing information regarding a given topic: partitions, consumer groups, replicas ...
use std::collections::HashSet;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use itertools::Itertools;
use lib::{ConsumerGroupDetail, ConsumerGroupState, TopicDetail};
use ratatui::{
    Frame,
    layout::{Margin, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, TableState},
};
use thousands::Separable;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    Action, Notification, action::Level, component::scroll_state::ScrollState, error::TuiError,
};

use super::{Component, ComponentName, State, WithHeight};

#[derive(Default)]
pub(crate) struct TopicDetailsComponent {
    details: Vec<TopicDetail>,
    action_tx: Option<UnboundedSender<Action>>,
    state: TableState,
    scroll: ScrollState,
    refreshing_data: bool,
    throbber_state: throbber_widgets_tui::ThrobberState,
}

impl WithHeight for TopicDetailsComponent {
    fn content_height(&self) -> usize {
        self.details
            .iter()
            .map(|e| {
                e.consumer_groups.len()
                    + 50
                    + e.config.as_ref().map(|e| e.entries.len()).unwrap_or(0)
            })
            .sum::<usize>()
    }
}

impl Component for TopicDetailsComponent {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) {
        self.action_tx = Some(tx);
    }

    fn id(&self) -> ComponentName {
        ComponentName::TopicDetails
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>, TuiError> {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.next();
                self.scroll.scroll_to_next_line();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.previous();
                self.scroll.scroll_to_previous_line();
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
                        Level::Info,
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
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect, state: &State) -> Result<(), TuiError> {
        let block = Block::new()
            .borders(Borders::ALL)
            .border_style(Style::default())
            .title(" Topic details ")
            .padding(Padding::horizontal(2))
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
                rect.inner(Margin::new(5, 2)),
                &mut self.throbber_state,
            );
            return Ok(());
        }

        let mut lines: Vec<Line<'_>> = vec![];
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
                rect.inner(Margin::new(5, 2)),
                &mut self.throbber_state,
            );
        }

        let detail = self.details.first().unwrap();

        lines.extend(vec![
            Line::from(""),
            Line::from(""),
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
        ]);

        if !self.details.is_empty() {
            lines.push(Line::from(
                "     🔬 The following list of consumer members is experimental, use it with caution.",
            ));
            lines.push(Line::from(""));
            lines.push(
                Line::from(vec![
                    Span::from(" "),
                    Span::from(format!("    {:<42}", "Name")),
                    Span::from(format!("    {:<23}", "State")),
                    Span::from(format!("    {:>10}", "Partitions")),
                    Span::from(format!("    {:>32}", "Members")),
                    Span::from(format!("    {:>6}", "Lag")),
                ])
                .bold(),
            );

            for detail in &self.details {
                let consumers_groups = detail.consumer_groups.clone();
                lines.extend(
                    consumers_groups
                        .into_iter()
                        .sorted_by(|a, b| a.name.cmp(&b.name))
                        .enumerate()
                        .map(|item| {
                            Line::from(vec![
                                (match item.1.state {
                                    ConsumerGroupState::Unknown => {
                                        Span::styled("⊘", Style::default().fg(state.theme.red))
                                    }
                                    ConsumerGroupState::Empty => {
                                        Span::styled("◯", Style::default().fg(state.theme.red))
                                    }
                                    ConsumerGroupState::Dead => {
                                        Span::styled("⊗", Style::default().fg(state.theme.red))
                                    }
                                    ConsumerGroupState::Stable => {
                                        Span::styled("⏺︎", Style::default().fg(state.theme.green))
                                    }
                                    ConsumerGroupState::PreparingRebalance => {
                                        Span::styled("⦿", Style::default().fg(state.theme.yellow))
                                    }
                                    ConsumerGroupState::CompletingRebalance => {
                                        Span::styled("⦿", Style::default().fg(state.theme.yellow))
                                    }
                                    ConsumerGroupState::Rebalancing => {
                                        Span::styled("⦿", Style::default().fg(state.theme.yellow))
                                    }
                                    ConsumerGroupState::UnknownRebalance => {
                                        Span::styled("⊘", Style::default().fg(state.theme.black))
                                    }
                                }),
                                Span::styled(
                                    format!("    {:<42}", item.1.name.clone()),
                                    Style::default(),
                                ),
                                Span::styled(
                                    format!("    {:<23}", item.1.state.to_string()),
                                    Style::default(),
                                ),
                                Span::styled(
                                    format!("    {:>10}", item.1.members.len().to_string()),
                                    Style::default(),
                                ),
                                Span::styled(format!("    {:>32}", "1"), Style::default()),
                                Span::styled(format!("    {:>6}", "?"), Style::default()),
                            ])
                        }),
                );
            }

            let detail = self.details.first().unwrap();

            //            f.render_widget(
            //                Paragraph::new(
            //                    "🔬 The following list of consumer members is experimental, use it with caution.",
            //                )
            //                .block(block_experimental),
            //                Rect {
            //                    x: 0,
            //                    y: 10.min(rect.height), // to avoid panicking with 'index outside of buffer'
            //                    width: rect.width + 3,
            //                    height: 3.min(rect.height),
            //                }
            //                .inner(Margin::new(7, 0)),
            //            );

            if let Some(config) = &detail.config {
                lines.push(Line::from(""));
                lines.push(
                    Line::from(format!("{:>42}    {}", "Topic configuration", "Value"))
                        .style(Style::default().bold()),
                );

                for (key, value) in config.entries.iter().sorted_by(|a, b| a.0.cmp(b.0)) {
                    lines.push(Line::from(format!("{:>42}    {}", key, value)));
                }
            }
            let number_of_lines = lines.len();

            f.render_widget(
                Paragraph::new(Text::from(lines))
                    .scroll((self.scroll.value(), 0))
                    .style(Style::default())
                    .block(block.clone()),
                rect,
            );

            self.scroll.draw(f, rect, number_of_lines);
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

#[cfg(test)]
use crate::assert_draw;

#[test]
fn test_draw() {
    let mut component = TopicDetailsComponent::default();

    component
        .update(Action::TopicDetails(vec![TopicDetail {
            name: "travel-stories".to_string(),
            partitions: 4,
            replicas: 6,
            consumer_groups: vec![],
            count: 0,
            config: None,
        }]))
        .unwrap();
    assert_draw!(component, 120, 20)
}

#[test]
fn test_draw_out_of_bounds() {
    let mut component = TopicDetailsComponent::default();

    component
        .update(Action::TopicDetails(vec![TopicDetail {
            name: "travel-stories".to_string(),
            partitions: 4,
            replicas: 6,
            consumer_groups: vec![],
            count: 0,
            config: None,
        }]))
        .unwrap();
    //todo!("something needs to be fixed")
    //assert_draw!(component, 60, 3)
}
