use anyhow::Result;
use chrono::Utc;
use reqwest::Url;
use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;

/* Represents a remote source that we fetch urls to index from
 */
#[derive(Serialize)]
pub struct Source {
    pub url: String,
    pub last_updated: i64,
}

impl Source {
    pub fn new(url_str: &str) -> Result<Self> {
        let url = Url::parse(url_str)?;
        Ok(Self {
            url: url.into(),
            last_updated: -1,
        })
    }

    pub fn get_updated(&self) -> Source {
        let now = Utc::now().timestamp();
        Source {
            url: self.url.clone(),
            last_updated: now,
        }
    }
}

pub fn create_table(connection: &Connection) -> Result<()> {
    connection.execute(
        "CREATE TABLE sources (
            url STRING NOT NULL,
            last_updated INTEGER NOT NULL,
            PRIMARY KEY (url)
        )",
        params![],
    )?;
    Ok(())
}

pub fn insert(connection: &Connection, source: &Source) -> Result<()> {
    connection.execute(
        "INSERT INTO
            sources (
                url, last_updated
            )
        VALUES
            (?1, ?2)
        ON CONFLICT
            (url)
        DO UPDATE
        SET
            last_updated = ?3
        ",
        params![source.url, source.last_updated],
    )?;
    Ok(())
}

pub fn get(connection: &Connection, url: &str) -> Result<Option<Source>> {
    let mut statement = connection.prepare(
        "SELECT
            url, last_updated
        FROM
            sources
        WHERE
            url = ?1
        LIMIT
            1
        ",
    )?;

    let result: Option<Source> = statement
        .query_row(params![url], |row| {
            Ok(Source {
                url: row.get(0)?,
                last_updated: row.get(1)?,
            })
        })
        .optional()?;
    Ok(result)
}

pub fn delete(connection: &Connection, url: &str, source: &str) -> Result<()> {
    let mut statement = connection.prepare(
        "DELETE
        FROM
            crawls
        WHERE
            url = ?1 AND source = ?2
        LIMIT
            1
        ",
    )?;

    statement.execute(params![url, source])?;
    Ok(())
}

pub fn delete_from_source(connection: &Connection, source: &str) -> Result<()> {
    let mut statement = connection.prepare(
        "DELETE
        FROM
            crawls
        WHERE
            source = ?1
        LIMIT
            1
        ",
    )?;

    statement.execute(params![source])?;
    Ok(())
}

pub fn get_all(connection: &Connection) -> Result<Vec<Source>> {
    let mut statement = connection.prepare(
        "SELECT
            url, last_updated
        FROM
            sources
        ",
    )?;

    let results: Vec<Source> = statement
        .query_map(params![], |row| {
            Ok(Source {
                url: row.get(0)?,
                last_updated: row.get(1)?,
            })
        })?
        .into_iter()
        .flatten()
        .collect();
    Ok(results)
}
