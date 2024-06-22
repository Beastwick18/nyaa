use std::{error::Error, io::stdout};

use app::App;
use config::{AppConfig, ConfigManager};
use ratatui::{backend::CrosstermBackend, Terminal};
use sync::AppSync;

pub mod app;
pub mod client;
pub mod clip;
pub mod config;
pub mod macros;
pub mod results;
pub mod source;
pub mod sync;
pub mod theme;
pub mod util;
pub mod widget;

struct Args {
    config_path: Option<String>,
}

fn parse_args() -> Result<Args, Box<dyn Error>> {
    use lexopt::prelude::*;

    let mut config_path = None;
    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Short('c') | Long("config") => {
                config_path = Some(shellexpand::full(&parser.value()?.string()?)?.to_string());
            }
            Short('v') | Short('V') | Long("version") => {
                println!("nyaa v{}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            Long("help") => {
                println!("Usage: nyaa [-v|-V|--version] [-c|--config=/path/to/config/folder]");
                std::process::exit(0);
            }
            _ => return Err(arg.unexpected().into()),
        }
    }

    Ok(Args { config_path })
}

#[tokio::main()]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // Try to reset terminal on panic
        let _ = util::term::reset_terminal();
        default_panic(info);
        std::process::exit(1);
    }));

    let args = parse_args()?;
    util::term::setup_terminal()?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::default();
    let config = match args.config_path {
        Some(path) => AppConfig::from_path(path),
        None => AppConfig::new(),
    }?;
    let sync = AppSync::new(config.path());

    app.run_app::<_, _, AppConfig, false>(&mut terminal, sync, config)
        .await?;

    util::term::reset_terminal()?;
    terminal.show_cursor()?;

    std::process::exit(0);
}
