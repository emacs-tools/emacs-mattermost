use mm_core::{BridgeError, ErrorKind};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ProtocolError>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProtocolError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

impl Response {
    pub fn ok(id: impl Into<String>, result: Value) -> Self {
        Self {
            id: id.into(),
            kind: "response",
            ok: true,
            result: Some(result),
            error: None,
        }
    }

    pub fn err(id: impl Into<String>, error: BridgeError) -> Self {
        let kind = error.kind();
        Self {
            id: id.into(),
            kind: "response",
            ok: false,
            result: None,
            error: Some(ProtocolError {
                code: kind.as_str().to_string(),
                message: error.message().to_string(),
                retryable: kind.retryable(),
            }),
        }
    }
}

pub fn parse_request(line: &str) -> Result<Request, (String, BridgeError)> {
    let value: Value = serde_json::from_str(line).map_err(|err| {
        (
            "unknown".to_string(),
            BridgeError::new(ErrorKind::InvalidJson, err.to_string()),
        )
    })?;

    let id = value
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();

    let request: Request = serde_json::from_value(value).map_err(|err| {
        (
            id.clone(),
            BridgeError::new(ErrorKind::InvalidRequest, err.to_string()),
        )
    })?;

    validate_request(&request).map_err(|err| (id, err))?;
    Ok(request)
}

fn validate_request(request: &Request) -> Result<(), BridgeError> {
    if request.id.trim().is_empty() {
        return Err(BridgeError::new(
            ErrorKind::InvalidRequest,
            "id must not be empty",
        ));
    }

    if request.kind != "request" {
        return Err(BridgeError::new(
            ErrorKind::InvalidRequest,
            "type must be request",
        ));
    }

    if request.method.trim().is_empty() {
        return Err(BridgeError::new(
            ErrorKind::InvalidRequest,
            "method must not be empty",
        ));
    }

    if !request.params.is_object() && !request.params.is_null() {
        return Err(BridgeError::new(
            ErrorKind::InvalidRequest,
            "params must be an object",
        ));
    }

    Ok(())
}

pub fn encode_response(response: &Response) -> Result<String, BridgeError> {
    let mut line = serde_json::to_string(response)
        .map_err(|err| BridgeError::new(ErrorKind::InternalError, err.to_string()))?;
    line.push('\n');
    Ok(line)
}

pub fn pong_result() -> Value {
    json!({ "pong": true })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ping_request() {
        let req = parse_request(r#"{"id":"1","type":"request","method":"ping","params":{}}"#)
            .expect("valid request");
        assert_eq!(req.method, "ping");
    }

    #[test]
    fn encodes_response_with_newline() {
        let line = encode_response(&Response::ok("1", pong_result())).expect("encode");
        assert!(line.ends_with('\n'));
        assert!(line.contains(r#""pong":true"#));
    }
}
