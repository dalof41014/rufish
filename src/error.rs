use thiserror::Error;

#[derive(Debug, Error)]
pub enum RedfishError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API error {status}: {message}")]
    Api { status: u16, message: String },

    #[error("Authentication failed")]
    AuthFailed,

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Session expired")]
    SessionExpired,

    #[error("Parse error: {0}")]
    Parse(String),
}

pub type Result<T> = std::result::Result<T, RedfishError>;
