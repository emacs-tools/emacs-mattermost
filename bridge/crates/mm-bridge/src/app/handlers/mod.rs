mod enrich;
mod helpers;

use crate::app::state::AppState;
use crate::protocol;
use helpers::*;
use mm_core::Post;
use std::path::Path;

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
        Err(mm_core::BridgeError::new(
            mm_core::ErrorKind::InvalidRequest,
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
                        crate::app::state::ConnectionState {
                            user: user.clone(),
                            token: token.clone(),
                            url: url.clone(),
                            ca_file,
                        },
                    );

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
    let current_user_id = match get_user_id(app_state, &server_id) {
        Ok(id) => id.0,
        Err(e) => return protocol::Response::err(request.id, e),
    };

    let channels_result = client.get_my_channels(&token).await;
    let teams_result = client.get_teams(&token).await;

    match (channels_result, teams_result) {
        (Ok(mut channels), Ok(teams)) => {
            for ch in &mut channels {
                if ch.channel_type == "D" && ch.display_name.is_empty() {
                    let parts: Vec<&str> = ch.name.split("__").collect();
                    if parts.len() == 2 {
                        let other_user_id = if parts[0] == current_user_id {
                            parts[1]
                        } else {
                            parts[0]
                        };

                        if let Ok(other_user) = client.get_user(&token, other_user_id).await {
                            ch.display_name = other_user.display_name();
                        }
                    }
                }
            }

            protocol::Response::ok(
                request.id,
                serde_json::json!({
                    "server_id": server_id,
                    "channels": channels,
                    "teams": teams,
                }),
            )
        }
        (Err(e), _) | (_, Err(e)) => protocol::Response::err(request.id, e.into()),
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
        Ok(mut posts) => {
            enrich::enrich_posts_with_display_names(&client, &token, &mut posts).await;
            enrich::enrich_posts_with_files(&client, &token, &mut posts).await;

            protocol::Response::ok(
                request.id,
                serde_json::json!({
                    "server_id": server_id,
                    "channel_id": channel_id,
                    "posts": posts,
                    "before": before,
                }),
            )
        }
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
