use anyhow::Result;
use clap::Parser;

mod app;
mod auth;
mod cli;
mod config;
mod db;
pub(crate) mod domain;
mod format;
mod tui;

fn main() -> Result<()> {
    let cli = cli::args::Cli::parse();
    let app = app::AppContext::new()?;
    cli::commands::dispatch(&app, cli)
}
