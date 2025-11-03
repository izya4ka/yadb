use clipboard::{ClipboardContext, ClipboardProvider};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
};
use tui_input::InputRequest;
use std::{
    sync::mpsc::{self, Receiver}, thread::{self}, time::Duration
};

use crate::lib::{
    tui::widgets::{
        field::Field, popup::Popup, worker_info::{FieldType, Selection, WorkerInfo, WorkerState, WorkerVariant}
    },
    worker::{
        builder::{BuilderError, WorkerBuilder},
        messages::{ProgressMessage, WorkerMessage},
    },
};

pub const LOG_MAX: usize = 5;
pub const MESSAGES_MAX: usize = 20;

#[derive(Debug, Default, PartialEq)]
enum CurrentWindow {
    #[default]
    Workers,
    Info,
}

#[derive(Debug)]
enum WorkerType {
    Worker,
    Builder(WorkerBuilder),
}

#[derive(Debug)]
struct WorkerRx {
    worker_type: WorkerType,
    rx: Receiver<WorkerMessage>,
}

impl Default for WorkerRx {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel::<WorkerMessage>();

        Self {
            worker_type: WorkerType::Builder(WorkerBuilder::new().message_sender(tx.into())),
            rx,
        }
    }
}

#[derive(Debug, Default, PartialEq)]
enum InputMode {
    #[default]
    Normal,
    Editing
}

/// The main application which holds the state and logic of the application.
#[derive(Debug, Default)]
pub struct App {
    // Is the application running?
    running: bool,

    // Logic state
    current_window: CurrentWindow,
    workers_info_state: Vec<WorkerState>,
    workers: Vec<WorkerRx>,
    show_help_popup: bool,
    worker_list_state: ListState,
    builder_error: Option<BuilderError>,
    input_mode: InputMode
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;

            for (sel, worker_state) in self.workers.iter_mut().enumerate() {
                if let Ok(msg) = worker_state.rx.try_recv() {
                    match msg {
                        WorkerMessage::Progress(progress_message) => {
                            match progress_message {
                                ProgressMessage::Total(progress_change_message) => {
                                    match progress_change_message {
                                        crate::lib::worker::messages::ProgressChangeMessage::SetMessage(_) => {},
                                        crate::lib::worker::messages::ProgressChangeMessage::SetSize(size) => {
                                            self.workers_info_state[sel].progress_all_total = size;
                                        },
                                        crate::lib::worker::messages::ProgressChangeMessage::Start(_) => {},
                                        crate::lib::worker::messages::ProgressChangeMessage::Advance => {
                                            self.workers_info_state[sel].progress_all_now += 1;
                                        },
                                        crate::lib::worker::messages::ProgressChangeMessage::Print(_) => {},
                                        crate::lib::worker::messages::ProgressChangeMessage::Finish => {
                                            self.workers_info_state[sel].current_parsing = "Done!".to_string();
                                            self.workers_info_state[sel].worker = WorkerVariant::Worker(true);
                                        },
                                    }
                                },
                                ProgressMessage::Current(progress_change_message) => {
                                    match progress_change_message {
                                        crate::lib::worker::messages::ProgressChangeMessage::SetMessage(str) => {
                                            self.workers_info_state[sel].current_parsing = str;
                                        },
                                        crate::lib::worker::messages::ProgressChangeMessage::SetSize(size) => {
                                            self.workers_info_state[sel].progress_current_now = 0;
                                            self.workers_info_state[sel].progress_current_total = size;
                                        },
                                        crate::lib::worker::messages::ProgressChangeMessage::Start(_) => {},
                                        crate::lib::worker::messages::ProgressChangeMessage::Advance => {
                                            self.workers_info_state[sel].progress_current_now += 1;
                                        },
                                        crate::lib::worker::messages::ProgressChangeMessage::Print(msg) => {
                                            let messages = &mut self.workers_info_state[sel].messages;
                                            messages.push_back(msg);
                                            if messages.len() > MESSAGES_MAX {
                                                messages.pop_front();
                                            }
                                        },
                                        crate::lib::worker::messages::ProgressChangeMessage::Finish => {},
                                    }
                                },
                            }
                        },
                        WorkerMessage::Log(loglevel, str) => {
                            
                            let log = &mut self.workers_info_state[sel].log;
                            match loglevel {
                                crate::lib::logger::traits::LogLevel::WARN => log.push_front("[WARN] ".to_owned() + &str),
                                crate::lib::logger::traits::LogLevel::ERROR => log.push_front("[ERROR] ".to_owned() + &str),
                                crate::lib::logger::traits::LogLevel::CRITICAL => log.push_front("[CRITICAL]".to_owned() + &str),
                                _ => {},
                            }
                            if log.len() > LOG_MAX {
                                log.pop_front();
                            }
                        },
                    }
                }
            }
        }
        Ok(())
    }

    /// Renders the user interface.
    fn render(&mut self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Max(30), Constraint::Min(0)].as_ref())
            .split(frame.area());

        let rect_list = layout[0];
        let rect_info = layout[1];

        let workers_title = Line::from(" Workers ").centered();

        let info_title = Line::from(" Info ");

        let mut block_list = Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .title(workers_title);

        let mut block_info = Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .title(info_title);

        let help_line = Line::from(vec![" Help - ".into(), "<h> ".bold()]).centered();

        match self.current_window {
            CurrentWindow::Workers => {
                block_list = block_list.border_style(Style::new().blue());
                block_list = block_list.title_bottom(help_line);
            }
            CurrentWindow::Info => {
                block_info = block_info.border_style(Style::new().blue());
                block_info = block_info.title_bottom(help_line);
            }
        }

        let block_list_inner = block_list.inner(rect_list);
        let block_info_inner = block_info.inner(rect_info);

        frame.render_widget(block_list, rect_list);
        frame.render_widget(block_info, rect_info);

        let workers_name_list = self
            .workers_info_state
            .iter()
            .enumerate()
            .map(|(i, w)| {
                let mut cloned_name = w.name.clone();
                match self.workers_info_state[i].worker {
                    WorkerVariant::Worker(s) if !s => cloned_name = "<RUN> ".to_owned() + &cloned_name,
                    WorkerVariant::Worker(s) if s => cloned_name = "<DONE> ".to_owned() + &cloned_name,
                    WorkerVariant::Builder => cloned_name = "<WAIT> ".to_owned() + &cloned_name,
                    _ => {}
                };
                let mut item = ListItem::new(cloned_name);
                if let Some(selected_index) = self.worker_list_state.selected() && selected_index == i  {
                    item = item.reversed().blue();
                }
                item
            })
            .collect::<Vec<ListItem>>();
        let workers_list = List::new(workers_name_list);
        frame.render_stateful_widget(workers_list, block_list_inner, &mut self.worker_list_state);

        if let Some(sel) = self.worker_list_state.selected() {
            let worker_info = WorkerInfo {};
            let state = &mut self.workers_info_state[sel];
            frame.render_stateful_widget(worker_info, block_info_inner, state);

            if self.input_mode == InputMode::Editing {
                frame.set_cursor_position(state.get_cursor_position());
            }
        }

        if self.show_help_popup {
            self.render_help_popup(frame);
        }

        if let Some(err) = &self.builder_error {
            self.render_error_popup(frame, err.clone());
        }
    }

    /// Reads the crossterm events and updates the state of [`App`].
    fn handle_crossterm_events(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(40))? {
            match event::read()? {
                // it's important to check KeyEventKind::Press to avoid handling key release events
                Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {

        if (key.modifiers, key.code) == (KeyModifiers::CONTROL, KeyCode::Char('c')) {
            self.quit();
            return;
        };

        match self.input_mode {
            InputMode::Normal => self.handle_normal_input(key),
            InputMode::Editing => self.handle_editing_input(key),
        }
    }

    fn handle_normal_input(&mut self, key: KeyEvent) {
        match self.current_window {
            CurrentWindow::Workers => self.handle_workers_list_keys(key),
            CurrentWindow::Info => self.handle_worker_info_keys(key),
        }
    }

    fn handle_workers_list_keys(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Char('a')) => {
                self.workers_info_state.push(WorkerState::default());
                self.workers.push(WorkerRx::default());
                if self.worker_list_state.selected().is_none(){
                    self.worker_list_state.select(Some(0));
                }
            },
            (_, KeyCode::Down) => {
                if self.workers_info_state.is_empty() {
                    return;
                }
                if self.worker_list_state.selected() == Some(self.workers_info_state.len() - 1) {
                    self.worker_list_state.select_first();
                    return;
                }
                self.worker_list_state.select_next();
            }
            (_, KeyCode::Up) => {
                if self.workers_info_state.is_empty() {
                    return;
                }
                if self.worker_list_state.selected() == Some(0) {
                    self.worker_list_state.select_last();
                    return;
                }
                self.worker_list_state.select_previous();
            }
            (_, KeyCode::Char('d')) | (_, KeyCode::Delete) => {
                if let Some(sel) = self.worker_list_state.selected() {
                    self.workers_info_state.remove(sel);
                    self.workers.remove(sel);
                }
            },
            (_, KeyCode::Char('h')) => {
                self.show_help_popup = !self.show_help_popup;
            },
            (_, KeyCode::Right | KeyCode::Enter | KeyCode::Tab) => {
                if !self.workers_info_state.is_empty() {
                    self.switch_window()
                }
            },
            _ => {}
        }
    }

    fn handle_worker_info_keys(&mut self, key: KeyEvent) {
        if let Some(sel) = self.worker_list_state.selected() {
            let worker_state = &mut self.workers_info_state[sel];
            match (key.modifiers, key.code) {
                (_, KeyCode::Char('h')) => {
                    self.show_help_popup = !self.show_help_popup;
                },
                (_, KeyCode::Tab | KeyCode::Left) => self.switch_window(),
                (_, KeyCode::Down) => worker_state.set_next_selection(),
                (_, KeyCode::Up) => worker_state.set_previous_selection(),
                (_, KeyCode::Enter) => {
                    if self.builder_error.is_some() || self.show_help_popup {
                        self.close_all_popups();
                        return;
                    };

                    match worker_state.selection {
                        Selection::Field(field) => {
                            worker_state.switch_field_editing(field);
                            self.switch_input_mode();
                        },
                        Selection::RunButton => {
                            worker_state.do_build = true;
                        },
                    }
                }
                _ => {}
            };

            if self.workers_info_state[sel].do_build
                && let WorkerType::Builder(builder) = &mut self.workers[sel].worker_type {
                    let builder_clone = builder
                        .clone()
                        .recursive(
                            self.workers_info_state[sel]
                                .fields_states[FieldType::Recursion.index()]
                                .get()
                                .parse()
                                .unwrap(),
                        )
                        .threads(
                            self.workers_info_state[sel]
                                .fields_states[FieldType::Threads.index()]
                                .get()
                                .parse()
                                .unwrap(),
                        )
                        .timeout(
                            self.workers_info_state[sel]
                                .fields_states[FieldType::Timeout.index()]
                                .get()
                                .parse()
                                .unwrap(),
                        )
                        .uri(&self.workers_info_state[sel].fields_states[FieldType::Uri.index()]
                                .get())
                        .wordlist(&self.workers_info_state[sel].fields_states[FieldType::WordlistPath.index()]
                                .get());

                    let worker_result = builder_clone.build();
                    match worker_result {
                        Ok(worker) => {
                            self.workers[sel].worker_type = WorkerType::Worker;
                            thread::spawn(move || worker.run());
                            self.workers_info_state[sel].worker = WorkerVariant::Worker(false);
                        }
                        Err(err) => {
                            self.builder_error = Some(err.clone());
                            self.workers_info_state[sel].do_build = false;
                        }
                    }
                }
        }
    }
    fn handle_editing_input(&mut self, key: KeyEvent) {
        match self.current_window {
            CurrentWindow::Workers => todo!(),
            CurrentWindow::Info => {
                if let Some(sel) = self.worker_list_state.selected() {
                    let state = &mut self.workers_info_state[sel];
                    if let Selection::Field(f) = state.selection {
                        let field_state = &mut state.fields_states[f.index()];
                        match (key.modifiers, key.code) {
                            (_, KeyCode::Char(c)) => {
                                if field_state.is_only_numbers {
                                    if c.is_ascii_digit() && !field_state.get().starts_with('0') {
                                        field_state.input.handle(InputRequest::InsertChar(c));
                                    }
                                } else {
                                    field_state.input.handle(InputRequest::InsertChar(c));
                                }
                            },
                            (KeyModifiers::CONTROL, KeyCode::Right) => {
                                field_state.input.handle(InputRequest::GoToEnd);
                            },
                            (KeyModifiers::CONTROL, KeyCode::Left) => {
                                field_state.input.handle(InputRequest::GoToStart);
                            },
                            (_, KeyCode::Backspace) => {field_state.input.handle(InputRequest::DeletePrevChar);},
                            (_, KeyCode::Delete) => {field_state.input.handle(InputRequest::DeleteNextChar);},
                            (_, KeyCode::Left) => {
                                field_state.input.handle(InputRequest::GoToPrevChar);
                            },
                            (_, KeyCode::Right) => {
                                field_state.input.handle(InputRequest::GoToNextChar);
                            },
                            (_, KeyCode::Esc | KeyCode::Enter) => {
                                state.switch_field_editing(f);
                                self.switch_input_mode();
                            }
                            _ => {}
                        };
                    };
                }
            },
        };
    }

    fn switch_window(&mut self) {
        match self.current_window {
            CurrentWindow::Workers => self.current_window = CurrentWindow::Info,
            CurrentWindow::Info => self.current_window = CurrentWindow::Workers,
        }
    }

    fn render_help_popup(&mut self, frame: &mut Frame) {
        let help_message = match self.current_window {
            CurrentWindow::Workers => Text::from(vec![
                "<TAB> / <LEFT> / <RIGHT>".bold().blue() + " - Switch Tabs".into(),
                "<a>".bold().blue() + " - Add Worker".into(),
                "<d>".bold().blue() + " - Delete Worker".into(),
                "<Enter>".bold().blue() + " - Start/Stop worker".into(),
            ]),
            CurrentWindow::Info => Text::from(vec![
                " <TAB> / <LEFT> / <RIGHT>".bold().blue() + " - Switch tabs".into(),
                " <UP> / <DOWN>".bold().blue() + " - Move focus".into(),
                " <Enter>".bold().blue() + " - Edit property or press button".into(), 
            ]),
        };
        let popup = Popup::new(" Help ".to_string(), help_message);
        frame.render_widget(popup, frame.area());
    }

    fn render_error_popup(&mut self, frame: &mut Frame, err: BuilderError) {
            let error_message = Text::from(err.to_string());
            let popup = Popup::new(" Error ".to_string(), error_message);

            frame.render_widget(popup, frame.area());
    }

    fn switch_input_mode(&mut self) {
        match self.input_mode {
            InputMode::Normal => self.input_mode = InputMode::Editing,
            InputMode::Editing => self.input_mode = InputMode::Normal,
        }
    }

    fn close_all_popups(&mut self) {
        self.builder_error = None;
        self.show_help_popup = false;
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}

