//! Web status UI.

pub mod routes;
pub mod state;
pub mod templates;

use anyhow::Context;
use axum::Router;
use state::WebState;
use tokio::net::TcpListener;

pub async fn run(listen: String, state: WebState) -> anyhow::Result<()> {
    let app = router(state);

    let listener = TcpListener::bind(&listen)
        .await
        .with_context(|| format!("failed to bind web listener {}", listen))?;

    tracing::info!(listen = %listen, "web status listening");

    axum::serve(listener, app)
        .await
        .context("web status server failed")?;

    Ok(())
}

pub fn router(state: WebState) -> Router {
    routes::router(state)
}
