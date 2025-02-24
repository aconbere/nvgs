use anyhow::Result;
use rusqlite::{Connection, params};

pub struct TfIdfScore {
    pub url: String,
    pub term: String,
    pub score: f64,
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
                url, term, COUNT(DISTINCT url) AS c
            FROM
                term_frequencies
            GROUP BY
                term
        ),
        scores as (
            SELECT
                tf.url,
                tf.term,
                -LOG10(
                    CAST(tdc.c AS FLOAT) / (SELECT CAST(c AS FLOAT) FROM document_count)
                )
            FROM
                term_frequencies AS tf
            JOIN
                term_document_count AS tdc
            ON
                tdc.url = tf.url AND
                tdc.term = tf.term
        )
        INSERT INTO
            tf_idf
        (url, term, score)
        SELECT * FROM scores
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
