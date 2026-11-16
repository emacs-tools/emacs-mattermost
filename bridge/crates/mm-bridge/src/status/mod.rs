use mm_core::BridgeStatus;
use serde_json::Value;

pub const BRIDGE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn current_status() -> BridgeStatus {
    BridgeStatus::running(BRIDGE_VERSION)
}

pub fn status_result() -> Value {
    serde_json::to_value(current_status()).unwrap_or_else(|_| {
        serde_json::json!({
            "bridge": {
                "version": BRIDGE_VERSION,
                "state": "running"
            },
            "servers": []
        })
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
