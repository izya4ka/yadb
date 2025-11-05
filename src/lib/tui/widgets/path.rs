use std::path::Path;

use ratatui::{
    layout::{self, Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

const MAX_VARIANTS: usize = 5;

pub struct PathHint<'a> {
    current_path: &'a str,
}

impl Widget for PathHint<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let layout: [Rect; 1] = Layout::new(
            layout::Direction::Vertical,
            [Constraint::Length((MAX_VARIANTS).try_into().unwrap())],
        )
        .areas(area);

        let block = Block::default();

        let strs = self.get_hints();

        if !strs.is_empty() {
            let mut lines: Vec<Line<'_>> = strs.into_iter().map(Line::from).collect();
            lines[0] = lines[0].clone().style(Style::new().blue().reversed());

            Paragraph::new(Text::from_iter(lines))
                .block(block)
                .render(layout[0], buf);
        }
    }
}

impl<'a> PathHint<'a> {
    fn get_hints(&'a self) -> Vec<String> {
        let mut hints: Vec<String> = Vec::new();

        let path = Path::new(self.current_path);
        if path.is_dir()
            && let Ok(read_dir) = path.read_dir()
        {
            for entry in read_dir
                .filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().into_string().ok())
                .take(5)
            {
                hints.push(entry);
            }
        } else {
            return hints;
        }

        if let Some(parent) = path.parent()
            && let Ok(read_dir) = parent.read_dir()
        {
            for entry in read_dir
                .filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().into_string().ok())
                .take(5)
            {
                if entry.starts_with(path.file_name().unwrap().to_str().unwrap()) {
                    hints.push(entry);
                }
            }
        }

        hints
    }

    pub fn new(current_path: &'a str) -> Self {
        Self { current_path }
    }
}
