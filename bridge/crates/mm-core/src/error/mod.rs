use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub enum ErrorKind {
    InvalidJson,
    InvalidRequest,
    UnknownMethod,
    Unauthorized,
    Forbidden,
    NotFound,
    RateLimited,
    NetworkError,
    TlsError,
    ServerError,
    WebsocketDisconnected,
    InternalError,
}

impl ErrorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidJson => "invalid_json",
            Self::InvalidRequest => "invalid_request",
            Self::UnknownMethod => "unknown_method",
            Self::Unauthorized => "unauthorized",
            Self::Forbidden => "forbidden",
            Self::NotFound => "not_found",
            Self::RateLimited => "rate_limited",
            Self::NetworkError => "network_error",
            Self::TlsError => "tls_error",
            Self::ServerError => "server_error",
            Self::WebsocketDisconnected => "websocket_disconnected",
            Self::InternalError => "internal_error",
        }
    }

    pub fn retryable(self) -> bool {
        matches!(
            self,
            Self::RateLimited
                | Self::NetworkError
                | Self::ServerError
                | Self::WebsocketDisconnected
        )
    }
}

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("{kind}: {message}", kind = kind.as_str())]
    Protocol { kind: ErrorKind, message: String },
}

impl BridgeError {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self::Protocol {
            kind,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> ErrorKind {
        match self {
            Self::Protocol { kind, .. } => *kind,
        }
    }

    pub fn message(&self) -> &str {
        match self {
            Self::Protocol { message, .. } => message,
        }
    }
}
