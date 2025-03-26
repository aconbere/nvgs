use std::fs;
use std::path::PathBuf;

use anyhow::{Result, anyhow};
use rusqlite::Connection;

use crate::db;

pub fn init(path: &PathBuf) -> Result<()> {
    if !path.exists() {
        fs::create_dir(path)?;
    }

    if !path.join("warcs").exists() {
        fs::create_dir(path.join("warcs"))?;
    }

    if path.join("nvgs.db").exists() {
        return Err(anyhow!(
            "Invalid path: {} - database exists alread.",
            path.display()
        ));
    }

    let connection = Connection::open(path.join("nvgs.db"))?;
    db::initalize_tables(&connection)?;

    Ok(())
}
