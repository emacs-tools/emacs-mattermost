use crate::{protocol, status};
use mm_core::{BridgeError, ErrorKind};

pub fn handle_request(request: protocol::Request) -> protocol::Response {
    match request.method.as_str() {
        "ping" => protocol::Response::ok(request.id, protocol::pong_result()),
        "status" => protocol::Response::ok(request.id, status::status_result()),
        _ => protocol::Response::err(
            request.id,
            BridgeError::new(
                ErrorKind::UnknownMethod,
                format!("unknown method: {}", request.method),
            ),
        ),
    }
}

pub fn handle_line(line: &str) -> protocol::Response {
    match protocol::parse_request(line) {
        Ok(request) => handle_request(request),
        Err((id, error)) => protocol::Response::err(id, error),
    }
}

pub fn response_line(line: &str) -> String {
    let response = handle_line(line);
    match protocol::encode_response(&response) {
        Ok(encoded) => encoded,
        Err(error) => fallback_error_line(&response.id, error),
    }
}

fn fallback_error_line(id: &str, error: BridgeError) -> String {
    let response = protocol::Response::err(id, error);
    let value = serde_json::to_value(response).unwrap_or_else(|_| {
        serde_json::json!({
            "id": id,
            "type": "response",
            "ok": false,
            "error": {
                "code": "internal_error",
                "message": "failed to encode error response",
                "retryable": false
            }
        })
    });
    format!("{value}\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ping_line_returns_pong() {
        let line = response_line(r#"{"id":"1","type":"request","method":"ping","params":{}}"#);
        assert!(line.contains(r#""ok":true"#));
        assert!(line.contains(r#""pong":true"#));
    }

    #[test]
    fn unknown_method_returns_error() {
        let line = response_line(r#"{"id":"2","type":"request","method":"nope","params":{}}"#);
        assert!(line.contains(r#""ok":false"#));
        assert!(line.contains("unknown_method"));
    }
}
