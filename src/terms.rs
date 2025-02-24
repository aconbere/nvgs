use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};

use anyhow::Result;

pub struct TermCounts {
    pub counts: HashMap<String, u64>,
    pub total: u64,
}

pub fn counts(body: &mut dyn Read) -> Result<TermCounts> {
    let lines = BufReader::new(body).lines();

    let mut counts = HashMap::new();
    let mut total = 0;

    for line in lines {
        let l = line?;
        let toks = l.split_whitespace();
        for s in toks {
            if let Some(count) = counts.get(s) {
                counts.insert(s.to_string(), count + 1);
            } else {
                total += 1;
                counts.insert(s.to_string(), 1);
            }
        }
    }

    Ok(TermCounts { counts, total })
}

pub fn frequencies(term_counts: &TermCounts) -> HashMap<String, f64> {
    let mut frequencies = HashMap::new();
    for (k, v) in term_counts.counts.iter() {
        let v = *v as f64;
        let total = term_counts.total as f64;
        frequencies.insert(k.to_string(), v / total);
    }
    frequencies
}
