use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
};
use std::{
    sync::mpsc::{self, Receiver}, thread::{self}, time::Duration
};

use crate::lib::{
    tui::widgets::{
        popup::Popup,
        worker_info::{WorkerInfo, WorkerState, WorkerVariant},
    },
    worker::{
        builder::{BuilderError, WorkerBuilder},
        messages::{ProgressMessage, WorkerMessage},
    },
};

const LOG_MAX: usize = 5;
const MESSAGES_MAX: usize = 30;

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
    is_editing: bool,
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
                                            self.workers_info_state[sel].progress_total = size;
                                        },
                                        crate::lib::worker::messages::ProgressChangeMessage::Start(_) => {},
                                        crate::lib::worker::messages::ProgressChangeMessage::Advance => {
                                            self.workers_info_state[sel].progress_current += 1;
                                        },
                                        crate::lib::worker::messages::ProgressChangeMessage::Print(_) => {},
                                        crate::lib::worker::messages::ProgressChangeMessage::Finish => {
                                            self.workers_info_state[sel].current_parsing = "Done!".to_string();
                                        },
                                    }
                                },
                                ProgressMessage::Current(progress_change_message) => {
                                    match progress_change_message {
                                        crate::lib::worker::messages::ProgressChangeMessage::SetMessage(str) => {
                                            self.workers_info_state[sel].current_parsing = str;
                                        },
                                        crate::lib::worker::messages::ProgressChangeMessage::SetSize(_) => {},
                                        crate::lib::worker::messages::ProgressChangeMessage::Start(_) => {},
                                        crate::lib::worker::messages::ProgressChangeMessage::Advance => {},
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
                    WorkerVariant::Worker => cloned_name = "<RUN> ".to_owned() + &cloned_name,
                    WorkerVariant::Builder => cloned_name = "<WAIT> ".to_owned() + &cloned_name,
                };
                let mut item = ListItem::new(cloned_name);
                if let Some(selected_index) = self.worker_list_state.selected() {
                    if selected_index == i {
                        item = item.reversed().blue();
                    }
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
        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => {
                if self.show_help_popup {
                    self.show_help_popup = false;
                } else if self.is_editing {
                    self.is_editing = false;
                } else {
                    self.quit()
                }
            }
            (_, KeyCode::Tab) | (_, KeyCode::Right) | (_, KeyCode::Left) => {
                if !self.is_editing {
                    self.switch_window();
                }
            },
            _ => {
                if (self.show_help_popup || self.builder_error.is_some()) && key.code == KeyCode::Enter {
                     {
                        self.show_help_popup = false;
                        self.builder_error = None;
                    }
                }

                match self.current_window {
                    CurrentWindow::Workers => self.handle_workers_list_keys(key),
                    CurrentWindow::Info => self.handle_worker_info_keys(key),
                }
            },
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
            }
            _ => {}
        }
    }

    fn handle_worker_info_keys(&mut self, key: KeyEvent) {
        if let Some(sel) = self.worker_list_state.selected() {
            if self.is_editing {
                self.workers_info_state[sel].handle_editing(key, &mut self.is_editing);
            } else {
                self.workers_info_state[sel].handle_keys(key, &mut self.is_editing);

                if self.workers_info_state[sel].do_build {
                    if let WorkerType::Builder(builder) = &mut self.workers[sel].worker_type {
                        let builder_clone = builder
                            .clone()
                            .recursive(
                                self.workers_info_state[sel]
                                    .properties
                                    .recursion
                                    .parse()
                                    .unwrap(),
                            )
                            .threads(
                                self.workers_info_state[sel]
                                    .properties
                                    .threads
                                    .parse()
                                    .unwrap(),
                            )
                            .timeout(
                                self.workers_info_state[sel]
                                    .properties
                                    .timeout
                                    .parse()
                                    .unwrap(),
                            )
                            .uri(&self.workers_info_state[sel].properties.uri)
                            .wordlist(&self.workers_info_state[sel].properties.wordlist_path);

                        let worker_result = builder_clone.build();
                        match worker_result {
                            Ok(worker) => {
                                self.workers[sel].worker_type = WorkerType::Worker;
                                thread::spawn(move || worker.run());
                                self.workers_info_state[sel].worker = WorkerVariant::Worker;
                                self.is_editing = false;
                            }
                            Err(err) => {
                                self.builder_error = Some(err.clone());
                                self.workers_info_state[sel].do_build = false;
                            }
                        }
                    }
                }
            }
        }
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

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}
