use std::path::PathBuf;

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
}

#[derive(Subcommand, Debug)]
pub enum Action {
    Fetch {
        #[arg(long)]
        url: String,

        #[arg(long)]
        output: PathBuf,
    },
    ReadWARC {
        #[arg(long)]
        input: PathBuf,
    },
    ExtractText {
        #[arg(long)]
        input: PathBuf,

        #[arg(long)]
        output: PathBuf,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match &cli.action {
        Action::Fetch { url, output } => actions::fetch::fetch(url, &output),
        Action::ReadWARC { input } => actions::read_warc::read_warc(&input),
        Action::ExtractText { input, output } => {
            actions::extract_text::extract_text(&input, &output)
        }
    }
}
