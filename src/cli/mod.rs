use std::path::PathBuf;

use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use rusqlite::Connection;

use crate::actions;

#[derive(Parser, Debug)]
#[command(name = "nvgs")]
#[command(author = "Anders Conbere<anders@conbere.org>")]
#[command(version = "0.1")]
#[command(about = "Not a Very Good Search engine", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    action: Action,

    #[arg(long)]
    path: PathBuf,
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
        #[arg(long, value_parser, num_args = 1.., value_delimiter = ' ')]
        query: Vec<String>,
    },
    Init,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    if let Action::Init = &cli.action {
        actions::init::init(&cli.path)?;
        return Ok(());
    }

    let db_path = cli.path.join("nvgs.db");
    let connection = Connection::open(db_path)?;

    match &cli.action {
        Action::Add { url } => actions::add::add(&connection, url),
        Action::Crawl => actions::crawl::crawl(&connection, &cli.path),
        Action::Index => actions::index::index(&connection),
        Action::Search { query } => actions::search::search(&connection, query),
        Action::Init => Err(anyhow!(
            "Should never get here, earlier check for init failed"
        )),
    }?;

    Ok(())
}
