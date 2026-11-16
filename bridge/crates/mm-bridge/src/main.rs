use anyhow::Context;
use mm_bridge::{config::BridgeConfig, server};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let config = BridgeConfig::from_env().context("failed to load bridge config")?;
    tracing::info!(listen = %config.listen, "starting emacs-mattermost bridge");

    server::run(config).await
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("mm_bridge=info,warn"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_writer(std::io::stderr)
        .init();
}
