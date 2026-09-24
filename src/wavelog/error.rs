use serde_json::Error as JsonError;
use thiserror::Error as ThisError;
use url::ParseError as UrlParseError;

#[derive(Debug, ThisError)]
pub enum WavelogError {
    #[error("invalid URL: {0}")]
    Url(#[from] UrlParseError),

    #[error("HTTP error: {0}")]
    Http(#[from] ureq::Error),

    #[error("API error ({status} {code}): {message}")]
    Api {
        status: u16,
        code: String,
        message: String,
    },

    #[error("unexpected HTTP status: {0}")]
    Status(u16),

    #[error("invalid response: {0}")]
    Json(#[from] JsonError),
}
