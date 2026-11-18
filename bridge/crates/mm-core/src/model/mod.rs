use crate::ids::{ChannelId, PostId, ServerId, TeamId, UserId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeState {
    Running,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerSummary {
    pub server_id: ServerId,
    pub state: String,
    pub has_token: bool,
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

/// Normalized Mattermost user. Only fields needed by the MVP UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    #[serde(default)]
    pub nickname: String,
    #[serde(default)]
    pub first_name: String,
    #[serde(default)]
    pub last_name: String,
}

/// Persisted per-server configuration (no secrets).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub server_id: ServerId,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: TeamId,
    pub name: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: PostId,
    pub channel_id: ChannelId,
    pub user_id: UserId,
    pub root_id: Option<PostId>,
    pub message: String,
    pub create_at: i64,
    pub update_at: i64,
    pub delete_at: i64,
}
