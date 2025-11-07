use std::collections::VecDeque;

use ratatui::{
    layout::{self, Constraint, Flex, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Gauge, Paragraph, StatefulWidget, Widget},
};

use crate::lib::{
    tui::{
        app::{LOG_MAX, MESSAGES_MAX},
        widgets::{
            field::{Field, FieldState, FieldType},
            path_hint::PathHintState,
        },
    },
    worker::builder::{DEFAULT_RECURSIVE_MODE, DEFAULT_THREADS_NUMBER, DEFAULT_TIMEOUT},
};

#[derive(Debug, Default, Clone)]
pub enum WorkerVariant {
    Worker(bool),
    #[default]
    Builder,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum FieldName {
    #[default]
    Name = 0,
    Uri = 1,
    Threads = 2,
    Recursion = 3,
    Timeout = 4,
    WordlistPath = 5,
    ProxyUrl = 6,
}

impl FieldName {
    pub fn index(self) -> usize {
        match self {
            FieldName::Name => 0,
            FieldName::Uri => 1,
            FieldName::Threads => 2,
            FieldName::Recursion => 3,
            FieldName::Timeout => 4,
            FieldName::WordlistPath => 5,
            FieldName::ProxyUrl => 6
        }
    }

    pub fn next(self) -> FieldName {
        match self {
            FieldName::Name => FieldName::Uri,
            FieldName::Uri => FieldName::Threads,
            FieldName::Threads => FieldName::Recursion,
            FieldName::Recursion => FieldName::Timeout,
            FieldName::Timeout => FieldName::WordlistPath,
            FieldName::WordlistPath => FieldName::ProxyUrl,
            FieldName::ProxyUrl => FieldName::Name,
        }
    }

    pub fn previous(self) -> FieldName {
        match self {
            FieldName::Name => FieldName::ProxyUrl,
            FieldName::Uri => FieldName::Name,
            FieldName::Threads => FieldName::Uri,
            FieldName::Recursion => FieldName::Threads,
            FieldName::Timeout => FieldName::Recursion,
            FieldName::WordlistPath => FieldName::Timeout,
            FieldName::ProxyUrl => FieldName::WordlistPath
        }
    }

    pub fn is_first(self) -> bool {
        self == FieldName::Name
    }

    pub fn is_last(self) -> bool {
        self == FieldName::ProxyUrl
    }
}

const FIELDS_NUMBER: usize = 7;

const NAMES: [&str; FIELDS_NUMBER] = [
    " Name ",
    " URI ",
    " Threads ",
    " Recursion depth ",
    " Max timeout ",
    " Wordlist path ",
    " Proxy URL "
];

#[derive(Debug, PartialEq)]
pub enum Selection {
    Field(FieldName),
    RunButton,
}

impl Default for Selection {
    fn default() -> Self {
        Selection::Field(FieldName::default())
    }
}

impl Selection {
    fn set_next(&mut self) {
        match self {
            Selection::Field(field) => {
                if field.is_last() {
                    *self = Selection::RunButton;
                    return;
                };
                *self = Selection::Field(field.next());
            }
            Selection::RunButton => *self = Selection::Field(FieldName::Name),
        }
    }

    fn set_previous(&mut self) {
        match self {
            Selection::Field(field) => {
                if field.is_first() {
                    *self = Selection::RunButton;
                    return;
                }
                *self = Selection::Field(field.previous());
            }
            Selection::RunButton => *self = Selection::Field(FieldName::WordlistPath),
        }
    }
}

#[derive(Debug)]
pub struct WorkerState {
    pub worker: WorkerVariant,
    pub selection: Selection,
    pub current_parsing: String,
    pub log: VecDeque<String>,
    pub messages: VecDeque<String>,
    pub progress_current_total: usize,
    pub progress_current_now: usize,
    pub progress_all_total: usize,
    pub progress_all_now: usize,
    pub do_build: bool,
    pub fields_states: [FieldState; FIELDS_NUMBER],
    cursor_position: (u16, u16),
}

impl Default for WorkerState {
    fn default() -> Self {
        Self {
            worker: Default::default(),
            cursor_position: Default::default(),
            selection: Default::default(),
            current_parsing: Default::default(),
            log: Default::default(),
            messages: Default::default(),
            do_build: Default::default(),
            progress_current_total: Default::default(),
            progress_current_now: Default::default(),
            progress_all_total: Default::default(),
            progress_all_now: Default::default(),
            fields_states: [
                FieldState::new("Unnamed", true, false, FieldType::Normal),
                FieldState::new("http://localhost", false, false, FieldType::Normal),
                FieldState::new(
                    DEFAULT_THREADS_NUMBER.to_string().as_str(),
                    false,
                    true,
                    FieldType::Normal,
                ),
                FieldState::new(
                    DEFAULT_RECURSIVE_MODE.to_string().as_str(),
                    false,
                    true,
                    FieldType::Normal,
                ),
                FieldState::new(
                    DEFAULT_TIMEOUT.to_string().as_str(),
                    false,
                    true,
                    FieldType::Normal,
                ),
                FieldState::new(
                    "/usr/share",
                    false,
                    false,
                    FieldType::Path(PathHintState::default()),
                ),
                FieldState::new(
                    "",
                     false, false, FieldType::Normal)
            ],
        }
    }
}

impl WorkerState {
    pub fn set_next_selection(&mut self) {
        if let Selection::Field(f) = self.selection {
            self.fields_states[f.index()].is_selected = false;
        };
        self.selection.set_next();
        if let Selection::Field(f) = self.selection {
            self.fields_states[f.index()].is_selected = true;
        };
    }

    pub fn set_previous_selection(&mut self) {
        if let Selection::Field(f) = self.selection {
            self.fields_states[f.index()].is_selected = false;
        }
        self.selection.set_previous();
        if let Selection::Field(f) = self.selection {
            self.fields_states[f.index()].is_selected = true;
        }
    }

    pub fn switch_field_editing(&mut self, field: FieldName) {
        let ind = field.index();
        self.fields_states[ind].is_editing = !self.fields_states[ind].is_editing;
    }

    pub fn get_cursor_position(&self) -> (u16, u16) {
        self.cursor_position
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
            WorkerVariant::Worker(_) => {
                let layout: [Rect; 5] = Layout::new(
                    layout::Direction::Vertical,
                    [
                        Constraint::Length((LOG_MAX + 2).try_into().unwrap()),
                        Constraint::Min((MESSAGES_MAX + 2).try_into().unwrap()),
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Length(3),
                    ],
                )
                .areas(area);

                let args_and_log_layout: [Rect; 2] = Layout::new(
                    layout::Direction::Horizontal,
                    [Constraint::Percentage(30), Constraint::Percentage(70)],
                )
                .areas(layout[0]);

                let names: [&str; 4] = [
                    " Logs ",
                    " Results ",
                    " Currently requesting ",
                    " Arguments ",
                ];

                Paragraph::new(Text::from_iter::<[Line; 5]>([
                    Line::from("URI: ") + state.fields_states[FieldName::Uri.index()].get().blue(),
                    Line::from("Threads: ")
                        + state.fields_states[FieldName::Threads.index()].get().blue(),
                    Line::from("Recursion depth: ")
                        + state.fields_states[FieldName::Recursion.index()]
                            .get()
                            .blue(),
                    Line::from("Timeout: ")
                        + state.fields_states[FieldName::Timeout.index()].get().blue(),
                    Line::from("Wordlist: ")
                        + state.fields_states[FieldName::WordlistPath.index()]
                            .get()
                            .blue(),
                ]))
                .block(Block::bordered().title(names[3]))
                .render(args_and_log_layout[0], buf);

                let log_lines = state.log.iter().map(|s| Line::from(s.as_str()));
                let message_lines = state.messages.iter().map(|s| Line::from(s.as_str()));

                Paragraph::new(Text::from_iter(log_lines))
                    .block(Block::bordered().title(names[0]))
                    .render(args_and_log_layout[1], buf);

                Paragraph::new(Text::from_iter(message_lines))
                    .block(Block::bordered().title(names[1]))
                    .render(layout[1], buf);

                Paragraph::new(Line::from(state.current_parsing.as_str()))
                    .block(Block::bordered().title(names[2]))
                    .render(layout[2], buf);

                if !state.fields_states[FieldName::Recursion.index()]
                    .get()
                    .starts_with('0')
                {
                    Gauge::default()
                        .block(Block::bordered().title(" Current recursion progress "))
                        .gauge_style(Style::new().white().on_black().italic())
                        .ratio(checked_ratio(
                            state.progress_current_now,
                            state.progress_current_total,
                        ))
                        .render(layout[3], buf);
                }

                Gauge::default()
                    .block(Block::bordered().title(" Total progress "))
                    .gauge_style(Style::new().blue().on_black().italic())
                    .ratio(checked_ratio(
                        state.progress_all_now,
                        state.progress_all_total,
                    ))
                    .render(layout[4], buf);
            }
            WorkerVariant::Builder => {
                let layout: [Rect; FIELDS_NUMBER + 1] = Layout::new(
                    layout::Direction::Vertical,
                    [
                        Constraint::Max(3),
                        Constraint::Max(3),
                        Constraint::Max(3),
                        Constraint::Max(3),
                        Constraint::Max(3),
                        Constraint::Max(3), 
                        Constraint::Max(7), 
                        Constraint::Max(5), // FOR BUTTON
                    ],
                )
                .areas(area);

                Paragraph::new("Run")
                    .centered()
                    .block(
                        Block::bordered().style(if state.selection == Selection::RunButton {
                            Style::default().green()
                        } else {
                            Style::default()
                        }),
                    )
                    .alignment(layout::Alignment::Center)
                    .render(
                        Self::center(layout[6], Constraint::Max(40), Constraint::Length(3)),
                        buf,
                    );

                for (ind, field_state) in state.fields_states.iter_mut().enumerate() {
                    if field_state.is_editing {
                        state.cursor_position = (
                            layout[ind].x + 1 + field_state.input.cursor() as u16,
                            layout[ind].y + 1,
                        );
                    }
                    Field::new(NAMES[ind]).render(layout[ind], buf, field_state);
                }
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

fn checked_ratio(a: usize, b: usize) -> f64 {
    let res = a as f64 / b as f64;
    if (0.0..=1.0).contains(&res) {
        return res;
    }
    0.0
}
