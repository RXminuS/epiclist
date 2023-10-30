#![feature(file_create_new)]

#[macro_use]
extern crate fstrings;

#[macro_use]
extern crate derive_builder;

#[macro_use]
extern crate lazy_static;

mod awesome_links;
mod commands;
mod github;
mod parser;

use anyhow::Result;
use clap::Parser;
use dotenv::dotenv;

#[derive(Debug, Parser)]
#[command(propagate_version = true)]
#[clap(name = "epiclist-scraper", version = "0.1.0")]
struct EpiclistScraperApp {
    #[command(subcommand)]
    command: commands::Command,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;
    let app = EpiclistScraperApp::parse();

    match app.command {
        commands::Command::Crawl(cmd) => cmd.run().await?,
        commands::Command::Ingest(cmd) => cmd.run().await?,
        commands::Command::Lance(cmd) => cmd.run().await?,
    };

    Ok(())
}
