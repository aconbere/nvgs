use anyhow::Result;
use rusqlite::Connection;

use crate::db;

pub fn add(connection: &Connection, uri_str: &str) -> Result<()> {
    let e = db::index::Entry::new(uri_str)?;
    db::index::insert(connection, &e)?;
    Ok(())
}
