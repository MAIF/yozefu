use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph},
};

use crate::error::TuiError;

use super::{Component, ComponentName, State};

#[derive(Default)]
pub(crate) struct IssueComponent {}

impl Component for IssueComponent {
    fn id(&self) -> ComponentName {
        ComponentName::Help
    }

    fn draw(&mut self, f: &mut Frame<'_>, rect: Rect, state: &State) -> Result<(), TuiError> {
        let block = Block::default()
            .borders(Borders::ALL)
            .padding(Padding::symmetric(5, 0))
            .style(Style::default())
            .border_type(BorderType::Rounded);

        let block = self.make_block_focused(state, block);
        let paragraph = Paragraph::new(vec![
            Line::from(""),
            Line::from("ʕノ•ᴥ•ʔノ ︵ ┻━┻"),
            Line::from("Alt: Your mate flipping the desk").italic(),
            Line::from(""),
            Line::from("Are you struggling with the tool?"),
            Line::from("Leave us an issue so we can improve it:"),
            Line::from("https://github.com/MAIF/yozefu/issues").bold(),
            Line::from(""),
            Line::from("Press any key to close.").italic(),
        ]);

        let mut rect = rect;
        rect.width -= 2;
        rect.y += 1;

        let [area] = Layout::horizontal([Constraint::Length(54)])
            .flex(Flex::End)
            .areas(rect);
        let [area] = Layout::vertical([Constraint::Length(11)])
            .flex(Flex::Start)
            .areas(area);

        f.render_widget(Clear, area);
        f.render_widget(paragraph.block(block), area);

        Ok(())
    }
}
