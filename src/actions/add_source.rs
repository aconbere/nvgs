use anyhow::Result;
use rusqlite::Connection;

use crate::db::sources;

pub fn add_source(connection: &Connection, url: &str) -> Result<()> {
    let source = sources::Source::new(url)?;
    sources::insert(connection, &source)?;

    Ok(())
}
