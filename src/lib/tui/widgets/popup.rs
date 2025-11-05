use ratatui::{
    buffer::Buffer,
    layout::{self, Constraint, Flex, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

pub struct Popup<'a> {
    // Custom widget properties
    content: Text<'a>,
    title: String,
}

impl<'a> Widget for Popup<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = Self::popup_area(area, 30, 15);
        Clear.render(area, buf);

        let title = Line::from(self.title)
            .bold()
            .style(Style::new().blue())
            .centered();

        let block = Block::default()
            .borders(Borders::all())
            .border_type(ratatui::widgets::BorderType::Double)
            .title(title);

        let layout: [Rect; 2] = Layout::new(
            layout::Direction::Vertical,
            [Constraint::Percentage(80), Constraint::Length(1)],
        )
        .areas(block.inner(area));

        block.render(area, buf);
        let text = Paragraph::new(self.content).centered();
        text.render(layout[0], buf);

        Paragraph::new("OK")
            .reversed()
            .blue()
            .render(layout[1], buf);
    }
}

impl<'a> Popup<'a> {
    pub fn new(title: String, content: Text<'a>) -> Self {
        Self { title, content }
    }

    fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
        let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
    }
}
