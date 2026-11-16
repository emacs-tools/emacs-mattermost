use std::env;

#[derive(Debug, Clone)]
pub struct BridgeConfig {
    pub listen: String,
    pub web_listen: String,
}

impl BridgeConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let listen = read_addr("MM_BRIDGE_LISTEN", "127.0.0.1:8765")?;
        let web_listen = read_addr("MM_WEB_LISTEN", "127.0.0.1:8766")?;

        Ok(Self { listen, web_listen })
    }
}

fn read_addr(name: &str, default: &str) -> anyhow::Result<String> {
    let value = env::var(name).unwrap_or_else(|_| default.to_string());
    validate_listen(name, &value)?;
    Ok(value)
}

fn validate_listen(name: &str, listen: &str) -> anyhow::Result<()> {
    anyhow::ensure!(!listen.trim().is_empty(), "{name} must not be empty");
    anyhow::ensure!(listen.contains(':'), "{name} must be host:port");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_listen() {
        assert!(validate_listen("MM_BRIDGE_LISTEN", "").is_err());
    }

    #[test]
    fn accepts_host_port() {
        assert!(validate_listen("MM_BRIDGE_LISTEN", "127.0.0.1:8765").is_ok());
    }
}
