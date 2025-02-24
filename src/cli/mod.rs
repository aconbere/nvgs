use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::actions;

#[derive(Parser, Debug)]
#[command(name = "nvgs")]
#[command(author = "Anders Conbere<anders@conbere.org>")]
#[command(version = "0.1")]
#[command(about = "Not a Very Good Search engine", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    action: Action,
    db_path: String,
}

#[derive(Subcommand, Debug)]
pub enum Action {
    Add {
        #[arg(long)]
        url: String,
    },
    Crawl,
    Index,
    Search {
        #[arg(long)]
        query: String,
    },
    Init {
        #[arg(long)]
        path: PathBuf,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    if cli.Action == Action::Init {}

    match &cli.action {
        Add { url } => {}
        Crawl => {}
        Index => {}
        Search => {}
        Init => Err(anyhow!(
            "Should never get here, earlier check for init failed"
        )),
    }
}
