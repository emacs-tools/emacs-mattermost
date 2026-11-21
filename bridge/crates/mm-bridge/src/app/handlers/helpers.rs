use crate::app::state::AppState;
use crate::protocol;
use mm_client::MattermostClient;
use mm_core::{BridgeError, ErrorKind, ServerConfig, ServerId, User};
use mm_store::{ServerStore, TokenStore};
use std::path::{Path, PathBuf};

pub fn protocol_err(id: impl Into<String>, msg: &str) -> protocol::Response {
    protocol::Response::err(id, BridgeError::new(ErrorKind::InvalidRequest, msg))
}

pub fn get_server_id(params: &serde_json::Map<String, serde_json::Value>) -> ServerId {
    params
        .get("server_id")
        .and_then(|v| v.as_str())
        .map(ServerId::new)
        .unwrap_or_else(|| ServerId::new("default"))
}

pub fn get_client(
    app_state: &AppState,
    server_id: &ServerId,
) -> Result<MattermostClient, BridgeError> {
    let state = app_state.get_connection(server_id)?;
    let ca_path = state.ca_file.as_ref().map(PathBuf::from);
    MattermostClient::new(state.url, ca_path.as_deref())
        .map_err(|e| BridgeError::new(ErrorKind::InternalError, e.to_string()))
}

pub fn get_token(app_state: &AppState, server_id: &ServerId) -> Result<String, BridgeError> {
    app_state.get_connection(server_id).map(|s| s.token.clone())
}

pub fn get_user_id(
    app_state: &AppState,
    server_id: &ServerId,
) -> Result<mm_core::UserId, BridgeError> {
    app_state
        .get_connection(server_id)
        .map(|s| s.user.id.clone())
}

pub async fn do_login(
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

pub async fn do_get_me(url: &str, token: &str, ca: Option<&str>) -> Result<User, BridgeError> {
    let ca_path = ca.map(PathBuf::from);
    let client = MattermostClient::new(url.to_string(), ca_path.as_deref())
        .map_err(|e| BridgeError::new(ErrorKind::InternalError, e.to_string()))?;
    client.get_me(token).await.map_err(Into::into)
}

pub fn save_token_and_config(
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
