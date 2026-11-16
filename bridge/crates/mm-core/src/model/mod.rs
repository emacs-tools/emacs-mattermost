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
