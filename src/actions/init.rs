use std::fs;
use std::path::PathBuf;

use anyhow::{Result, anyhow};
use rusqlite::Connection;

use crate::db;

pub fn init(path: &PathBuf) -> Result<()> {
    if path.exists() {
        return Err(anyhow!(
            "Invalid path: {} - Can't initialize an already existing directory.",
            path.display()
        ));
    }

    fs::create_dir(path)?;
    fs::create_dir(path.join("warcs"))?;

    let connection = Connection::open(path.join("nvgs.db"))?;
    db::crawls::create_table(&connection)?;
    db::term_frequencies::create_table(&connection)?;
    db::tf_idf::create_table(&connection)?;

    Ok(())
}
