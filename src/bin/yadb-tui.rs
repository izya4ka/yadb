use crossterm::cursor::SetCursorStyle;
use yadb::lib::tui::app::App;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    _ = crossterm::execute!(std::io::stdout(), SetCursorStyle::SteadyBar);
    let result = App::new().run(terminal);
    ratatui::restore();
    _ = crossterm::execute!(std::io::stdout(), SetCursorStyle::DefaultUserShape);
    result
}