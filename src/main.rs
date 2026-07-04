use std::io::stdout;

use color_eyre::Result;

mod app;
mod config;
mod fs;
mod image;
mod input;
mod ui;

fn main() -> Result<()> {
    color_eyre::install()?;
    crossterm::execute!(stdout(), crossterm::event::EnableMouseCapture)?;

    let mut terminal = ratatui::init();
    let mut app = app::app::App::new()?;
    let result = app.run(&mut terminal);

    let _ = crossterm::execute!(stdout(), crossterm::event::DisableMouseCapture);
    ratatui::restore();
    result
}
