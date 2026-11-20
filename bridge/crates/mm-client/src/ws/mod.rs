use crate::error::ClientError;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async_tls_with_config, tungstenite::Message};
use url::Url;

pub struct WsConnection {
    stream: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
}

impl WsConnection {
    pub async fn new(base_url: &str, token: &str) -> Result<Self, ClientError> {
        tracing::info!("Connecting to WebSocket at {}", base_url);

        let mut url = Url::parse(base_url).map_err(|e| ClientError::Tls(e.to_string()))?;
        let scheme = if url.scheme() == "https" { "wss" } else { "ws" };
        url.set_scheme(scheme)
            .map_err(|_| ClientError::Tls("Invalid scheme".into()))?;
        url.set_path("/api/v4/websocket");

        let (stream, _) = connect_async_tls_with_config(url.to_string(), None, false, None)
            .await
            .map_err(|e| ClientError::Tls(format!("WS connect failed: {}", e)))?;

        let mut conn = Self { stream };
        tracing::debug!("Sending authentication challenge");
        conn.authenticate(token).await?;

        tracing::debug!("Waiting for hello event");
        conn.wait_for_hello().await?;
        tracing::info!("WebSocket authenticated successfully");

        Ok(conn)
    }

    async fn authenticate(&mut self, token: &str) -> Result<(), ClientError> {
        let auth_msg = serde_json::json!({
            "seq": 1,
            "action": "authentication_challenge",
            "data": { "token": token }
        });

        self.stream
            .send(Message::Text(auth_msg.to_string().into()))
            .await
            .map_err(|e| ClientError::Tls(format!("Auth send failed: {}", e)))?;

        Ok(())
    }

    async fn wait_for_hello(&mut self) -> Result<(), ClientError> {
        while let Some(msg) = self.stream.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if text.contains("\"event\":\"hello\"") {
                        return Ok(());
                    }
                }
                Ok(_) => continue,
                Err(e) => return Err(ClientError::Tls(format!("WS read error: {}", e))),
            }
        }
        Err(ClientError::Tls("Connection closed before hello".into()))
    }

    pub async fn next_event(&mut self) -> Result<Option<String>, ClientError> {
        while let Some(msg) = self.stream.next().await {
            match msg {
                Ok(Message::Text(text)) => return Ok(Some(text.to_string())),
                Ok(Message::Close(_)) => return Ok(None),
                Ok(_) => continue,
                Err(e) => return Err(ClientError::Tls(format!("WS read error: {}", e))),
            }
        }
        Ok(None)
    }
}
