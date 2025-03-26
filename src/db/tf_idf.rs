use anyhow::Result;
use rusqlite::{Connection, params};

pub struct TfIdfScore {
    pub url: String,
    #[allow(dead_code)]
    pub term: String,
    pub score: f64,
}

impl TfIdfScore {
    pub fn new(url: &str, term: &str, score: f64) -> Self {
        Self {
            url: url.to_string(),
            term: term.to_string(),
            score,
        }
    }
}

pub fn create_table(connection: &Connection) -> Result<()> {
    connection.execute(
        "
        CREATE TABLE tf_idf (
            url STRING NOT NULL,
            term STRING NOT NULL,
            score REAL NOT NULL,
            PRIMARY KEY (url, term)
        )
        ",
        params![],
    )?;
    Ok(())
}

pub fn insert(connection: &Connection, item: &TfIdfScore) -> Result<()> {
    connection.execute(
        "INSERT INTO
            tf_idf (
                url, term, score
            )
        VALUES
            (?1, ?2, ?3)
        ON CONFLICT
            (url, term)
        DO UPDATE
        SET
            term = ?2,
            score = ?3
        ",
        params![item.url, item.term, item.score],
    )?;
    Ok(())
}

pub fn index(connection: &Connection) -> Result<()> {
    connection.execute(
        "
        WITH document_count AS (
            SELECT
                COUNT(DISTINCT url) AS c
            FROM
                term_frequencies
        ),
        term_document_count AS (
            SELECT
                term, COUNT(DISTINCT url) AS c
            FROM
                term_frequencies
            GROUP BY
                term
        ),
        scores as (
            SELECT
                tf.url,
                tf.term,
                tf.frequency * -LOG10(
                    CAST(tdc.c AS FLOAT) / (SELECT CAST(c AS FLOAT) FROM document_count)
                )
            FROM
                term_frequencies AS tf
            JOIN
                term_document_count AS tdc
            ON
                tdc.term = tf.term
        )
        INSERT INTO
            tf_idf (
                url, term, score
            )
        SELECT * FROM scores WHERE true
        ON CONFLICT
            (url, term)
        DO UPDATE
        SET
            score = excluded.score
        ",
        params![],
    )?;
    Ok(())
}

pub fn get_top_by_term(connection: &Connection, term: &str, limit: u64) -> Result<Vec<TfIdfScore>> {
    let mut statement = connection.prepare(
        "
        SELECT
            term, url, score
        FROM
            tf_idf
        WHERE
            term = ?1
        ORDER BY
            score DESC
        LIMIT
            ?2
        ",
    )?;

    let results: Vec<TfIdfScore> = statement
        .query_map(params![term, limit], |row| {
            Ok(TfIdfScore {
                term: row.get(0)?,
                url: row.get(1)?,
                score: row.get(2)?,
            })
        })?
        .into_iter()
        .flatten()
        .collect();
    Ok(results)
}
