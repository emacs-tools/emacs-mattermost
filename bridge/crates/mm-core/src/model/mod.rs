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

impl User {
    /// Human-readable display name: "First Last", falling back to username.
    pub fn display_name(&self) -> String {
        let full = format!("{} {}", self.first_name, self.last_name)
            .trim()
            .to_string();
        if full.is_empty() {
            self.username.clone()
        } else {
            full
        }
    }
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

/// Normalized Mattermost channel. Only fields needed by the MVP UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: ChannelId,
    pub team_id: TeamId,
    #[serde(rename = "type")]
    pub channel_type: String,
    pub display_name: String,
    pub name: String,
    /// Timestamp of the last post in this channel (milliseconds since epoch).
    #[serde(default)]
    pub last_post_at: i64,
}

/// File metadata attached to a post.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub size: i64,
    #[serde(default)]
    pub mime_type: String,
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
    #[serde(default)]
    pub file_ids: Vec<String>,
    /// Populated by the bridge after resolving user_id via /users/ids.
    #[serde(default, skip_deserializing)]
    pub user_display_name: String,
    /// Populated by the bridge after fetching file metadata.
    #[serde(default, skip_deserializing)]
    pub files: Vec<FileInfo>,
}
