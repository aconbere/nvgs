use std::collections::HashMap;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

use another_rust_warc::header::{FieldNames, Header, RecordID, RecordTypes};
use another_rust_warc::reader::{Reader, find_record_by_type};
use another_rust_warc::record::Record;
use another_rust_warc::writer::write_record;
use anyhow::{Result, anyhow};
use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use chrono::Utc;
use chrono::format::SecondsFormat;
use reqwest::blocking::Client;
use reqwest::header::{CONTENT_TYPE, USER_AGENT};
use rusqlite::Connection;
use scraper::Html;

use crate::db::{crawls, term_frequencies};

static USER_AGENT_STR: &str = "nvgs/1.0";

fn encode_url(url: &str) -> String {
    URL_SAFE.encode(url)
}

pub fn crawl(connection: &mut Connection, path: &PathBuf) -> Result<()> {
    let client = Client::new();
    let entries = crawls::get_all_needing_update(connection)?;

    println!("Crawling {} pages", entries.len());

    for e in entries {
        crawl_one(connection, path, &client, &e.url)?;
    }
    Ok(())
}

pub fn crawl_one(
    connection: &mut Connection,
    path: &PathBuf,
    client: &Client,
    url: &str,
) -> Result<()> {
    crawls::set_crawling(connection, url)?;

    let result: Result<()> = try {
        let request = client.get(url).header(USER_AGENT, USER_AGENT_STR).build()?;

        println!("Fetching {}", url);
        let mut response =
            client.execute(request.try_clone().ok_or(anyhow!("could not clone body"))?)?;

        let encoded_url = encode_url(url);

        /* We stream the result body into the warc file later we will read the file again to create
         * a wat file the benefit will be that we can read through the stream twice without ever
         * having the whole file in memory
         */
        println!("Writing warc...");
        let warc_path = path.join("warcs").join(format!("{}.warc", encoded_url));
        let mut warc_file = OpenOptions::new()
            .write(true)
            .append(false)
            .read(true)
            .create(true)
            .open(warc_path)?;
        write_request_record(&mut warc_file, &request)?;
        write_response_record(&mut warc_file, &mut response)?;
        warc_file.seek(SeekFrom::Start(0))?;

        /* We stream the wet file similarly, we will then take the text and process the counts
         * and frequencies
         */
        println!("Writing wet...");
        let wet_path = path.join("warcs").join(format!("{}.wet", encoded_url));
        let mut wet_file = OpenOptions::new()
            .write(true)
            .append(false)
            .read(true)
            .create(true)
            .open(wet_path)?;
        write_wat_record(&warc_file, &mut wet_file)?;
        wet_file.seek(SeekFrom::Start(0))?;

        println!("Analyzing terms...");
        let terms = analyze_terms(&mut wet_file, url)?;

        let tx = connection.transaction()?;
        println!("Updating term frequencies...");
        for t in terms {
            term_frequencies::insert(&tx, &t)?;
        }
        tx.commit()?;

        let now = Utc::now().timestamp();
        crawls::set_ready(connection, url, now)?;
    };

    match result {
        Ok(_) => {
            let now = Utc::now().timestamp();
            crawls::set_ready(connection, url, now)?;
        }
        Err(e) => {
            crawls::set_ready(connection, url, 0)?;
            println!("Failed to fetch: {}", url);
            println!("\tError:{}", e);
        }
    }

    Ok(())
}

fn write_request_record(
    writer: &mut dyn Write,
    request: &reqwest::blocking::Request,
) -> Result<()> {
    let date = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let request_string = request_to_string(&request)?;
    let content_length = request_string.len() as u64;

    let mut header = Header::new();
    header.insert(FieldNames::RecordID, RecordID::new().to_string());
    header.insert(FieldNames::Type, RecordTypes::Request.to_string());
    header.insert(FieldNames::Date, date);
    header.insert(FieldNames::IPAddress, "127.0.0.1".to_string());
    header.insert(FieldNames::ContentLength, request_string.len().to_string());

    let record = Record::new(header, content_length);

    let mut body = request_string.as_bytes();

    write_record(writer, &record, &mut body)?;
    Ok(())
}

struct ContentType {
    mime_type: String,
    charset: String,
    boundary: String,
}

impl ContentType {
    pub fn from_response(response: &reqwest::blocking::Response) -> ContentType {
        let content_type = match response.headers().get(CONTENT_TYPE) {
            Some(hv) => hv.to_str().unwrap_or("application/octect-stream"),
            None => "application/octet-stream",
        }
        .to_string();
        Self::from_string(&content_type)
    }

    pub fn from_string(input: &str) -> ContentType {
        let mime_type = match input.split_once(";") {
            Some((mime, _rest)) => mime.to_string(),
            None => input.to_string(),
        };

        let charset = "".to_string();
        let boundary = "".to_string();

        ContentType {
            mime_type,
            charset,
            boundary,
        }
    }
}

fn write_response_record(
    writer: &mut dyn Write,
    response: &mut reqwest::blocking::Response,
) -> Result<()> {
    let date = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);

    // Note sometimes there is no content length header and I'll need to read the whole body in and
    // write a record. Maybe need to bifurcate the code here since trying to do this generically is
    // hard.
    let content_length = response
        .content_length()
        .ok_or(anyhow!("no valid content length header on response"))?;

    let remote_addr = response
        .remote_addr()
        .ok_or(anyhow!("no valid remote address header in response"))?
        .to_string();

    let content_type = ContentType::from_response(response);

    let mut header = Header::new();
    header.insert(FieldNames::RecordID, RecordID::new().to_string());
    header.insert(FieldNames::Type, RecordTypes::Response.to_string());
    header.insert(FieldNames::Date, date);
    header.insert(FieldNames::IPAddress, remote_addr);
    header.insert(FieldNames::ContentType, content_type.mime_type);
    header.insert(FieldNames::ContentLength, content_length.to_string());
    let record = Record::new(header, content_length);

    write_record(writer, &record, response)?;

    Ok(())
}

pub fn request_to_string(request: &reqwest::blocking::Request) -> Result<String> {
    let mut output = String::new();

    let method = request.method();
    let url = request.url();
    let path = url.path();
    let version = request.version();

    fmt::write(
        &mut output,
        format_args!("{} {} {:?}\n", method.as_str(), path, version),
    )?;

    for (k, v) in request.headers().into_iter() {
        fmt::write(&mut output, format_args!("{}: {}", k.as_str(), v.to_str()?))?;
    }

    Ok(output)
}

pub fn write_wat_record(warc_file: &File, writer: &mut dyn Write) -> Result<()> {
    let mut reader = Reader::new(BufReader::new(warc_file));

    let record = find_record_by_type(&mut reader, RecordTypes::Response)?
        .ok_or(anyhow!("no response record found"))?;

    let content_type = record
        .header
        .get(&FieldNames::ContentType)
        .ok_or(anyhow!("No content_type"))?;

    match content_type.as_str() {
        "text/plain" => {
            let body = String::from_utf8_lossy_owned(record.content);
            writer.write_all(body.as_bytes())?;
        }
        "text/html" => {
            let body = String::from_utf8_lossy_owned(record.content);
            let document = Html::parse_document(&body);

            for text in document.root_element().text() {
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    continue;
                }
                for word in text.split_whitespace() {
                    let t = word.trim();
                    if t.is_empty() {
                        continue;
                    }
                    write!(writer, "{} ", word)?;
                }
                write!(writer, "\n")?;
            }
        }
        _ => {
            return Err(anyhow!("Cannot process files of type: {}", content_type));
        }
    };

    Ok(())
}

pub fn analyze_terms(
    reader: &mut dyn Read,
    url: &str,
) -> Result<Vec<term_frequencies::TermFrequency>> {
    let lines = BufReader::new(reader).lines();

    let mut terms: HashMap<String, term_frequencies::TermFrequency> = HashMap::new();
    let mut total = 0;

    for line in lines {
        let l = line?;
        for word in l.split_whitespace() {
            let w = word.to_lowercase();
            total += 1;
            if let Some(tf) = terms.get_mut(&w) {
                tf.count += 1;
            } else {
                terms.insert(w.clone(), term_frequencies::TermFrequency {
                    term: w,
                    count: 1,
                    frequency: 0.0,
                    url: url.to_string(),
                });
            }
        }
    }

    for (_, v) in terms.iter_mut() {
        v.frequency = v.count as f64 / total as f64;
    }

    let result: Vec<term_frequencies::TermFrequency> = terms.into_values().collect();
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_type() {
        let test = "text/html; charset=UTF-8".to_string();
        let result = ContentType::from_string(&test);
        assert_eq!(result.mime_type, "text/html");
    }
}
