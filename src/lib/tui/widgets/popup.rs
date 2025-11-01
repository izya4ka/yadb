use ratatui::{buffer::Buffer, layout::{Constraint, Flex, Layout, Rect}, style::{Style, Stylize}, text::{Line, Text}, widgets::{Block, Borders, Clear, Paragraph, Widget}};

pub struct Popup<'a> {
    // Custom widget properties
    content: Text<'a>,
    title: String
}

impl<'a> Widget for Popup<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {

            let area = Self::popup_area(area, 60, 40);

            let title = Line::from(self.title)
                .bold()
                .style(Style::new().blue())
                .centered();

            let block = Block::default()
                .borders(Borders::all())
                .border_type(ratatui::widgets::BorderType::Double)
                .title(title);

            let text = Paragraph::new(self.content).centered().block(block);
            Clear::default().render(area, buf);
            text.render(area, buf);
    }
}

impl<'a> Popup<'a> {
    pub fn new(title: String, content: Text<'a>) -> Self {
        Self {
            title,
            content
        }
    }

    fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
        let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
    }
}