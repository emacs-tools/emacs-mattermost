use crate::{app::App, config::BridgeConfig, status, transport};
use anyhow::Context;
use mm_web::state::WebState;
use tokio::net::TcpListener;

pub async fn run(config: BridgeConfig) -> anyhow::Result<()> {
    let tcp_listener = TcpListener::bind(&config.listen)
        .await
        .with_context(|| format!("failed to bind {}", config.listen))?;

    tracing::info!(listen = %config.listen, "bridge listening");

    let app = App::new(config.data_dir.clone());
    let web_state = WebState::new(status::current_status(&config.data_dir));
    let web_task = tokio::spawn(mm_web::run(config.web_listen.clone(), web_state));

    tokio::select! {
        result = accept_loop(tcp_listener, app) => {
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

async fn accept_loop(listener: TcpListener, app: App) -> anyhow::Result<()> {
    loop {
        let (stream, peer) = listener.accept().await.context("accept failed")?;
        tracing::debug!(%peer, "accepted connection");

        let app = app.clone();
        tokio::spawn(async move {
            if let Err(error) = transport::handle_client(app, stream).await {
                tracing::warn!(%error, "client handler failed");
            }
        });
    }
}
