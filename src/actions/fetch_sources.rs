use anyhow::Result;
use reqwest::blocking::Client;
use reqwest::header::USER_AGENT;
use rusqlite::Connection;

use crate::db::crawls;
use crate::db::sources;
use crate::user_agent;

pub fn fetch_sources(connection: &Connection) -> Result<()> {
    let client = Client::new();

    let sources = sources::get_all(connection)?;

    for source in sources {
        let request = client
            .get(&source.url)
            .header(USER_AGENT, user_agent::USER_AGENT)
            .build()?;

        let response = client.execute(request)?;
        let response_text = response.text()?;
        let urls = response_text.lines();
        for url in urls {
            let crawl = crawls::ToCrawl::new(url, &source.url)?;
            crawls::insert(connection, &crawl)?;
        }
        sources::insert(connection, &source.get_updated());
    }

    Ok(())
}
