use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct LanceArgs {
    output_path: PathBuf,
}

impl LanceArgs {
    pub async fn run(&self) -> Result<()> {
        todo!()
    }
}
