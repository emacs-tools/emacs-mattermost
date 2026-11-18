use mm_core::{BridgeError, ErrorKind};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Mattermost API error: {message} ({id})")]
    Api {
        id: String,
        message: String,
        status_code: u16,
    },

    #[error("Missing Token header in login response")]
    MissingTokenHeader,

    #[error("Invalid JSON in response: {0}")]
    InvalidJson(#[from] serde_json::Error),

    #[error("TLS configuration error: {0}")]
    Tls(String),
}

impl From<ClientError> for BridgeError {
    fn from(err: ClientError) -> Self {
        let kind = match &err {
            ClientError::Http(e) if e.is_timeout() => ErrorKind::NetworkError,
            ClientError::Http(e) if e.is_connect() => ErrorKind::NetworkError,
            ClientError::Api { .. } => ErrorKind::ServerError,
            ClientError::MissingTokenHeader => ErrorKind::Unauthorized,
            ClientError::InvalidJson(_) => ErrorKind::InternalError,
            ClientError::Tls(_) => ErrorKind::TlsError,
            _ => ErrorKind::InternalError,
        };
        BridgeError::new(kind, err.to_string())
    }
}
