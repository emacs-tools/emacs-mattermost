use crate::{config::BridgeConfig, status, transport};
use anyhow::Context;
use mm_web::state::WebState;
use tokio::net::TcpListener;

pub async fn run(config: BridgeConfig) -> anyhow::Result<()> {
    let tcp_listener = TcpListener::bind(&config.listen)
        .await
        .with_context(|| format!("failed to bind {}", config.listen))?;

    tracing::info!(listen = %config.listen, "bridge listening");

    let web_state = WebState::new(status::current_status());
    let web_task = tokio::spawn(mm_web::run(config.web_listen.clone(), web_state));

    tokio::select! {
        result = accept_loop(tcp_listener) => {
            result
        }
        result = web_task => {
            match result {
                Ok(Ok(())) => Ok(()),
                Ok(Err(error)) => Err(error).context("web status task failed"),
                Err(error) => Err(error).context("web status task join failed"),
            }
        }
    }
}

async fn accept_loop(listener: TcpListener) -> anyhow::Result<()> {
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
