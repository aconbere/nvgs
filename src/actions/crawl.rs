use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use another_rust_warc::header::{FieldNames, Header, RecordID, RecordTypes};
use another_rust_warc::record::Record;
use another_rust_warc::writer::write_record;
use anyhow::{Result, anyhow};
use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use chrono::Utc;
use chrono::format::SecondsFormat;
use reqwest::blocking::Client;
use reqwest::header::{CONTENT_TYPE, USER_AGENT};
use rusqlite::Connection;

use crate::db;

static USER_AGENT_STR: &str = "nvgs/1.0";

fn encode_url(url: &str) -> String {
    URL_SAFE.encode(url)
}

pub fn crawl(connection: &Connection, path: &PathBuf) -> Result<()> {
    let client = Client::new();
    let entries = db::index::get_all_needing_update(connection)?;

    for e in entries {
        crawl_one(connection, path, &client, &e.url)?;
    }
    Ok(())
}

pub fn crawl_one(
    connection: &Connection,
    path: &PathBuf,
    client: &Client,
    url: &str,
) -> Result<()> {
    db::index::set_crawling(connection, url)?;

    let result: Result<()> = {
        let request = client.get(url).header(USER_AGENT, USER_AGENT_STR).build()?;

        let mut response =
            client.execute(request.try_clone().ok_or(anyhow!("could not clone body"))?)?;

        let encoded_url = encode_url(url);
        let warc_path = path.join("warcs").join(encoded_url);
        let mut warc_file = File::create(warc_path)?;

        write_request_record(&mut warc_file, &request)?;
        write_response_record(&mut warc_file, &mut response)?;
        Ok(())
    };

    match result {
        Ok(_) => {
            let now = Utc::now().timestamp();
            db::index::set_ready(connection, url, now)?;
        }
        Err(_) => db::index::set_ready(connection, url, 0)?,
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

fn write_response_record(
    writer: &mut dyn Write,
    response: &mut reqwest::blocking::Response,
) -> Result<()> {
    let date = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);

    let content_length = response
        .content_length()
        .ok_or(anyhow!("no valid content length header on response"))?;

    let remote_addr = response
        .remote_addr()
        .ok_or(anyhow!("no valid remote address header in response"))?
        .to_string();

    let content_type = match response.headers().get(CONTENT_TYPE) {
        Some(hv) => hv.to_str().unwrap_or("application/octect-stream"),
        None => "application/octet-stream",
    }
    .to_string();

    //.unwrap_or("application/octet-stream".to_string());

    let mut header = Header::new();
    header.insert(FieldNames::RecordID, RecordID::new().to_string());
    header.insert(FieldNames::Type, RecordTypes::Response.to_string());
    header.insert(FieldNames::Date, date);
    header.insert(FieldNames::IPAddress, remote_addr);
    header.insert(FieldNames::ContentType, content_type);
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
