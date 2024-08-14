use std::io::{self, stdout};

use crossterm::{
    cursor::SetCursorStyle,
    event::{DisableBracketedPaste, EnableBracketedPaste},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand as _,
};

#[cfg(unix)]
use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
#[cfg(unix)]
use ratatui::{backend::Backend, Terminal};
#[cfg(unix)]
use std::error::Error;

pub fn setup_terminal() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnableBracketedPaste)?;
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(SetCursorStyle::SteadyBar)?;
    Ok(())
}

pub fn reset_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    stdout().execute(SetCursorStyle::DefaultUserShape)?;
    stdout().execute(LeaveAlternateScreen)?;
    stdout().execute(DisableBracketedPaste)?;
    Ok(())
}

#[cfg(unix)]
pub fn suspend_self<B: Backend>(terminal: &mut Terminal<B>) -> Result<(), Box<dyn Error>> {
    // Make sure cursor is drawn

    terminal.draw(|f| f.set_cursor_position((0, 0)))?;

    reset_terminal()?;

    signal::kill(Pid::from_raw(std::process::id() as i32), Signal::SIGTSTP)?;
    Ok(())
}

#[cfg(unix)]
pub fn continue_self<B: Backend>(terminal: &mut Terminal<B>) -> Result<(), Box<dyn Error>> {
    setup_terminal()?;

    Terminal::clear(terminal)?;
    Ok(())
}
