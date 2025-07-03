use anyhow::Result;
use rusqlite::Connection;

use crate::db;

pub fn add_url(connection: &Connection, uri_str: &str) -> Result<()> {
    let e = db::crawls::Crawl::new(uri_str)?;
    db::crawls::insert(connection, &e)?;
    Ok(())
}
