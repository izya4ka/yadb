use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame, layout::{Constraint, Direction, Layout, Rect}, style::{Style, Stylize}, text::Line, widgets::{Block, BorderType, Borders, Gauge, Paragraph, Widget}
};
use yadb::lib::{buster::Buster, progress_handler::traits::ProgressHandler};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}

#[derive(Debug)]
struct TuiProgress {
    percent: u16,
    label: String
}

impl ProgressHandler for TuiProgress {
    fn advance(&self) {
        
    }

    fn finish(&self) {
        
    }

    fn println(&self, str: String) {
        
    }

    fn set_message(&self, str: String) {
        
    }

    fn set_size(&self, size: usize) {
        
    }

    fn start(&self, total: usize) {
        
    }
}

impl Widget for TuiProgress {
    
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        
        let block = Block::default()
            .title("Прогресс Задачи")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
            
        let label_text = format!("{}% - {}", self.percent, self.label);
        
        let gauge = Gauge::default()
            .block(block) 
            .gauge_style(Style::default()) 
            .label(label_text)
            .ratio(self.percent as f64 / 100.0); 

        gauge.render(area, buf);
    }
}

impl TuiProgress {
    fn new() {

    }
}

#[derive(Debug)]
struct BusterInfo<T: ProgressHandler> {
    buster: Buster<T>,
    name: String
}

/// The main application which holds the state and logic of the application.
#[derive(Debug, Default)]
pub struct App {
    // Is the application running?
    running: bool,

    // State
    // busters: Vec<BusterInfo<TuiProgress<'a>>>

    
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
            .constraints([
                Constraint::Max(30),
                Constraint::Min(0),
            ].as_ref())
            .split(frame.area());

        let rect_list = layout[0];
        let rect_info = layout[1];

        let block_list = Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .title("AGENTS");

        let block_info = Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .title("INFO");

        let block_list_inner = block_list.inner(rect_list);
        let block_info_inner = block_list.inner(rect_info);

        

        frame.render_widget(block_list, rect_list);
        frame.render_widget(block_info, rect_info);
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            // Add other key handlers here.
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}