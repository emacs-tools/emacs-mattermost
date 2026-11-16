use std::env;

#[derive(Debug, Clone)]
pub struct BridgeConfig {
    pub listen: String,
}

impl BridgeConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let listen =
            env::var("MM_BRIDGE_LISTEN").unwrap_or_else(|_| "127.0.0.1:8765".to_string());

        validate_listen(&listen)?;

        Ok(Self { listen })
    }
}

fn validate_listen(listen: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        !listen.trim().is_empty(),
        "MM_BRIDGE_LISTEN must not be empty"
    );
    anyhow::ensure!(
        listen.contains(':'),
        "MM_BRIDGE_LISTEN must be host:port"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_listen() {
        assert!(validate_listen("").is_err());
    }

    #[test]
    fn accepts_host_port() {
        assert!(validate_listen("127.0.0.1:8765").is_ok());
    }
}
