use crate::{config::BridgeConfig, transport};
use anyhow::Context;
use tokio::net::TcpListener;

pub async fn run(config: BridgeConfig) -> anyhow::Result<()> {
    let listener = TcpListener::bind(&config.listen)
        .await
        .with_context(|| format!("failed to bind {}", config.listen))?;

    tracing::info!(listen = %config.listen, "bridge listening");

    loop {
        let (stream, peer) = listener.accept().await.context("accept failed")?;
        tracing::debug!(%peer, "accepted connection");

        tokio::spawn(async move {
            if let Err(error) = transport::handle_client(stream).await {
                tracing::warn!(%error, "client handler failed");
            }
        });
    }
}
