use std::error::Error;

#[derive(Default)]
pub struct Args {
    pub config_path: Option<String>,
    pub debug_info: Option<String>,
}

static HELP_MSG: &str = "\
A TUI for browsing and downloading torrents

Usage:
  nyaa --config=<config_path>
  nyaa --help
  nyaa --version

Options:
-h --help        Show this screen
-v -V --version  Show version
-c --config      Set the directory to look for config files [default: \"~/.config/nyaa\"]";

pub fn read_args() -> Result<Args, Box<dyn Error>> {
    use lexopt::prelude::*;

    let mut args = Args::default();
    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Short('c') | Long("config") => {
                let parser_value = parser.value().expect("Failed to parse cli value");

                let string_value = parser_value
                    .string()
                    .expect("Parsed value was not a valid string");

                let expanded_value = shellexpand::full(&string_value)
                    .expect("Failed to expand values within string parsed string");

                args.config_path = Some(expanded_value.to_string());
            }
            Short('v') | Short('V') | Long("version") => {
                println!("nyaa v{}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            Short('h') | Long("help") => {
                println!("{}", HELP_MSG);
                std::process::exit(0);
            }
            _ => return Err(arg.unexpected().into()),
        }
    }

    Ok(args)
}
