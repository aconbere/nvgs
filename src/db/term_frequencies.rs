use anyhow::Result;
use rusqlite::{Connection, params};

#[derive(Debug, PartialEq)]
pub struct TermFrequency {
    pub url: String,
    pub term: String,
    pub count: u64,
    pub frequency: f64,
}

impl TermFrequency {
    pub fn new(url: &str, term: &str, count: u64, frequency: f64) -> Self {
        Self {
            url: url.to_string(),
            term: term.to_string(),
            count,
            frequency,
        }
    }
}

pub fn create_table(connection: &Connection) -> Result<()> {
    connection.execute(
        "CREATE TABLE term_frequencies (
            url String NOT NULL,
            term String NOT NULL,
            count INTEGER NOT NULL,
            frequency REAL NOT NULL,
            PRIMARY KEY (url, term)
        )",
        params![],
    )?;
    Ok(())
}

pub fn insert(connection: &Connection, entry: &TermFrequency) -> Result<()> {
    connection.execute(
        "INSERT INTO
            term_frequencies (
                url, term, count, frequency
            )
        VALUES
            (?1, ?2, ?3, ?4)
        ON CONFLICT
            (url, term)
        DO UPDATE
        SET
            count = ?3,
            frequency = ?4
        ",
        params![entry.url, entry.term, entry.count, entry.frequency],
    )?;
    Ok(())
}
