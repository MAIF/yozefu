//! Component showing the help

use crossterm::event::{KeyCode, KeyEvent};
use itertools::Itertools;
use ratatui::{
    layout::Rect,
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, Wrap},
    Frame,
};

use crate::{error::TuiError, Action};

use super::{
    issue_component::IssueComponent, scroll_state::ScrollState, Component, ComponentName, Shortcut,
    State,
};

const HELP_HEIGHT: usize = 42;
const TEN_MINUTES_FRAME: usize = 30 * 60 * 10;
const REPOSITORY_URL: &str = concat!(
    "      https://github.com/MAIF/yozefu/tree/v",
    env!("CARGO_PKG_VERSION")
);

#[derive(Default)]
pub struct HelpComponent {
    pub scroll: ScrollState,
    pub rendered: usize,
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
            Line::from(""),
            Line::from(""),
            Line::from("                                                           Key      Description").bold(),
            Line::from("                                                             /      Focus search input"),
            Line::from("                                                           ESC      Close the window/app"),
            Line::from("                                                           TAB      Focus next window"),
            Line::from("                                                   SHIFT + TAB      Focus previous window"),
            Line::from(""),

            Line::from("                                                      Variable      Type                        Alias       Description").bold(),
            Line::from("                                                         topic      String                          t       Kafka topic"),
            Line::from("                                                        offset      Number                          o       Offset of the record"),
            Line::from("                                                           key                                      k       Key of the record"),
            Line::from("                                                         value                                      v       Value of the record"),
            Line::from("                                                     partition      Number                          p       Partition of the record"),
            Line::from("                                                     timestamp      String                         ts       Timestamp of the record"),
            Line::from("                                                          size      String                         si       Size of the record"),
            Line::from("                                                       headers      Map<String, String>             h       Headers of the record"),
            Line::from(""),

            Line::from("                                                      Operator      Type                                    Description").bold(),
            Line::from("                                     == | != | > | >= | < | <=      Number | String                         Wayne's world, party time! Excellent!"),
            Line::from("                                                 contains | ~=      String                                  Test if the variable contains the specified string"),
            Line::from("                                                   starts with      String                                  Test if the variable starts with the specified string"),
            Line::from(""),


            Line::from("                                                        Clause      Syntax                                  Description").bold(),
            Line::from("                                                         limit      limit <number>                          Limit the number of kafka records to receive"),
            Line::from("                                                          from      from <begin|end|date|offset>            Start consuming records from the beginning, the end or a date"),
            Line::from("                                                      order by      order by <var> <asc|desc>               Sort kafka records"),
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
                                        "      Theme is '{}'. You can switch between [{}] in the config file or with the '--theme' flag",
                                        state.theme.name,
                                        state.themes.iter().filter(|f| *f != &state.theme.name).join(", ")
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
