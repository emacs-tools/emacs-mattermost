use mm_core::{BridgeStatus, ServerId, ServerSummary};
use mm_store::{ServerStore, TokenStore};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::Path;

pub const BRIDGE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build status from persisted stores. Never includes secrets.
pub fn current_status(data_dir: &Path) -> BridgeStatus {
    let mut status = BridgeStatus::running(BRIDGE_VERSION);

    let token_store = TokenStore::new(data_dir);
    let server_store = ServerStore::new(data_dir);

    let token_ids = token_store.list_servers().unwrap_or_default();
    let config_ids = server_store.list_servers().unwrap_or_default();

    let mut all: BTreeSet<String> = BTreeSet::new();
    all.extend(token_ids.iter().map(|id| id.as_str().to_string()));
    all.extend(config_ids.iter().map(|id| id.as_str().to_string()));

    for id in all {
        let server_id = ServerId::new(id);
        let has_token = token_ids.contains(&server_id);
        status.servers.push(ServerSummary {
            server_id,
            state: "disconnected".to_string(),
            has_token,
        });
    }

    status
}

pub fn status_result(data_dir: &Path) -> Value {
    serde_json::to_value(current_status(data_dir)).unwrap_or_else(|_| {
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
    fn empty_dir_has_no_servers() {
        let dir = tempfile::tempdir().unwrap();
        let value = status_result(dir.path());
        assert_eq!(value["servers"].as_array().unwrap().len(), 0);
        assert_eq!(value["bridge"]["state"], "running");
    }

    #[test]
    fn stored_token_appears_without_secret() {
        let dir = tempfile::tempdir().unwrap();
        TokenStore::new(dir.path())
            .save(&ServerId::new("work"), "super-secret")
            .unwrap();

        let value = status_result(dir.path());
        let servers = value["servers"].as_array().unwrap();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0]["server_id"], "work");
        assert_eq!(servers[0]["has_token"], true);
        assert!(!value.to_string().contains("super-secret"));
    }
}
