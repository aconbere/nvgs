use std::path::PathBuf;

use anyhow::Result;
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response,
    response::{IntoResponse, Response},
    routing::post,
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio_rusqlite::Connection;

use crate::actions::search;
use crate::db::crawls;

pub async fn start(path: &PathBuf, address: &str) -> Result<()> {
    let db_path = path.join("nvgs.db");
    let connection = Connection::open(db_path).await?;

    let app = Router::new()
        .route("/add", post(add))
        .route("/search", post(search))
        .with_state(connection);

    let listener = TcpListener::bind(address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

async fn add(
    State(connection): State<Connection>,
    Json(payload): Json<AddUrls>,
) -> Result<(StatusCode, String), AppError> {
    connection
        .call(|conn| {
            for u in payload.urls {
                let crawl =
                    crawls::Crawl::new(&u).map_err(|e| tokio_rusqlite::Error::Other(e.into()))?;
                crawls::insert(&conn, &crawl)
                    .map_err(|e| tokio_rusqlite::Error::Other(e.into()))?;
            }
            Ok(())
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

// the input to our `create_user` handler
#[derive(Deserialize)]
struct AddUrls {
    urls: Vec<String>,
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct SearchQuery {
    terms: Vec<String>,
}

// the input to our `create_user` handler
#[derive(Serialize)]
struct SearchResult {
    results: Vec<(String, f64)>,
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
