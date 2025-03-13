use std::path::PathBuf;

use anyhow::{Result, anyhow};
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response,
    response::{IntoResponse, Response},
    routing,
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio_rusqlite::Connection;

use crate::actions::search;
use crate::actions::search::Document;
use crate::db::crawls;

static SEARCH_PAGE: &str = include_str!("../../data/search_page.html");

pub async fn start(path: &PathBuf, address: &str) -> Result<()> {
    println!("Starting nvgs server: {}", address);
    let db_path = path.join("nvgs.db");
    let connection = Connection::open(db_path).await?;

    // Note using post for crawls/get because sending
    // urls through query params is a pain in my ass
    let app = Router::new()
        .route("/crawls", routing::post(add_crawl))
        .route("/crawls/get", routing::post(get_crawl))
        .route("/crawls", routing::delete(delete_crawl))
        .route("/search", routing::post(search))
        .route("/search", routing::get(search_page))
        .with_state(connection);

    let listener = TcpListener::bind(address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

#[derive(Deserialize)]
struct GetCrawlRequest {
    url: String,
}

async fn get_crawl(
    State(connection): State<Connection>,
    Json(payload): Json<GetCrawlRequest>,
) -> Result<(StatusCode, response::Json<crawls::Crawl>), AppError> {
    let crawl = connection
        .call(move |conn| {
            Ok(crawls::get(&conn, &payload.url)
                .map_err(|e| tokio_rusqlite::Error::Other(e.into()))?)
        })
        .await?
        .ok_or(anyhow!("no crawl found"))?;
    Ok((StatusCode::CREATED, response::Json(crawl)))
}

#[derive(Deserialize)]
struct AddCrawlRequest {
    urls: Vec<String>,
}

async fn add_crawl(
    State(connection): State<Connection>,
    Json(payload): Json<AddCrawlRequest>,
) -> Result<(StatusCode, String), AppError> {
    connection
        .call(|conn| {
            for u in payload.urls {
                let crawl =
                    crawls::Crawl::new(&u).map_err(|e| tokio_rusqlite::Error::Other(e.into()))?;
                crawls::insert(&conn, &crawl)
                    .map_err(|e| tokio_rusqlite::Error::Other(e.into()))?;
                println!("Added url: {}", u);
            }
            Ok(())
        })
        .await?;
    Ok((StatusCode::CREATED, "".to_string()))
}

#[derive(Deserialize)]
struct DeleteCrawlRequest {
    url: String,
}

async fn delete_crawl(
    State(connection): State<Connection>,
    Json(payload): Json<DeleteCrawlRequest>,
) -> Result<(StatusCode, String), AppError> {
    connection
        .call(move |conn| {
            Ok(crawls::delete(&conn, &payload.url)
                .map_err(|e| tokio_rusqlite::Error::Other(e.into()))?)
        })
        .await?;
    Ok((StatusCode::CREATED, "".to_string()))
}

async fn search(
    State(connection): State<Connection>,
    Json(payload): Json<SearchQuery>,
) -> Result<(StatusCode, response::Json<SearchResult>), AppError> {
    let results = connection
        .call(move |conn| {
            let results = search::execute(conn, &payload.terms)
                .map_err(|e| tokio_rusqlite::Error::Other(e.into()))?;
            Ok(results)
        })
        .await?;

    Ok((
        StatusCode::CREATED,
        response::Json(SearchResult { results }),
    ))
}

async fn search_page() -> response::Html<&'static str> {
    response::Html(SEARCH_PAGE)
}

#[derive(Deserialize)]
struct SearchQuery {
    terms: Vec<String>,
}

#[derive(Serialize)]
struct SearchResult {
    results: Vec<Document>,
}

// Make our own error that wraps `anyhow::Error`.
struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
