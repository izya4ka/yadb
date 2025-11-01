use std::collections::VecDeque;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{self, Constraint, Flex, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Borders, Gauge, Paragraph, StatefulWidget, Widget},
};

use crate::lib::worker::builder::{
        DEFAULT_RECURSIVE_MODE, DEFAULT_THREADS_NUMBER, DEFAULT_TIMEOUT,
    };

#[derive(Debug, Default, Clone)]
pub enum WorkerVariant {
    Worker,
    #[default]
    Builder,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Field {
    #[default]
    Name,
    Threads,
    Recursion,
    Timeout,
    WordlistPath,
    Uri,
    RunButton,
}

#[derive(Debug)]
pub struct WorkerProperties {
    pub threads: String,
    pub recursion: String,
    pub timeout: String,
    pub wordlist_path: String,
    pub uri: String,
}

impl Default for WorkerProperties {
    fn default() -> Self {
        Self {
            threads: DEFAULT_THREADS_NUMBER.to_string(),
            recursion: DEFAULT_RECURSIVE_MODE.to_string(),
            timeout: DEFAULT_TIMEOUT.to_string(),
            wordlist_path: "/usr/share".into(),
            uri: "http://".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct WorkerState {
    pub name: String,
    pub worker: WorkerVariant,
    currently_editing: Option<Field>,
    selected: Field,
    pub current_parsing: String,
    pub properties: WorkerProperties,
    pub log: VecDeque<String>,
    pub messages: VecDeque<String>,
    pub progress_total: usize,
    pub progress_current: usize,
    pub do_build: bool,
}

impl Default for WorkerState {
    fn default() -> Self {
        Self {
            name: "Unnamed".to_string(),
            worker: Default::default(),
            currently_editing: Default::default(),
            selected: Default::default(),
            current_parsing: Default::default(),
            properties: Default::default(),
            log: Default::default(),
            messages: Default::default(),
            progress_total: 1,
            progress_current: 0,
            do_build: Default::default(),
        }
    }
}

#[derive(Debug, Default)]
pub struct WorkerInfo {}

impl StatefulWidget for WorkerInfo {
    type State = WorkerState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        match &state.worker {
            WorkerVariant::Worker => {
                let layout: [Rect; 4] = Layout::new(
                    layout::Direction::Vertical,
                    [
                        Constraint::Min(10),
                        Constraint::Min(30),
                        Constraint::Max(3),
                        Constraint::Max(10),
                    ],
                )
                .areas(area);

                let log_block = Block::new().borders(Borders::all()).title(" Logs ");
                let message_block = Block::new().borders(Borders::all()).title(" Results ");
                let current_block = Block::new()
                    .borders(Borders::all())
                    .title(" Currently requesting ");

                let log_lines = state.log.iter().map(|s| Line::from(s.as_str()));
                let message_lines = state.messages.iter().map(|s| Line::from(s.as_str()));

                Paragraph::new(Text::from_iter(log_lines))
                    .block(log_block)
                    .render(layout[0], buf);

                Paragraph::new(Text::from_iter(message_lines))
                    .block(message_block)
                    .render(layout[1], buf);
                
                Paragraph::new(Line::from(state.current_parsing.as_str()))
                    .block(current_block)
                    .render(layout[2], buf);

                Gauge::default()
                    .block(Block::bordered().title("Progress"))
                    .gauge_style(Style::new().white().on_black().italic())
                    .ratio((state.progress_current as f64) / (state.progress_total as f64))
                    .render(layout[3], buf);
            }
            WorkerVariant::Builder => {
                let layout: [Rect; 7] = Layout::new(
                    layout::Direction::Vertical,
                    [
                        Constraint::Max(3),
                        Constraint::Max(3),
                        Constraint::Max(3),
                        Constraint::Max(3),
                        Constraint::Max(3),
                        Constraint::Max(3),
                        Constraint::Max(5),
                    ],
                )
                .areas(area);

                // NAME
                let mut name_block = Block::new().borders(Borders::all()).title(" Name ");
                let mut name_paragraph = Paragraph::new(state.name.as_str());

                // URI
                let mut uri_block = Block::new().borders(Borders::all()).title(" URI ");
                let mut uri_paragraph = Paragraph::new(state.properties.uri.as_str());

                // THREADS
                let mut threads_block = Block::new().borders(Borders::all()).title(" Threads ");
                let mut threads_paragraph = Paragraph::new(state.properties.threads.as_str());

                // RECURSION DEPTH
                let mut recursion_block = Block::new()
                    .borders(Borders::all())
                    .title(" Recursion depth ");
                let mut recursion_paragraph = Paragraph::new(state.properties.recursion.as_str());

                // MAX TIMEOUT
                let mut timeout_block = Block::new().borders(Borders::all()).title(" Max Timeout ");
                let mut timeout_paragraph = Paragraph::new(state.properties.timeout.as_str());

                // WORDLIST PATH
                let mut wordlist_path_block = Block::new()
                    .borders(Borders::all())
                    .title(" Wordlist path ");
                let mut wordlist_path_paragraph =
                    Paragraph::new(state.properties.wordlist_path.as_str());

                // RUN BUTTON
                let mut run_button = Block::new().borders(Borders::all());

                match state.selected {
                    Field::Name => name_block = name_block.border_style(Style::default().red()),
                    Field::Threads => {
                        threads_block = threads_block.border_style(Style::default().red())
                    }
                    Field::Recursion => {
                        recursion_block = recursion_block.border_style(Style::default().red())
                    }
                    Field::Timeout => {
                        timeout_block = timeout_block.border_style(Style::default().red())
                    }
                    Field::WordlistPath => {
                        wordlist_path_block =
                            wordlist_path_block.border_style(Style::default().red())
                    }
                    Field::Uri => uri_block = uri_block.border_style(Style::default().red()),
                    Field::RunButton => {
                        run_button = run_button.border_style(Style::default().green())
                    }
                }

                if let Some(input) = &state.currently_editing {
                    match input {
                        Field::Name => name_paragraph = name_paragraph.italic(),
                        Field::Threads => threads_paragraph = threads_paragraph.italic(),
                        Field::Recursion => recursion_paragraph = recursion_paragraph.italic(),
                        Field::Timeout => timeout_paragraph = timeout_paragraph.italic(),
                        Field::WordlistPath => {
                            wordlist_path_paragraph = wordlist_path_paragraph.italic()
                        }
                        Field::Uri => uri_paragraph = uri_paragraph.italic(),
                        _ => {}
                    }
                }

                name_paragraph.block(name_block).render(layout[0], buf);
                uri_paragraph.block(uri_block).render(layout[1], buf);
                threads_paragraph
                    .block(threads_block)
                    .render(layout[2], buf);
                recursion_paragraph
                    .block(recursion_block)
                    .render(layout[3], buf);
                timeout_paragraph
                    .block(timeout_block)
                    .render(layout[4], buf);
                wordlist_path_paragraph
                    .block(wordlist_path_block)
                    .render(layout[5], buf);
                Paragraph::new("Run")
                    .centered()
                    .block(run_button)
                    .alignment(layout::Alignment::Center)
                    .render(
                        Self::center(layout[6], Constraint::Max(40), Constraint::Length(3)),
                        buf,
                    );
            }
        }
    }
}

impl WorkerInfo {
    fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
        let [area] = Layout::horizontal([horizontal])
            .flex(Flex::Center)
            .areas(area);
        let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
        area
    }
}

impl WorkerState {
    pub fn handle_keys(&mut self, key: KeyEvent, is_editing: &mut bool) {
        match self.worker {
            WorkerVariant::Worker => match (key.modifiers, key.code) {
                _ => {}
            },
            WorkerVariant::Builder => match (key.modifiers, key.code) {
                (_, KeyCode::Down) => match &self.selected {
                    Field::Name => self.selected = Field::Uri,
                    Field::Uri => self.selected = Field::Threads,
                    Field::Threads => self.selected = Field::Recursion,
                    Field::Recursion => self.selected = Field::Timeout,
                    Field::Timeout => self.selected = Field::WordlistPath,
                    Field::WordlistPath => self.selected = Field::RunButton,
                    Field::RunButton => self.selected = Field::Name,
                },
                (_, KeyCode::Up) => match &self.selected {
                    Field::Name => self.selected = Field::RunButton,
                    Field::Uri => self.selected = Field::Name,
                    Field::Threads => self.selected = Field::Uri,
                    Field::Recursion => self.selected = Field::Threads,
                    Field::Timeout => self.selected = Field::Recursion,
                    Field::WordlistPath => self.selected = Field::Timeout,
                    Field::RunButton => self.selected = Field::WordlistPath,
                },
                (_, KeyCode::Char('r')) => {
                    self.currently_editing = Some(self.selected);
                    *is_editing = true;
                }
                (_, KeyCode::Enter) => {
                    if self.selected == Field::RunButton {
                        self.do_build = true;
                    }
                }
                _ => {}
            },
        }
    }

    pub fn handle_editing(&mut self, key: KeyEvent, is_editing: &mut bool) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Char(ch)) => {
                if let Some(field) = &self.currently_editing { match field {
                        Field::Name => self.name.push(ch),
                        Field::Threads => {
                            if ch.is_ascii_digit() {
                                self.properties.threads.push(ch)
                            }
                        }
                        Field::Recursion => {
                            if ch.is_ascii_digit() {
                                self.properties.recursion.push(ch);
                            }
                        }
                        Field::Timeout => {
                            if ch.is_ascii_digit() {
                                self.properties.timeout.push(ch);
                            }
                        }
                        Field::WordlistPath => self.properties.wordlist_path.push(ch),
                        Field::Uri => self.properties.uri.push(ch),
                        Field::RunButton => {}
                    }
                }
            }
            (_, KeyCode::Backspace) => if let Some(field) = &self.currently_editing {
                match field {
                    Field::Name => _ = self.name.pop(),
                    Field::Threads => _ = self.properties.threads.pop(),
                    Field::Recursion => _ = self.properties.recursion.pop(),
                    Field::Timeout => _ = self.properties.timeout.pop(),
                    Field::WordlistPath => _ = self.properties.wordlist_path.pop(),
                    Field::Uri => _ = self.properties.uri.pop(),
                    Field::RunButton => {},
                }
            },
            (_, KeyCode::Enter) | (_, KeyCode::Esc) => {
                *is_editing = false;
                self.currently_editing = None;
            }
            _ => {}
        }
    }
}
