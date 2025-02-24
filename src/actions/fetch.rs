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
