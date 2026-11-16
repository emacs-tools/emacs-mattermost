use mm_core::{BridgeState, ServerSummary};
use serde_json::{json, Value};

pub const BRIDGE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn status_result() -> Value {
    let servers: Vec<ServerSummary> = Vec::new();

    json!({
        "bridge": {
            "version": BRIDGE_VERSION,
            "state": BridgeState::Running,
        },
        "servers": servers,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_has_empty_servers() {
        let value = status_result();
        assert_eq!(value["servers"].as_array().unwrap().len(), 0);
        assert_eq!(value["bridge"]["state"], "running");
    }
}
