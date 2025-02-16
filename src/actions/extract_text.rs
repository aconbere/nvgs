use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use another_rust_warc::header::{FieldNames, RecordTypes};
use another_rust_warc::reader::Reader;
use anyhow::{Result, anyhow};
use scraper::Html;

pub fn extract_text(input: &Path, _output: &Path) -> Result<()> {
    let input = File::open(input)?;
    let mut buffer = BufReader::new(input);

    let reader = Reader::new(&mut buffer);

    for maybe_record in reader {
        match maybe_record {
            Ok(record) => {
                let record_type = record
                    .header
                    .get(&FieldNames::Type)
                    .ok_or(anyhow!("No record type"))?;
                let record_type = match RecordTypes::from_string(record_type) {
                    Ok(rt) => rt,
                    Err(e) => return Err(anyhow!("invalid record type: {}", e)),
                };

                if record_type == RecordTypes::Response {
                    let content_type = record
                        .header
                        .get(&FieldNames::ContentType)
                        .ok_or(anyhow!("No content_type"))?;
                    match content_type.as_str() {
                        "text/plain" => {
                            let content = String::from_utf8_lossy_owned(record.content);
                            println!("{}", content);
                        }
                        "text/html" => {
                            let content = String::from_utf8_lossy_owned(record.content);
                            let parsed_document = Html::parse_document(&content);
                            let extracted_text: Vec<&str> =
                                parsed_document.root_element().text().collect();
                            for e in extracted_text {
                                let e = e.trim();
                                if e.is_empty() {
                                    continue;
                                }
                                println!("s:{}", e);
                            }
                        }
                        _ => continue,
                    }
                }
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }
    Ok(())
}
