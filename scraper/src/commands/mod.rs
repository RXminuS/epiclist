use clap::Subcommand;
use kdam::{term, term::Colorizer, tqdm, BarExt, Column, RichProgress, Spinner};

mod crawl;
mod ingest;
mod lance;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Start a broad crawl of awesome lists
    Crawl(crawl::CrawlArgs),

    /// Load crawled data into a database
    Ingest(ingest::IngestArgs),

    // Convert crawled data to a lance dataset
    Lance(lance::LanceArgs),
}
