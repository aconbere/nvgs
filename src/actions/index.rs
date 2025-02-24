use anyhow::Result;
use rusqlite::Connection;

use crate::db::tf_idf;

pub fn index(connection: &Connection) -> Result<()> {
    tf_idf::index(connection)?;
    Ok(())
}
