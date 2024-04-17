use std::{
    error::Error,
    io::{stdout, BufReader, Read as _},
    process::{Command, Stdio},
};

use crossterm::{
    cursor::SetCursorStyle,
    event::{
        DisableBracketedPaste, EnableBracketedPaste, KeyCode, KeyModifiers, MediaKeyCode,
        ModifierKeyCode,
    },
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand as _,
};
use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use ratatui::{backend::Backend, Terminal};
use regex::Regex;

pub fn setup_terminal() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    stdout().execute(EnableBracketedPaste)?;
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(SetCursorStyle::SteadyBar)?;
    Ok(())
}

pub fn reset_terminal() -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    stdout().execute(SetCursorStyle::DefaultUserShape)?;
    stdout().execute(LeaveAlternateScreen)?;
    stdout().execute(DisableBracketedPaste)?;
    Ok(())
}

#[cfg(unix)]
pub fn suspend_self<B: Backend>(terminal: &mut Terminal<B>) -> Result<(), Box<dyn Error>> {
    // Make sure cursor is drawn
    terminal.draw(|f| f.set_cursor(0, 0))?;

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

pub struct CommandBuilder {
    cmd: String,
}

impl CommandBuilder {
    pub fn new(cmd: String) -> Self {
        CommandBuilder { cmd }
    }

    pub fn sub(&mut self, pattern: &str, sub: &str) -> &mut Self {
        self.cmd = self.cmd.replace(pattern, sub);
        self
    }

    pub fn run<S: Into<Option<String>>>(&self, shell: S) -> Result<(), Box<dyn Error>> {
        let shell = Into::<Option<String>>::into(shell).unwrap_or(Self::default_shell());
        let cmds = shell.split_whitespace().collect::<Vec<&str>>();
        if let [base_cmd, args @ ..] = cmds.as_slice() {
            let cmd = Command::new(base_cmd)
                .args(args)
                .arg(&self.cmd)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn();

            let child = match cmd {
                Ok(child) => child,
                Err(e) => return Err(format!("{}:\nFailed to run:\n{}", self.cmd, e).into()),
            };
            let output = match child.wait_with_output() {
                Ok(output) => output,
                Err(e) => return Err(format!("{}:\nFailed to get output:\n{}", self.cmd, e).into()),
            };

            if output.status.code() != Some(0) {
                let mut err = BufReader::new(&*output.stderr);
                let mut err_str = String::new();
                err.read_to_string(&mut err_str).unwrap_or(0);
                return Err(format!(
                    "{}:\nExited with status code {}:\n{}",
                    self.cmd, output.status, err_str
                )
                .into());
            }
            Ok(())
        } else {
            Err(format!("Shell command is not properly formatted:\n{}", shell).into())
        }
    }

    pub fn default_shell() -> String {
        #[cfg(windows)]
        return "powershell.exe -Command".to_owned();
        #[cfg(unix)]
        return "sh -c".to_owned();
    }
}

pub fn add_protocol<S: Into<String>>(url: S, default_https: bool) -> String {
    let protocol = match default_https {
        true => "https",
        false => "http",
    };
    let url = url.into();
    let re = Regex::new(r"^https?://.+$").unwrap();
    match re.is_match(&url) {
        true => url,
        // Assume http(s) if not present
        false => format!("{}://{}", protocol, url),
    }
}

pub fn to_bytes(size: &str) -> usize {
    let mut split = size.split_whitespace();
    let b = split.next().unwrap_or("0");
    let unit = split.last().unwrap_or("B");
    let f = b.parse::<f64>().unwrap_or(0.0);
    let power = match unit.chars().next().unwrap_or('B') {
        'T' => 4,
        'G' => 3,
        'M' => 2,
        'K' => 1,
        _ => 0,
    };
    (1024_f64.powi(power) * f) as usize
}

pub fn shorten_number(n: u32) -> String {
    if n >= 10000 {
        format!("{}K", n / 1000)
    } else {
        n.to_string()
    }
}

pub fn key_to_string(key: KeyCode, modifier: KeyModifiers) -> String {
    let key = match key {
        KeyCode::Backspace => "BS".to_owned(),
        KeyCode::Enter => "CR".to_owned(),
        KeyCode::Left => "Left".to_owned(),
        KeyCode::Right => "Right".to_owned(),
        KeyCode::Up => "Up".to_owned(),
        KeyCode::Down => "Down".to_owned(),
        KeyCode::Home => "Home".to_owned(),
        KeyCode::End => "End".to_owned(),
        KeyCode::PageUp => "PgUp".to_owned(),
        KeyCode::PageDown => "PgDown".to_owned(),
        KeyCode::Tab | KeyCode::BackTab => "Tab".to_owned(),
        KeyCode::Delete => "Del".to_owned(),
        KeyCode::Insert => "Ins".to_owned(),
        KeyCode::F(f) => format!("F{}", f),
        KeyCode::Char(' ') => "Space".to_owned(),
        KeyCode::Char(c) => match modifier {
            KeyModifiers::NONE | KeyModifiers::SHIFT => return c.to_string(),
            _ => c.to_string(),
        },
        KeyCode::Esc => "Esc".to_owned(),
        KeyCode::Null => "Null".to_owned(),
        KeyCode::CapsLock => "CapsLock".to_owned(),
        KeyCode::ScrollLock => "ScrollLock".to_owned(),
        KeyCode::NumLock => "NumLock".to_owned(),
        KeyCode::PrintScreen => "Print".to_owned(),
        KeyCode::Pause => "Pause".to_owned(),
        KeyCode::Menu => "Menu".to_owned(),
        KeyCode::KeypadBegin => "Begin".to_owned(),
        KeyCode::Media(m) => match m {
            MediaKeyCode::Play => "MediaPlay".to_owned(),
            MediaKeyCode::Pause => "MediaPause".to_owned(),
            MediaKeyCode::PlayPause => "MediaPlayPause".to_owned(),
            MediaKeyCode::Reverse => "MediaReverse".to_owned(),
            MediaKeyCode::Stop => "MediaStop".to_owned(),
            MediaKeyCode::FastForward => "MediaFastForward".to_owned(),
            MediaKeyCode::Rewind => "MediaRewind".to_owned(),
            MediaKeyCode::TrackNext => "MediaTrackNext".to_owned(),
            MediaKeyCode::TrackPrevious => "MediaTrackPrevious".to_owned(),
            MediaKeyCode::Record => "MediaRecord".to_owned(),
            MediaKeyCode::LowerVolume => "MediaLowerVolume".to_owned(),
            MediaKeyCode::RaiseVolume => "MediaRaiseVolume".to_owned(),
            MediaKeyCode::MuteVolume => "MediaMuteVolume".to_owned(),
        },
        KeyCode::Modifier(m) => match m {
            ModifierKeyCode::LeftShift => "LeftShift".to_owned(),
            ModifierKeyCode::LeftControl => "LeftControl".to_owned(),
            ModifierKeyCode::LeftAlt => "LeftAlt".to_owned(),
            ModifierKeyCode::LeftSuper => "LeftSuper".to_owned(),
            ModifierKeyCode::LeftHyper => "LeftHyper".to_owned(),
            ModifierKeyCode::LeftMeta => "LeftMeta".to_owned(),
            ModifierKeyCode::RightShift => "RightShift".to_owned(),
            ModifierKeyCode::RightControl => "RightControl".to_owned(),
            ModifierKeyCode::RightAlt => "RightAlt".to_owned(),
            ModifierKeyCode::RightSuper => "RightSuper".to_owned(),
            ModifierKeyCode::RightHyper => "RightHyper".to_owned(),
            ModifierKeyCode::RightMeta => "RightMeta".to_owned(),
            ModifierKeyCode::IsoLevel3Shift => "IsoLevel3Shift".to_owned(),
            ModifierKeyCode::IsoLevel5Shift => "IsoLevel5Shift".to_owned(),
        },
    };
    let modifier = match modifier {
        KeyModifiers::CONTROL => "C-",
        KeyModifiers::SHIFT => "S-",
        KeyModifiers::ALT => "A-",
        KeyModifiers::SUPER => "U-",
        KeyModifiers::META => "M-",
        KeyModifiers::HYPER => "H-",
        _ => "",
    };
    format!("<{}{}>", modifier, key)
}
