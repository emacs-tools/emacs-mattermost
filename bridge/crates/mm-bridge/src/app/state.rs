use mm_client::ws::WsConnection;
use mm_client::MattermostClient;
use mm_core::{BridgeError, ErrorKind, ServerId, User};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub struct ConnectionState {
    pub user: User,
    pub token: String,
    pub url: String,
    #[allow(dead_code)]
    pub ca_file: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AppState {
    connections: Arc<Mutex<HashMap<ServerId, ConnectionState>>>,
    event_tx: broadcast::Sender<Value>,
}

impl AppState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            event_tx: tx,
        }
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<Value> {
        self.event_tx.subscribe()
    }

    pub fn get_connection(&self, server_id: &ServerId) -> Result<ConnectionState, BridgeError> {
        self.connections
            .lock()
            .unwrap()
            .get(server_id)
            .cloned()
            .ok_or_else(|| BridgeError::new(ErrorKind::Unauthorized, "not connected"))
    }

    pub fn add_connection(&self, server_id: ServerId, state: ConnectionState) {
        self.connections.lock().unwrap().insert(server_id, state);
    }

    pub async fn start_ws(&self, server_id: &ServerId) -> Result<(), BridgeError> {
        let state = self.get_connection(server_id)?;
        let sid = server_id.clone();
        let event_tx = self.event_tx.clone();

        // Создаём клиент для резолвинга имён в WS-событиях
        let ca_path = state.ca_file.as_ref().map(PathBuf::from);
        let client = match MattermostClient::new(state.url.clone(), ca_path.as_deref()) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(server_id = %sid, "Failed to create client for WS: {}", e);
                return Err(BridgeError::new(ErrorKind::InternalError, e.to_string()));
            }
        };
        let token = state.token.clone();

        tokio::spawn(async move {
            match WsConnection::new(&state.url, &state.token).await {
                Ok(mut ws) => {
                    tracing::info!(server_id = %sid, "WebSocket connected");
                    loop {
                        match ws.next_event().await {
                            Ok(Some(event_data)) => {
                                if let Ok(json_val) = serde_json::from_str::<Value>(&event_data) {
                                    if let Some(event_type) =
                                        json_val.get("event").and_then(|v| v.as_str())
                                    {
                                        if event_type == "posted" {
                                            if let Some(data) = json_val.get("data") {
                                                if let Some(post_str) =
                                                    data.get("post").and_then(|v| v.as_str())
                                                {
                                                    if let Ok(mut post_obj) =
                                                        serde_json::from_str::<Value>(post_str)
                                                    {
                                                        // Резолвим имя отправителя
                                                        if let Some(user_id) = post_obj
                                                            .get("user_id")
                                                            .and_then(|v| v.as_str())
                                                        {
                                                            if let Ok(user) = client
                                                                .get_user(&token, user_id)
                                                                .await
                                                            {
                                                                post_obj["user_display_name"] =
                                                                    Value::String(
                                                                        user.display_name(),
                                                                    );
                                                            }
                                                        }

                                                        let normalized = serde_json::json!({
                                                            "type": "event",
                                                            "method": "posted",
                                                            "params": {
                                                                "post": post_obj
                                                            }
                                                        });
                                                        let _ = event_tx.send(normalized);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Ok(None) => {
                                tracing::warn!(server_id = %sid, "WebSocket closed");
                                break;
                            }
                            Err(e) => {
                                tracing::error!(server_id = %sid, "WS Error: {}", e);
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(server_id = %sid, "WS Connect failed: {}", e);
                }
            }
        });

        Ok(())
    }
}
