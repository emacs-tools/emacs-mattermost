use mm_client::ws::WsConnection;
use mm_core::{BridgeError, ErrorKind, ServerId, User};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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
}

impl AppState {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
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

        tokio::spawn(async move {
            match WsConnection::new(&state.url, &state.token).await {
                Ok(mut ws) => {
                    tracing::info!(server_id = %sid, "WebSocket connected");
                    loop {
                        match ws.next_event().await {
                            Ok(Some(event_data)) => {
                                tracing::debug!(server_id = %sid, "WS Event: {}", event_data);
                                // TODO: Fan out to Emacs clients
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
