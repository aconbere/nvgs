use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use another_rust_warc::header::FieldNames;
use another_rust_warc::reader::Reader;
use anyhow::{Result, anyhow};

pub fn read_warc(input: &Path) -> Result<()> {
    let input = File::open(input)?;
    let mut buffer = BufReader::new(input);

    let reader = Reader::new(&mut buffer);

    for maybe_record in reader {
        match maybe_record {
            Ok(record) => {
                let record_id = record
                    .header
                    .get(&FieldNames::RecordID)
                    .ok_or(anyhow!("No record id"))?;
                println!("Record: {}", record_id);
                let content = String::from_utf8_lossy_owned(record.content);
                println!("{}", content);
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }

    Ok(())
}
