use std::collections::HashMap;

use anyhow::Result;
use rusqlite::Connection;
use serde::Serialize;

use crate::db::tf_idf;

#[derive(Serialize, Debug, PartialEq)]
pub struct Document {
    pub url: String,
    pub score: f64,
}

impl Document {
    pub fn new(url: &str, score: f64) -> Self {
        Self {
            url: url.to_string(),
            score,
        }
    }
}

pub fn execute(connection: &Connection, terms: &Vec<String>) -> Result<Vec<Document>> {
    // a map of (url, term) pairs to a score
    // each subsequent term adds a smaller amount
    // to the total score.
    let mut scored_results: HashMap<String, f64> = HashMap::new();

    for (i, t) in terms.iter().map(|t| t.to_lowercase()).enumerate() {
        let top = tf_idf::get_top_by_term(connection, &t, 100)?;
        for e in top {
            let key = e.url;
            if let Some(score) = scored_results.get_mut(&key) {
                let scale: f64 = 1.0 / i as f64;
                *score += e.score * scale;
                //scored_results.insert((e.url, e.term), score + (e.score * scale));
            } else {
                scored_results.insert(key, e.score);
            }
        }
    }

    let mut sorted_results: Vec<Document> = scored_results
        .into_iter()
        .map(|(url, score)| Document::new(&url, score))
        .collect();

    sorted_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    Ok(sorted_results)
}

pub fn search(connection: &Connection, terms: &Vec<String>) -> Result<()> {
    let results = execute(connection, terms)?;
    for document in results {
        println!("\t{}\t{}", document.url, document.score);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::db::tf_idf::TfIdfScore;

    #[test]
    fn test_execute() {
        let connection = Connection::open_in_memory().unwrap();
        db::initalize_tables(&connection).unwrap();
        let terms = vec![
            "Alpha".to_string(),
            "bravo".to_string(),
            "charlie".to_string(),
        ];
        let results = execute(&connection, &terms).unwrap();

        let expected: Vec<Document> = vec![];
        assert_eq!(expected, results);

        db::tf_idf::insert(
            &connection,
            &TfIdfScore::new("http://alpha.example.com", "alpha", 0.01),
        )
        .unwrap();
        db::tf_idf::insert(
            &connection,
            &TfIdfScore::new("http://alpha2.example.com", "alpha", 0.02),
        )
        .unwrap();

        let results = execute(&connection, &terms).unwrap();
        assert_eq!(
            vec![
                Document::new("http://alpha2.example.com", 0.02,),
                Document::new("http://alpha.example.com", 0.01,),
            ],
            results
        );
    }
}
