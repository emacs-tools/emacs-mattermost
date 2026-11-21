use mm_client::ws::WsConnection;
use mm_core::{BridgeError, ErrorKind, ServerId, User};
use serde_json::Value;
use std::collections::HashMap;
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
        let (tx, _) = broadcast::channel(100); // Буфер до 100 событий
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

        tokio::spawn(async move {
            match WsConnection::new(&state.url, &state.token).await {
                Ok(mut ws) => {
                    tracing::info!(server_id = %sid, "WebSocket connected");
                    loop {
                        match ws.next_event().await {
                            Ok(Some(event_data)) => {
                                // Пытаемся распарсить и нормализовать событие 'posted'
                                if let Ok(json_val) = serde_json::from_str::<Value>(&event_data) {
                                    if let Some(event_type) =
                                        json_val.get("event").and_then(|v| v.as_str())
                                    {
                                        if event_type == "posted" {
                                            if let Some(data) = json_val.get("data") {
                                                // Mattermost присылает post как строку, содержащую JSON
                                                if let Some(post_str) =
                                                    data.get("post").and_then(|v| v.as_str())
                                                {
                                                    if let Ok(post_obj) =
                                                        serde_json::from_str::<Value>(post_str)
                                                    {
                                                        let normalized = serde_json::json!({
                                                            "type": "event",
                                                            "method": "posted",
                                                            "params": {
                                                                "post": post_obj
                                                            }
                                                        });
                                                        // Игнорируем ошибку, если нет подписчиков (Emacs отключён)
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
