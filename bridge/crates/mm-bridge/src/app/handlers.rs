use crate::app::state::{AppState, ConnectionState};
use crate::protocol;
use mm_client::MattermostClient;
use mm_core::{BridgeError, ErrorKind, ServerConfig, ServerId, User};
use mm_store::{ServerStore, TokenStore};
use std::path::{Path, PathBuf};

pub async fn handle_connect(
    app_state: &AppState,
    data_dir: &Path,
    request: protocol::Request,
) -> protocol::Response {
    tracing::info!(
        "Received connect request for server_id={:?}",
        request.params.get("server_id")
    );

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
        do_login(&url, login, password, ca_file.as_deref()).await
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
            if let Err(e) = save_token_and_config(data_dir, &server_id, &token, &url) {
                return e;
            }

            match do_get_me(&url, &token, ca_file.as_deref()).await {
                Ok(user) => {
                    app_state.add_connection(
                        server_id.clone(),
                        ConnectionState {
                            user: user.clone(),
                            token: token.clone(),
                            url: url.clone(),
                            ca_file,
                        },
                    );

                    // Start WebSocket in background
                    tracing::info!(server_id = %server_id, "Connection established, starting WebSocket");
                    if let Err(e) = app_state.start_ws(&server_id).await {
                        tracing::error!(server_id = %server_id, "Failed to start WebSocket: {}", e);
                    } else {
                        tracing::info!(server_id = %server_id, "WebSocket spawn task created");
                    }

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

pub async fn handle_me(app_state: &AppState, request: protocol::Request) -> protocol::Response {
    tracing::debug!("Received me request");
    let params = match request.params.as_object() {
        Some(p) => p,
        None => return protocol_err(request.id, "params must be an object"),
    };
    let server_id = get_server_id(params);

    match app_state.get_connection(&server_id) {
        Ok(state) => protocol::Response::ok(
            request.id,
            serde_json::json!({
                "server_id": server_id,
                "user": state.user,
            }),
        ),
        Err(e) => protocol::Response::err(request.id, e),
    }
}

pub async fn handle_teams(app_state: &AppState, request: protocol::Request) -> protocol::Response {
    tracing::debug!("Received teams request");
    let params = match request.params.as_object() {
        Some(p) => p,
        None => return protocol_err(request.id, "params must be an object"),
    };
    let server_id = get_server_id(params);

    let client = match get_client(app_state, &server_id) {
        Ok(c) => c,
        Err(e) => return protocol::Response::err(request.id, e),
    };
    let token = match get_token(app_state, &server_id) {
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

pub async fn handle_list_channels(
    app_state: &AppState,
    request: protocol::Request,
) -> protocol::Response {
    tracing::debug!("Received list_channels request");
    let params = match request.params.as_object() {
        Some(p) => p,
        None => return protocol_err(request.id, "params must be an object"),
    };
    let server_id = get_server_id(params);

    let client = match get_client(app_state, &server_id) {
        Ok(c) => c,
        Err(e) => return protocol::Response::err(request.id, e),
    };
    let token = match get_token(app_state, &server_id) {
        Ok(t) => t,
        Err(e) => return protocol::Response::err(request.id, e),
    };

    match client.get_my_channels(&token).await {
        Ok(channels) => protocol::Response::ok(
            request.id,
            serde_json::json!({
                "server_id": server_id,
                "channels": channels,
            }),
        ),
        Err(e) => protocol::Response::err(request.id, e.into()),
    }
}

pub async fn handle_open_self_dm(
    app_state: &AppState,
    request: protocol::Request,
) -> protocol::Response {
    tracing::debug!("Received open_self_dm request: {:?}", request.params);
    let params = match request.params.as_object() {
        Some(p) => p,
        None => return protocol_err(request.id, "params must be an object"),
    };
    let server_id = get_server_id(params);

    let client = match get_client(app_state, &server_id) {
        Ok(c) => c,
        Err(e) => return protocol::Response::err(request.id, e),
    };
    let user_id = match get_user_id(app_state, &server_id) {
        Ok(id) => id.0,
        Err(e) => return protocol::Response::err(request.id, e),
    };
    let token = match get_token(app_state, &server_id) {
        Ok(t) => t,
        Err(e) => return protocol::Response::err(request.id, e),
    };

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

pub async fn handle_get_history(
    app_state: &AppState,
    request: protocol::Request,
) -> protocol::Response {
    tracing::debug!("Received get_history request: {:?}", request.params);
    let params = match request.params.as_object() {
        Some(p) => p,
        None => return protocol_err(request.id, "params must be an object"),
    };
    let server_id = get_server_id(params);
    let channel_id = match params.get("channel_id").and_then(|v| v.as_str()) {
        Some(id) => id,
        None => return protocol_err(request.id, "missing channel_id"),
    };
    let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as u32;
    let before = params.get("before").and_then(|v| v.as_str());

    let client = match get_client(app_state, &server_id) {
        Ok(c) => c,
        Err(e) => return protocol::Response::err(request.id, e),
    };
    let token = match get_token(app_state, &server_id) {
        Ok(t) => t,
        Err(e) => return protocol::Response::err(request.id, e),
    };

    match client.get_posts(&token, channel_id, limit, before).await {
        Ok(posts) => protocol::Response::ok(
            request.id,
            serde_json::json!({
                "server_id": server_id,
                "channel_id": channel_id,
                "posts": posts,
                "before": before,
            }),
        ),
        Err(e) => protocol::Response::err(request.id, e.into()),
    }
}

pub async fn handle_send_post(
    app_state: &AppState,
    request: protocol::Request,
) -> protocol::Response {
    tracing::debug!("Received send_post request");
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

    let client = match get_client(app_state, &server_id) {
        Ok(c) => c,
        Err(e) => return protocol::Response::err(request.id, e),
    };
    let token = match get_token(app_state, &server_id) {
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

fn get_client(app_state: &AppState, server_id: &ServerId) -> Result<MattermostClient, BridgeError> {
    let state = app_state.get_connection(server_id)?;
    let ca_path = state.ca_file.as_ref().map(PathBuf::from);
    MattermostClient::new(state.url, ca_path.as_deref())
        .map_err(|e| BridgeError::new(ErrorKind::InternalError, e.to_string()))
}

fn get_token(app_state: &AppState, server_id: &ServerId) -> Result<String, BridgeError> {
    app_state.get_connection(server_id).map(|s| s.token.clone())
}

fn get_user_id(app_state: &AppState, server_id: &ServerId) -> Result<mm_core::UserId, BridgeError> {
    app_state
        .get_connection(server_id)
        .map(|s| s.user.id.clone())
}

async fn do_login(
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

async fn do_get_me(url: &str, token: &str, ca: Option<&str>) -> Result<User, BridgeError> {
    let ca_path = ca.map(PathBuf::from);
    let client = MattermostClient::new(url.to_string(), ca_path.as_deref())
        .map_err(|e| BridgeError::new(ErrorKind::InternalError, e.to_string()))?;
    client.get_me(token).await.map_err(Into::into)
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
