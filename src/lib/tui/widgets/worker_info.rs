use std::sync::mpsc::{self, Receiver};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{self, Constraint, Flex, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};

use crate::lib::worker::{
    builder::{DEFAULT_RECURSIVE_MODE, DEFAULT_THREADS_NUMBER, DEFAULT_TIMEOUT, WorkerBuilder},
    messages::WorkerMessage,
    worker::Worker,
};

#[derive(Debug)]
pub enum WorkerVariant {
    Worker(Worker),
    Builder(WorkerBuilder),
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Field {
    #[default]
    Name,
    Threads,
    Recursion,
    Timeout,
    WordlistPath,
    URI,
    RunButton,
}

impl Default for WorkerVariant {
    fn default() -> Self {
        WorkerVariant::Builder(WorkerBuilder::default())
    }
}

#[derive(Debug)]
pub struct WorkerProperties {
    threads: String,
    recursion: String,
    timeout: String,
    wordlist_path: String,
    uri: String,
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
    properties: WorkerProperties,
    rx: Receiver<WorkerMessage>,
    log: Vec<String>,
    progress_max: usize,
    progress_current: usize,
    is_running: bool,
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
            WorkerVariant::Worker(worker) => todo!(),
            WorkerVariant::Builder(_) => {
                let layout: [ratatui::prelude::Rect; 7] = Layout::new(
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
                    Field::URI => uri_block = uri_block.border_style(Style::default().red()),
                    Field::RunButton => run_button = run_button.border_style(Style::default().green()),
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
                        Field::URI => uri_paragraph = uri_paragraph.italic(),
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
                Paragraph::new("Run").centered().block(run_button).alignment(layout::Alignment::Center).render(Self::center(layout[6], Constraint::Max(40), Constraint::Length(3)), buf);

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

impl Default for WorkerState {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel::<WorkerMessage>();
        Self {
            name: "Unnamed".to_string(),
            worker: WorkerVariant::Builder(WorkerBuilder::new().message_sender(tx.into())),
            rx,
            currently_editing: Default::default(),
            properties: Default::default(),
            selected: Default::default(),
            log: Default::default(),
            progress_max: Default::default(),
            progress_current: Default::default(),
            is_running: Default::default(),
        }
    }
}

impl WorkerState {
    pub fn handle_keys(&mut self, key: KeyEvent, is_editing: &mut bool) {
        match self.worker {
            WorkerVariant::Worker(_) => match (key.modifiers, key.code) {
                _ => {}
            },
            WorkerVariant::Builder(_) => match (key.modifiers, key.code) {
                (_, KeyCode::Down) => match &self.selected {
                    Field::Name => self.selected = Field::URI,
                    Field::URI => self.selected = Field::Threads,
                    Field::Threads => self.selected = Field::Recursion,
                    Field::Recursion => self.selected = Field::Timeout,
                    Field::Timeout => self.selected = Field::WordlistPath,
                    Field::WordlistPath => self.selected = Field::RunButton,
                    Field::RunButton => self.selected = Field::Name,
                },
                (_, KeyCode::Up) => match &self.selected {
                    Field::Name => self.selected = Field::RunButton,
                    Field::URI => self.selected = Field::Name,
                    Field::Threads => self.selected = Field::URI,
                    Field::Recursion => self.selected = Field::Threads,
                    Field::Timeout => self.selected = Field::Recursion,
                    Field::WordlistPath => self.selected = Field::Timeout,
                    Field::RunButton => self.selected = Field::WordlistPath,
                },
                (_, KeyCode::Char('r')) => {
                    self.currently_editing = Some(self.selected);
                    *is_editing = true;
                }
                _ => {}
            },
        }
    }

    pub fn handle_editing(&mut self, key: KeyEvent, is_editing: &mut bool) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Char(ch)) => {
                match &self.currently_editing {
                    Some(field) => match field {
                        Field::Name => self.name.push(ch),
                        Field::Threads => {
                            if ch.is_digit(10) {
                                self.properties.threads.push(ch)
                            }
                        }
                        Field::Recursion => {
                            if ch.is_digit(10) {
                                self.properties.recursion.push(ch);
                            }
                        }
                        Field::Timeout => {
                            if ch.is_digit(10) {
                                self.properties.timeout.push(ch);
                            }
                        }
                        Field::WordlistPath => self.properties.wordlist_path.push(ch),
                        Field::URI => self.properties.uri.push(ch),
                        Field::RunButton => {}
                    },
                    _ => {}
                };
            }
            (_, KeyCode::Backspace) => match &self.currently_editing {
                Some(field) => match field {
                    Field::Name => _ = self.name.pop(),
                    Field::Threads => _ = self.properties.threads.pop(),
                    Field::Recursion => _ = self.properties.recursion.pop(),
                    Field::Timeout => _ = self.properties.timeout.pop(),
                    Field::WordlistPath => _ = self.properties.wordlist_path.pop(),
                    Field::URI => _ = self.properties.uri.pop(),
                    Field::RunButton => {}
                },
                _ => {}
            },
            (_, KeyCode::Enter) | (_, KeyCode::Esc) => {
                *is_editing = false;
                self.currently_editing = None;
            }
            _ => {}
        }
    }
}
