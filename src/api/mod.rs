use std::path::PathBuf;

use anyhow::Result;
use axum::{
    Json, Router,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware,
    middleware::Next,
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

mod auth;

#[derive(Clone)]
struct AppState {
    connection: Connection,
    auth_backend: auth::Backend,
}

static SEARCH_PAGE: &str = include_str!("../../data/search_page.html");

pub async fn start(path: &PathBuf, address: &str) -> Result<()> {
    println!("Starting nvgs server: {}", address);
    let db_path = path.join("nvgs.db");
    println!("Connecting: {}", db_path.display());
    let connection = Connection::open(db_path).await?;
    let auth_backend = auth::Backend::new(connection.clone());
    println!("Established connection");
    let state = AppState {
        connection,
        auth_backend,
    };

    // Note using post for crawls/get because sending
    // urls through query params is a pain in my ass
    let app = Router::new()
        .route("/crawls", routing::post(add_crawl))
        .route("/crawls/get", routing::post(get_crawl))
        .route("/crawls/delete", routing::post(delete_crawl))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .route("/search", routing::post(search))
        .route("/search", routing::get(search_page))
        .with_state(state)
        .fallback(handler_404);

    let listener = TcpListener::bind(address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    println!("done");
    Ok(())
}

#[derive(Deserialize)]
struct GetCrawlRequest {
    url: String,
}

async fn auth_middleware(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let Some(username) = headers.get("NVGS-USERNAME") else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let Some(password) = headers.get("NVGS-PASSWORD") else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let Ok(un) = username.to_str() else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let Ok(p) = password.to_str() else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let Ok(maybe_user) = app_state.auth_backend.authenticate(un, p).await else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let Some(_) = maybe_user else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let response = next.run(request).await;
    Ok(response)
}

async fn get_crawl(
    State(app_state): State<AppState>,
    Json(payload): Json<GetCrawlRequest>,
) -> Response {
    let crawl_result = app_state
        .connection
        .call(move |conn| {
            Ok(crawls::get(&conn, &payload.url)
                .map_err(|e| tokio_rusqlite::Error::Other(e.into()))?)
        })
        .await;

    let maybe_crawl = match crawl_result {
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
        Ok(mc) => mc,
    };

    if let Some(crawl) = maybe_crawl {
        (StatusCode::CREATED, response::Json(crawl)).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

#[derive(Deserialize)]
struct AddCrawlRequest {
    urls: Vec<String>,
}

async fn add_crawl(
    State(state): State<AppState>,
    Json(payload): Json<AddCrawlRequest>,
) -> Result<(StatusCode, String), AppError> {
    state
        .connection
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
    State(state): State<AppState>,
    Json(payload): Json<DeleteCrawlRequest>,
) -> Result<(StatusCode, String), AppError> {
    state
        .connection
        .call(move |conn| {
            Ok(crawls::delete(&conn, &payload.url)
                .map_err(|e| tokio_rusqlite::Error::Other(e.into()))?)
        })
        .await?;
    Ok((StatusCode::CREATED, "".to_string()))
}

async fn search(
    State(state): State<AppState>,
    Json(payload): Json<SearchQuery>,
) -> Result<(StatusCode, response::Json<SearchResult>), AppError> {
    let results = state
        .connection
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

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}
