mod handlers;
mod state;

use crate::{protocol, status};
use mm_core::BridgeError;
use std::path::PathBuf;

pub use state::AppState;

#[derive(Clone)]
pub struct App {
    pub data_dir: PathBuf,
    pub state: AppState,
}

impl App {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
            state: AppState::new(),
        }
    }

    pub async fn handle_request(&self, request: protocol::Request) -> protocol::Response {
        match request.method.as_str() {
            "ping" => protocol::Response::ok(request.id, protocol::pong_result()),
            "status" => protocol::Response::ok(request.id, status::status_result(&self.data_dir)),
            "connect" => handlers::handle_connect(&self.state, &self.data_dir, request).await,
            "me" => handlers::handle_me(&self.state, request).await,
            "teams" => handlers::handle_teams(&self.state, request).await,
            "list_channels" => handlers::handle_list_channels(&self.state, request).await,
            "open_self_dm" => handlers::handle_open_self_dm(&self.state, request).await,
            "get_history" => handlers::handle_get_history(&self.state, request).await,
            "send_post" => handlers::handle_send_post(&self.state, request).await,
            _ => protocol::Response::err(
                request.id,
                BridgeError::new(
                    mm_core::ErrorKind::UnknownMethod,
                    format!("unknown method: {}", request.method),
                ),
            ),
        }
    }

    pub async fn response_line(&self, line: &str) -> String {
        let response = match protocol::parse_request(line) {
            Ok(request) => self.handle_request(request).await,
            Err((id, error)) => protocol::Response::err(id, error),
        };
        match protocol::encode_response(&response) {
            Ok(encoded) => encoded,
            Err(error) => fallback_error_line(&response.id, error),
        }
    }
}

fn fallback_error_line(id: &str, error: BridgeError) -> String {
    let response = protocol::Response::err(id, error);
    let value = serde_json::to_value(response).unwrap_or_else(|_| {
        serde_json::json!({
            "id": id,
            "type": "response",
            "ok": false,
            "error": { "code": "internal_error", "message": "encoding failed", "retryable": false }
        })
    });
    format!("{value}\n")
}
