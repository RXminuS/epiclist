use clap::Subcommand;

mod crawl;
mod ingest;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Start a broad crawl of awesome lists
    Crawl(crawl::CrawlArgs),

    /// Load crawled data into a database
    Ingest(ingest::IngestArgs),
}
