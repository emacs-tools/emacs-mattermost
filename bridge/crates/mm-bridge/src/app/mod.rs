use crate::{protocol, status};
use mm_client::MattermostClient;
use mm_core::{BridgeError, ErrorKind, ServerConfig, ServerId, User};
use mm_store::{ServerStore, TokenStore};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
struct ConnectionState {
    user: User,
    #[allow(dead_code)]
    token: String,
    url: String,
    ca_file: Option<String>,
}

#[derive(Debug, Clone)]
pub struct App {
    data_dir: PathBuf,
    connections: Arc<Mutex<HashMap<ServerId, ConnectionState>>>,
}

impl App {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub async fn handle_request(&self, request: protocol::Request) -> protocol::Response {
        match request.method.as_str() {
            "ping" => protocol::Response::ok(request.id, protocol::pong_result()),
            "status" => protocol::Response::ok(request.id, status::status_result(&self.data_dir)),
            "connect" => self.handle_connect(request).await,
            "me" => self.handle_me(request),
            "teams" => self.handle_teams(request).await,
            "open_self_dm" => self.handle_open_self_dm(request).await,
            "get_history" => self.handle_get_history(request).await,
            "send_post" => self.handle_send_post(request).await,
            _ => protocol::Response::err(
                request.id,
                BridgeError::new(
                    ErrorKind::UnknownMethod,
                    format!("unknown method: {}", request.method),
                ),
            ),
        }
    }

    async fn handle_connect(&self, request: protocol::Request) -> protocol::Response {
        let params = match request.params.as_object() {
            Some(p) => p,
            None => return protocol_err(request.id, "params must be an object"),
        };

        let server_id = get_server_id(params);
        let url = match params.get("url").and_then(|v| v.as_str()) {
            Some(u) => u.to_string(),
            None => return protocol_err(request.id, "missing url"),
        };

        let ca_file = params
            .get("ca_file")
            .and_then(|v| v.as_str())
            .map(String::from);

        let token_result = if let (Some(login), Some(password)) = (
            params.get("login_id").and_then(|v| v.as_str()),
            params.get("password").and_then(|v| v.as_str()),
        ) {
            self.do_login(&url, login, password, ca_file.as_deref())
                .await
        } else if let Some(token) = params.get("token").and_then(|v| v.as_str()) {
            Ok(token.to_string())
        } else {
            Err(BridgeError::new(
                ErrorKind::InvalidRequest,
                "provide credentials or token",
            ))
        };

        match token_result {
            Ok(token) => {
                if let Err(e) = save_token_and_config(&self.data_dir, &server_id, &token, &url) {
                    return e;
                }

                match self.do_get_me(&url, &token, ca_file.as_deref()).await {
                    Ok(user) => {
                        let mut conns = self.connections.lock().unwrap();
                        conns.insert(
                            server_id.clone(),
                            ConnectionState {
                                user: user.clone(),
                                token: token.clone(),
                                url: url.clone(),
                                ca_file,
                            },
                        );

                        protocol::Response::ok(
                            request.id,
                            serde_json::json!({
                                "server_id": server_id,
                                "user": user,
                            }),
                        )
                    }
                    Err(e) => protocol::Response::err(request.id, e),
                }
            }
            Err(e) => protocol::Response::err(request.id, e),
        }
    }

    fn handle_me(&self, request: protocol::Request) -> protocol::Response {
        let params = match request.params.as_object() {
            Some(p) => p,
            None => return protocol_err(request.id, "params must be an object"),
        };
        let server_id = get_server_id(params);

        let conns = self.connections.lock().unwrap();
        if let Some(state) = conns.get(&server_id) {
            protocol::Response::ok(
                request.id,
                serde_json::json!({
                    "server_id": server_id,
                    "user": state.user,
                }),
            )
        } else {
            protocol::Response::err(
                request.id,
                BridgeError::new(ErrorKind::Unauthorized, "not connected"),
            )
        }
    }

    async fn handle_teams(&self, request: protocol::Request) -> protocol::Response {
        let params = match request.params.as_object() {
            Some(p) => p,
            None => return protocol_err(request.id, "params must be an object"),
        };
        let server_id = get_server_id(params);

        let client = match self.get_client(&server_id) {
            Ok(c) => c,
            Err(e) => return protocol::Response::err(request.id, e),
        };

        let token = match self.get_token(&server_id) {
            Ok(t) => t,
            Err(e) => return protocol::Response::err(request.id, e),
        };

        match client.get_teams(&token).await {
            Ok(teams) => protocol::Response::ok(
                request.id,
                serde_json::json!({
                    "server_id": server_id,
                    "teams": teams,
                }),
            ),
            Err(e) => protocol::Response::err(request.id, e.into()),
        }
    }

    async fn handle_open_self_dm(&self, request: protocol::Request) -> protocol::Response {
        let params = match request.params.as_object() {
            Some(p) => p,
            None => return protocol_err(request.id, "params must be an object"),
        };
        let server_id = get_server_id(params);

        let client = match self.get_client(&server_id) {
            Ok(c) => c,
            Err(e) => return protocol::Response::err(request.id, e),
        };

        let user_id = match self.get_user_id(&server_id) {
            Ok(id) => id.0,
            Err(e) => return protocol::Response::err(request.id, e),
        };

        let token = match self.get_token(&server_id) {
            Ok(t) => t,
            Err(e) => return protocol::Response::err(request.id, e),
        };

        // For self-DM, we pass the same ID twice
        match client
            .create_direct_channel(&token, &[user_id.clone(), user_id])
            .await
        {
            Ok(channel_id) => protocol::Response::ok(
                request.id,
                serde_json::json!({
                    "server_id": server_id,
                    "channel": {
                        "id": channel_id,
                        "type": "direct",
                        "display_name": "Self"
                    }
                }),
            ),
            Err(e) => protocol::Response::err(request.id, e.into()),
        }
    }

    async fn handle_get_history(&self, request: protocol::Request) -> protocol::Response {
        let params = match request.params.as_object() {
            Some(p) => p,
            None => return protocol_err(request.id, "params must be an object"),
        };
        let server_id = get_server_id(params);
        let channel_id = match params.get("channel_id").and_then(|v| v.as_str()) {
            Some(id) => id,
            None => return protocol_err(request.id, "missing channel_id"),
        };
        let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(50) as u32;

        let client = match self.get_client(&server_id) {
            Ok(c) => c,
            Err(e) => return protocol::Response::err(request.id, e),
        };

        let token = match self.get_token(&server_id) {
            Ok(t) => t,
            Err(e) => return protocol::Response::err(request.id, e),
        };

        match client.get_posts(&token, channel_id, limit).await {
            Ok(posts) => protocol::Response::ok(
                request.id,
                serde_json::json!({
                    "server_id": server_id,
                    "channel_id": channel_id,
                    "posts": posts,
                }),
            ),
            Err(e) => protocol::Response::err(request.id, e.into()),
        }
    }

    async fn handle_send_post(&self, request: protocol::Request) -> protocol::Response {
        let params = match request.params.as_object() {
            Some(p) => p,
            None => return protocol_err(request.id, "params must be an object"),
        };
        let server_id = get_server_id(params);
        let channel_id = match params.get("channel_id").and_then(|v| v.as_str()) {
            Some(id) => id,
            None => return protocol_err(request.id, "missing channel_id"),
        };
        let message = match params.get("message").and_then(|v| v.as_str()) {
            Some(m) => m,
            None => return protocol_err(request.id, "missing message"),
        };
        let root_id = params.get("root_id").and_then(|v| v.as_str());

        let client = match self.get_client(&server_id) {
            Ok(c) => c,
            Err(e) => return protocol::Response::err(request.id, e),
        };

        let token = match self.get_token(&server_id) {
            Ok(t) => t,
            Err(e) => return protocol::Response::err(request.id, e),
        };

        match client
            .create_post(&token, channel_id, message, root_id)
            .await
        {
            Ok(post) => protocol::Response::ok(
                request.id,
                serde_json::json!({
                    "server_id": server_id,
                    "post": post,
                }),
            ),
            Err(e) => protocol::Response::err(request.id, e.into()),
        }
    }

    // --- Helpers ---

    fn get_client(&self, server_id: &ServerId) -> Result<MattermostClient, BridgeError> {
        let conns = self.connections.lock().unwrap();
        let state = conns
            .get(server_id)
            .ok_or_else(|| BridgeError::new(ErrorKind::Unauthorized, "not connected"))?;

        let ca_path = state.ca_file.as_ref().map(PathBuf::from);
        MattermostClient::new(state.url.clone(), ca_path.as_deref())
            .map_err(|e| BridgeError::new(ErrorKind::InternalError, e.to_string()))
    }

    fn get_token(&self, server_id: &ServerId) -> Result<String, BridgeError> {
        let conns = self.connections.lock().unwrap();
        conns
            .get(server_id)
            .map(|s| s.token.clone())
            .ok_or_else(|| BridgeError::new(ErrorKind::Unauthorized, "not connected"))
    }

    fn get_user_id(&self, server_id: &ServerId) -> Result<mm_core::UserId, BridgeError> {
        let conns = self.connections.lock().unwrap();
        conns
            .get(server_id)
            .map(|s| s.user.id.clone())
            .ok_or_else(|| BridgeError::new(ErrorKind::Unauthorized, "not connected"))
    }

    async fn do_login(
        &self,
        url: &str,
        login: &str,
        pass: &str,
        ca: Option<&str>,
    ) -> Result<String, BridgeError> {
        let ca_path = ca.map(PathBuf::from);
        let client = MattermostClient::new(url.to_string(), ca_path.as_deref())
            .map_err(|e| BridgeError::new(ErrorKind::InternalError, e.to_string()))?;
        client.login(login, pass).await.map_err(Into::into)
    }

    async fn do_get_me(
        &self,
        url: &str,
        token: &str,
        ca: Option<&str>,
    ) -> Result<User, BridgeError> {
        let ca_path = ca.map(PathBuf::from);
        let client = MattermostClient::new(url.to_string(), ca_path.as_deref())
            .map_err(|e| BridgeError::new(ErrorKind::InternalError, e.to_string()))?;
        client.get_me(token).await.map_err(Into::into)
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

fn protocol_err(id: impl Into<String>, msg: &str) -> protocol::Response {
    protocol::Response::err(id, BridgeError::new(ErrorKind::InvalidRequest, msg))
}

fn get_server_id(params: &serde_json::Map<String, serde_json::Value>) -> ServerId {
    params
        .get("server_id")
        .and_then(|v| v.as_str())
        .map(ServerId::new)
        .unwrap_or_else(|| ServerId::new("default"))
}

fn save_token_and_config(
    data_dir: &Path,
    server_id: &ServerId,
    token: &str,
    url: &str,
) -> Result<(), protocol::Response> {
    let token_store = TokenStore::new(data_dir);
    let server_store = ServerStore::new(data_dir);

    token_store
        .save(server_id, token)
        .map_err(|e| protocol::Response::err("req", e))?;

    let config = ServerConfig {
        server_id: server_id.clone(),
        url: url.to_string(),
    };
    server_store
        .save(&config)
        .map_err(|e| protocol::Response::err("req", e))?;

    Ok(())
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
