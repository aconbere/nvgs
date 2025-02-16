use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use another_rust_warc::header::{FieldNames, Header, RecordID, RecordTypes};
use another_rust_warc::record::Record;
use another_rust_warc::writer::write_record;
use anyhow::{Result, anyhow};
use chrono::prelude::*;
use reqwest::blocking::Client;
use reqwest::header::{CONTENT_TYPE, USER_AGENT};

static USER_AGENT_STR: &str = "nvgs/1.0";

pub fn fetch(uri_str: &str, output: &Path) -> Result<()> {
    let client = Client::new();

    let request = client
        .get(uri_str)
        .header(USER_AGENT, USER_AGENT_STR)
        .build()?;

    let mut response =
        client.execute(request.try_clone().ok_or(anyhow!("could not clone body"))?)?;

    let mut out_file = File::create(output)?;
    write_request_record(&mut out_file, &request)?;
    write_response_record(&mut out_file, &mut response)?;

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
