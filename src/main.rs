use std::{env, io::stdout};

use app::App;
use ratatui::{backend::CrosstermBackend, Terminal};

mod app;
mod client;
mod clip;
mod config;
mod macros;
mod results;
mod source;
mod sync;
mod theme;
mod util;
mod widget;

#[tokio::main()]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // Try to reset terminal on panic
        let _ = util::term::reset_terminal();
        default_panic(info);
    }));

    // TODO: Use real command line package
    let args: Vec<String> = env::args().collect();
    for arg in args {
        if arg == "--version" || arg == "-V" || arg == "-v" {
            println!("nyaa v{}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
    }
    util::term::setup_terminal()?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::default();

    app.run_app(&mut terminal).await?;

    util::term::reset_terminal()?;
    terminal.show_cursor()?;

    std::process::exit(0);
}
