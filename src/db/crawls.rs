use anyhow::Result;
use chrono::{TimeDelta, Utc};
use reqwest::Url;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;

#[derive(Serialize)]
pub enum Status {
    Ready,
    Crawling,
}

impl TryFrom<&str> for Status {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, std::string::String> {
        match value {
            "ready" => Ok(Status::Ready),
            "crawling" => Ok(Status::Crawling),
            _ => Err(format!("Invalid status: {}", value)),
        }
    }
}

impl Into<String> for &Status {
    fn into(self) -> String {
        match self {
            Status::Ready => "ready".to_string(),
            Status::Crawling => "crawling".to_string(),
        }
    }
}

impl FromSql for Status {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().and_then(|s| match Status::try_from(s) {
            Ok(rk) => Ok(rk),
            Err(_) => Err(FromSqlError::InvalidType),
        })
    }
}

impl ToSql for Status {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let o: String = self.into();
        Ok(ToSqlOutput::from(o))
    }
}

#[derive(Serialize)]
pub struct Crawl {
    pub url: String,
    pub status: Status,
    pub last_updated: i64,
}

impl Crawl {
    pub fn new(url_str: &str) -> Result<Self> {
        let url = Url::parse(url_str)?;
        Ok(Self {
            url: url.into(),
            status: Status::Ready,
            last_updated: -1,
        })
    }
}

pub fn create_table(connection: &Connection) -> Result<()> {
    connection.execute(
        "CREATE TABLE crawls (
            url STRING NOT NULL,
            status STRING NOT NULL,
            last_updated INTEGER NOT NULL,
            PRIMARY KEY (url)
        )",
        params![],
    )?;
    Ok(())
}

pub fn insert(connection: &Connection, crawl: &Crawl) -> Result<()> {
    connection.execute(
        "INSERT INTO
            crawls (
                url, status, last_updated
            )
        VALUES
            (?1, ?2, ?3)
        ON CONFLICT
            (url)
        DO UPDATE
        SET
            status = ?2,
            last_updated = ?3
        ",
        params![crawl.url, crawl.status, crawl.last_updated],
    )?;
    Ok(())
}

pub fn get(connection: &Connection, url: &str) -> Result<Option<Crawl>> {
    let mut statement = connection.prepare(
        "SELECT
            url, status, last_updated
        FROM
            crawls
        WHERE
            url = ?1
        LIMIT
            1
        ",
    )?;

    let result: Option<Crawl> = statement
        .query_row(params![url], |row| {
            Ok(Crawl {
                url: row.get(0)?,
                status: row.get(1)?,
                last_updated: row.get(2)?,
            })
        })
        .optional()?;
    Ok(result)
}

pub fn delete(connection: &Connection, url: &str) -> Result<()> {
    let mut statement = connection.prepare(
        "DELETE
        FROM
            crawls
        WHERE
            url = ?1
        LIMIT
            1
        ",
    )?;

    statement.execute(params![url])?;
    Ok(())
}

pub fn get_all_with_status_since(
    connection: &Connection,
    status: &Status,
    since: &TimeDelta,
) -> Result<Vec<Crawl>> {
    let mut statement = connection.prepare(
        "SELECT
            url, status, last_updated
        FROM
            crawls
        WHERE
            status = ?1 AND
            last_updated < ?2
        ",
    )?;

    let last_updated = (Utc::now() - *since).timestamp();

    let result: Vec<Crawl> = statement
        .query_map(params![status, last_updated], |row| {
            Ok(Crawl {
                url: row.get(0)?,
                status: row.get(1)?,
                last_updated: row.get(2)?,
            })
        })?
        .into_iter()
        .flatten()
        .collect();

    Ok(result)
}

pub fn get_all_needing_update(connection: &Connection) -> Result<Vec<Crawl>> {
    get_all_with_status_since(connection, &Status::Ready, &TimeDelta::minutes(3))
}

pub fn set_crawling(connection: &Connection, url: &str) -> Result<()> {
    connection.execute(
        "UPDATE
            crawls
        SET
            status = ?2
        WHERE
            url = ?1
        ",
        params![url, Status::Crawling],
    )?;
    Ok(())
}

pub fn set_ready(connection: &Connection, url: &str, updated_at: i64) -> Result<()> {
    connection.execute(
        "UPDATE
            crawls
        SET
            status = ?2,
            last_updated = ?3
        WHERE
            url = ?1
        ",
        params![url, Status::Ready, updated_at],
    )?;
    Ok(())
}
