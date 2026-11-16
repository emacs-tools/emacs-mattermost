use crate::ids::ServerId;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeState {
    Running,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerSummary {
    pub server_id: ServerId,
    pub state: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BridgeInfo {
    pub version: String,
    pub state: BridgeState,
}

#[derive(Debug, Clone, Serialize)]
pub struct BridgeStatus {
    pub bridge: BridgeInfo,
    pub servers: Vec<ServerSummary>,
}

impl BridgeStatus {
    pub fn running(version: impl Into<String>) -> Self {
        Self {
            bridge: BridgeInfo {
                version: version.into(),
                state: BridgeState::Running,
            },
            servers: Vec::new(),
        }
    }
}
