use std::path::Path;

use ratatui::{
    layout::{self, Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Paragraph, StatefulWidget, Widget},
};

const MAX_VARIANTS: usize = 5;

#[derive(Debug, PartialEq)]
pub struct PathHintState {
    pub possible_paths: Vec<String>,
    selected: usize,
}

impl Default for PathHintState {
    fn default() -> Self {
        Self {
            possible_paths: Vec::with_capacity(MAX_VARIANTS),
            selected: 0,
        }
    }
}

pub struct PathHint {}

impl StatefulWidget for PathHint {
    type State = PathHintState;

    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State) {
        if !state.possible_paths.is_empty() {
            let layout: [Rect; 1] = Layout::new(
                layout::Direction::Vertical,
                [Constraint::Length((MAX_VARIANTS).try_into().unwrap())],
            )
            .areas(area);
            let block = Block::default();
            let lines: Vec<Line<'_>> = state
                .possible_paths
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    Line::from(s.as_str()).style(if i == state.selected {
                        Style::new().blue().reversed()
                    } else {
                        Style::new().white()
                    })
                })
                .collect();

            Paragraph::new(Text::from_iter(lines))
                .block(block)
                .render(layout[0], buf);
        }
    }
}

impl PathHint {
    pub fn new() -> Self {
        Self {}
    }
}

impl PathHintState {
    pub fn get_hints(&mut self, current_path: &str) {
        self.possible_paths.clear();
        self.selected = 0;

        let path = Path::new(current_path);
        if path.is_dir()
            && let Ok(read_dir) = path.read_dir()
            && current_path.ends_with('/')
        {
            for entry in read_dir
                .filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().into_string().ok())
                .take(MAX_VARIANTS)
            {
                self.possible_paths.push(entry);
            }
            return;
        }

        if let Some(parent) = path.parent()
            && let Ok(read_dir) = parent.read_dir()
        {
            for entry in read_dir
                .filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().into_string().ok())
                .filter(|e| e.starts_with(path.file_name().unwrap().to_str().unwrap()))
                .take(MAX_VARIANTS)
            {
                self.possible_paths.push(entry);
            }
        }
    }

    pub fn get_selected(&mut self) -> Option<&String> {
        self.possible_paths.get(self.selected)
    }

    pub fn next(&mut self) {
        if self.possible_paths.is_empty() {
            return;
        }
        self.selected += 1;
        self.selected %= self.possible_paths.len();
    }

    pub fn previous(&mut self) {
        if self.possible_paths.is_empty() {
            return;
        }
        if self.selected == 0 {
            self.selected = self.possible_paths.len() - 1;
            return;
        }
        self.selected -= 1;
    }
}
