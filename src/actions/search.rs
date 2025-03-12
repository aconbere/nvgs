use std::collections::HashMap;

use anyhow::Result;
use rusqlite::Connection;

use crate::db::tf_idf;

pub fn execute(connection: &Connection, terms: &Vec<String>) -> Result<Vec<(String, f64)>> {
    // a map of (url, term) pairs to a score
    // each subsequent term adds a smaller amount
    // to the total score.
    let mut scored_results: HashMap<String, f64> = HashMap::new();

    for (i, t) in terms.iter().enumerate() {
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

    let mut sorted_results: Vec<(String, f64)> = scored_results.into_iter().collect();
    sorted_results.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());

    Ok(sorted_results)
}

pub fn search(connection: &Connection, terms: &Vec<String>) -> Result<()> {
    let results = execute(connection, terms)?;
    for (url, score) in results {
        println!("\t{}\t{}", url, score);
    }

    Ok(())
}
