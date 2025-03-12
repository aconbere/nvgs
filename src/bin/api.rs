use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use nvgs::api;

#[derive(Parser, Debug)]
#[command(name = "nvgs-api")]
#[command(author = "Anders Conbere<anders@conbere.org>")]
#[command(version = "0.1")]
#[command(about = "Not a Very Good Search engine", long_about = None)]
pub struct Cli {
    #[arg(long)]
    address: String,

    #[arg(long)]
    path: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    api::start(&cli.path, &cli.address).await?;
    Ok(())
}
