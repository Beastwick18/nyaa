use app::App;

mod action;
mod app;
mod cli;
mod clients;
mod components;
mod config;
mod errors;
mod keys;
mod sources;
mod themes;
mod tui;

#[tokio::main()]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::read_args()?;

    let mut app = App::new(args)?;
    app.run().await?;

    std::process::exit(0);
}
