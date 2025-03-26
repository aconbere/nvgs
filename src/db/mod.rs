use anyhow::Result;
use rusqlite::Connection;

pub mod crawls;
pub mod term_frequencies;
pub mod tf_idf;
pub mod users;

pub fn initalize_tables(connection: &Connection) -> Result<()> {
    crawls::create_table(&connection)?;
    term_frequencies::create_table(&connection)?;
    tf_idf::create_table(&connection)?;
    users::create_table(&connection)?;
    Ok(())
}
