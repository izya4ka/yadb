use std::{
    sync::mpsc::{self, Receiver},
    time::Duration,
};

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Text},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
};

use crate::lib::{
    tui::widgets::{popup::Popup, worker_info::{WorkerInfo, WorkerState, WorkerVariant}},
    worker::{builder::WorkerBuilder, messages::WorkerMessage, worker::Worker},
};

#[derive(Debug, Default, PartialEq)]
enum CurrentWindow {
    #[default]
    Workers,
    Info,
}

/// The main application which holds the state and logic of the application.
#[derive(Debug, Default)]
pub struct App {
    // Is the application running?
    running: bool,

    // Logic state
    current_window: CurrentWindow,
    workers_state: Vec<WorkerState>,
    show_help_popup: bool,
    worker_list_state: ListState,
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
        let block_info_inner = block_list.inner(rect_info);

        frame.render_widget(block_list, rect_list);
        frame.render_widget(block_info, rect_info);

        let workers_name_list = self.workers_state.iter()
            .enumerate()
            .map(|(i, w)| {
                let mut item = ListItem::new(w.name.clone());
                if let Some(selected_index) = self.worker_list_state.selected() {
                    if selected_index == i {
                        if self.current_window == CurrentWindow::Workers && self.is_editing {
                            item = item.italic().underlined();
                        } else {
                            item = item.reversed().blue();
                        }
                    }
                }
                item
            }).collect::<Vec<ListItem>>();
        let workers_list = List::new(workers_name_list);
        frame.render_stateful_widget(workers_list, block_list_inner, &mut self.worker_list_state);

        if let Some(sel) = self.worker_list_state.selected() {
            let worker_info = WorkerInfo{};
            let state = &mut self.workers_state[sel];
            frame.render_stateful_widget(worker_info, block_info_inner, state);
        }

        if self.show_help_popup {
            self.render_help_popup(frame);
        }
    }

    /// Reads the crossterm events and updates the state of [`App`].
    fn handle_crossterm_events(&mut self) -> Result<()> {
        // 90 FPS
        if event::poll(Duration::from_millis(12))? {
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
            },
            (_, KeyCode::Tab) => {
                match self.current_window {
                    CurrentWindow::Workers => self.current_window = CurrentWindow::Info,
                    CurrentWindow::Info => self.current_window = CurrentWindow::Workers,
                }
            },
            (_, KeyCode::Char('h')) => {
                if !self.is_editing {
                    self.show_help_popup = !self.show_help_popup
                }
            },
            _ =>
                match self.current_window {
                    CurrentWindow::Workers => self.handle_workers_list_keys(key),
                    CurrentWindow::Info => self.handle_worker_info_keys(key),
            },
        }
    }

    fn handle_workers_list_keys(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Char('a')) => {
                self.workers_state.push(WorkerState::default());
                if self.worker_list_state.selected() == None {
                    self.worker_list_state.select(Some(0));
                }
            },
            (_, KeyCode::Down) => {
                if self.workers_state.is_empty() {
                    return;
                }
                if self.worker_list_state.selected() == Some(self.workers_state.len() - 1) {
                    self.worker_list_state.select_first();
                    return;
                }
                self.worker_list_state.select_next();
            },
            (_, KeyCode::Up) => {
                if self.workers_state.is_empty() {
                    return;
                }
                if self.worker_list_state.selected() == Some(0) {
                    self.worker_list_state.select_last();
                    return;
                }
                self.worker_list_state.select_previous();
            },
            (_, KeyCode::Char('d')) | (_, KeyCode::Delete) => {
                if let Some(sel) = self.worker_list_state.selected() {
                    self.workers_state.remove(sel);
                }
            },
            _ => {}
        }
    }

    fn handle_worker_info_keys(&mut self, key: KeyEvent){
        if let Some(sel) = self.worker_list_state.selected() {
            if self.is_editing {
                self.workers_state[sel].handle_editing(key, &mut self.is_editing);
            } else {
                self.workers_state[sel].handle_keys(key, &mut self.is_editing);
            }
        }
    }
    
    fn render_help_popup(&mut self, frame: &mut Frame) {
        let help_message = match self.current_window {
                CurrentWindow::Workers => Text::from(vec![
                    Line::from(""),
                    Line::from("<TAB>".bold().blue() + " - Switch Tabs".into()),
                    Line::from("<a>".bold().blue() + " - Add Worker".into()),
                    Line::from("<d>".bold().blue() + " - Delete Worker".into()),
                    Line::from("<r>".bold().blue() + " - Rename Worker".into()),
                    Line::from("<Enter>".bold().blue() + " - Start/Stop worker".into()),
                ]),
                CurrentWindow::Info => Text::from(vec![
                    Line::from(""),
                    Line::from(" <TAB> ".bold().blue() + " - Switch tabs".into()),
                    Line::from(" (No other actions)"),
                ]),
            };
            let popup = Popup::new(" Help ".to_string(), help_message);
            frame.render_widget(popup, frame.area());
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}
