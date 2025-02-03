use app::App;

mod action;
mod animate;
mod app;
mod cli;
mod clients;
mod color;
mod components;
mod config;
mod errors;
mod keys;
mod result;
mod sources;
mod themes;
mod tui;
mod widgets;

#[tokio::main()]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::read_args()?;

    let mut app = App::new(args)?;
    app.run().await?;

    std::process::exit(0);
}
