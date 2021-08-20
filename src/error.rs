use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] hyper::http::Error),
    #[error("Hyper error: {0}")]
    Hyper(#[from] hyper::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Query error: {0}")]
    Query(#[from] serde_qs::Error),
    #[error("Invalid URL: {0}")]
    Url(#[from] hyper::http::uri::InvalidUri),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
