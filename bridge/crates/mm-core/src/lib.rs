//! Shared domain types for emacs-mattermost.
//!
//! This crate contains no network code.

pub mod error;
pub mod ids;
pub mod model;

pub use error::{BridgeError, ErrorKind};
pub use ids::{ChannelId, PostId, ServerId, TeamId, UserId};
pub use model::{BridgeState, ServerSummary};
