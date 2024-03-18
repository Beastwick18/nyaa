use std::{env, io::stdout};

use app::{run_app, App};
use crossterm::{
    cursor::SetCursorStyle,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, Terminal};

mod app;
mod client;
mod config;
mod macros;
mod source;
mod widget;

#[tokio::main()]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Use real command line package
    let args: Vec<String> = env::args().collect();
    for arg in args {
        if arg == "--version" || arg == "-V" || arg == "-v" {
            println!(
                "nyaa v{}",
                option_env!("CARGO_PKG_VERSION").unwrap_or("UNKNOWN")
            );
            return Ok(());
        }
    }
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(SetCursorStyle::SteadyBar)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::default();

    run_app(&mut terminal, &mut app).await?;

    disable_raw_mode()?;
    stdout().execute(SetCursorStyle::DefaultUserShape)?;
    stdout().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
