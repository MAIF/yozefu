//! Component showing the help

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, Wrap},
};

use crate::{Action, error::TuiError};

use super::{
    Component, ComponentName, Shortcut, State, issue_component::IssueComponent,
    scroll_state::ScrollState,
};

const HELP_HEIGHT: usize = 42;
const TEN_MINUTES_FRAME: usize = 30 * 60 * 10;
const REPOSITORY_URL: &str = concat!(
    "      https://github.com/MAIF/yozefu/tree/v",
    env!("CARGO_PKG_VERSION")
);

#[derive(Default)]
pub(crate) struct HelpComponent {
    scroll: ScrollState,
    rendered: usize,
}

impl Component for HelpComponent {
    fn id(&self) -> ComponentName {
        ComponentName::Help
    }

    fn shortcuts(&self) -> Vec<Shortcut> {
        vec![]
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>, TuiError> {
        self.rendered = 0;
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

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect, state: &State) -> Result<(), TuiError> {
        f.render_widget(Clear, rect);

        let block = Block::default()
            .borders(Borders::ALL)
            .padding(Padding::horizontal(2))
            .border_type(BorderType::Rounded)
            .title(" Help ");

        let block = self.make_block_focused_with_state(state, block);

        let text = vec![
            Line::from(Span::raw("")),
            Line::from(Span::raw("")),
            Line::from("                                                           Key      Description").bold(),
            Line::from("                                                             /      Focus search input"),
            Line::from("                                                           ESC      Close the window/app"),
            Line::from("                                                           TAB      Focus next window"),
            Line::from("                                                   SHIFT + TAB      Focus previous window"),
            Line::from(""),

            Line::from("                                                      Variable      Type                        Alias       Description").bold(),
            Line::from(vec![Span::raw("                                                         topic      "), Span::from("String").fg(state.theme.green), Span::from("                          t").fg(state.theme.blue), Span::from("        Kafka topic")]),
            Line::from(vec![Span::raw("                                                        offset      "), Span::from("Number").fg(state.theme.green), Span::from("                          o").fg(state.theme.blue), Span::from("       Offset of the record")]),
            Line::from(vec![Span::raw("                                                           key      "), Span::from(""), Span::from("                                k").fg(state.theme.blue), Span::from("       Key of the record")]),
            Line::from(vec![Span::raw("                                                         value      "), Span::from(""), Span::from("                                v").fg(state.theme.blue), Span::from("       Value of the record")]),
            Line::from(vec![Span::raw("                                                     partition      "), Span::from("Number").fg(state.theme.green), Span::from("                          p").fg(state.theme.blue), Span::from("       Partition of the record")]),
            Line::from(vec![Span::raw("                                                     timestamp      "), Span::from("String").fg(state.theme.green), Span::from("                         ts").fg(state.theme.blue), Span::from("       Timestamp of the record (RFC 3339) → 2025-06-01T12:00:00.000+02:00")]),
            Line::from(vec![Span::raw("                                                          size      "), Span::from("Number").fg(state.theme.green), Span::from("                         si").fg(state.theme.blue), Span::from("       Size of the record")]),
            Line::from(vec![Span::raw("                                                       headers      "), Span::from("Map<String, String>").fg(state.theme.green), Span::from("             h").fg(state.theme.blue), Span::from("       Headers of the record")]),
            Line::from(Span::raw("")),

            Line::from(vec![Span::from("                                                      Operator"), Span::from("      Type").fg(state.theme.green), Span::from("                                    Description").bold()]),
            Line::from(vec![Span::from("                                     == | != | > | >= | < | <="), Span::from("      Number | String").fg(state.theme.green), Span::from("                         Wayne's world, party time! Excellent!")]),
            Line::from(vec![Span::from("                                                 contains | ~="), Span::from("      String").fg(state.theme.green), Span::from("                                  Test if the variable contains the specified string")]),
            Line::from(vec![Span::from("                                                   starts with"), Span::from("      String").fg(state.theme.green), Span::from("                                  Test if the variable starts with the specified string")]),
            Line::from(""),


            Line::from(vec![Span::from("                                                        Clause      Syntax                                  Description").bold()]),
            Line::from(vec![Span::from("                                                         limit      limit <"), Span::from("number").fg(state.theme.yellow), Span::from(">                          Limit the number of kafka records to receive")]),
            Line::from(vec![Span::from("                                                          from      from <"), Span::from("begin").fg(state.theme.yellow), Span::from("|"), Span::from("end").fg(state.theme.yellow), Span::from("|"), Span::from("date").fg(state.theme.yellow), Span::from("|"), Span::from("offset").fg(state.theme.yellow), Span::from(">            Start consuming records from the beginning, the end or a date")]),
            Line::from(vec![Span::from("                                                      order by      order by <"), Span::from("var").fg(state.theme.yellow), Span::from("> <"), Span::from("asc").fg(state.theme.yellow), Span::from("|"), Span::from("desc").fg(state.theme.yellow), Span::from(">               Sort kafka records")]),
            Line::from(""),

            Line::from("                                                         Input      Description").bold(),
            Line::from(r#"                                    timestamp >= "1 hours ago"      All records published within the last hour"#),
            Line::from(r#"v contains "rust" and partition == 2 from beginning limit 1000      The first 1_000 kafka records from partition 2 containing 'rust' in the value"#),
            Line::from(r#"              (key == "ABC") || (key ~= "XYZ") from end - 5000      Among the latest 5_000 records, return the records where the key is "ABC" or the key contains "XYZ""#),
            Line::from(r#"                      value.hello == "world" order by key desc      Any kafka JSON record with a JSON property "hello" with the value "world", sorted by key in descending order"#),
            Line::from(""),
            Line::from(vec![
                Span::from("                                                         Theme").bold(),
                Span::from(format!(
                                        "      Theme is '{}'. run `yozf config get themes` to list available themes.",
                                        state.theme.name
                                    ))
            ]),
            Line::from(vec![
                Span::from("                                                 Configuration").bold(),
                Span::from(format!("      '{}'", state.configuration_file.display()))
            ]),
            Line::from(vec![
                Span::from("                                                          Logs").bold(),
                Span::from(format!("      '{}'", state.logs_file.display()))
            ]),
            Line::from(vec![
                Span::from("                                                       Filters").bold(),
                Span::from(format!("      '{}'", state.filters_dir.display()))
            ]),
            Line::from(vec![
                Span::from("                                                        Themes").bold(),
                Span::from(format!("      '{}'", state.themes_file.display()))
            ]),
            Line::from(vec![
                Span::from("                                                       Version").bold(),
                Span::from(REPOSITORY_URL)
            ]),
            Line::from(""),
        ];

        let paragraph = Paragraph::new(text)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll.value(), 0));
        f.render_widget(paragraph.block(block), rect);

        if self.rendered > TEN_MINUTES_FRAME {
            let mut issue = IssueComponent::default();
            issue.draw(f, rect, state)?;
        }

        self.scroll.draw(f, rect, HELP_HEIGHT);
        self.rendered += 1;

        Ok(())
    }
}

#[cfg(test)]
use crate::assert_draw;

#[test]
fn test_draw() {
    let mut component = HelpComponent::default();
    assert_draw!(component, 300, 60)
}
