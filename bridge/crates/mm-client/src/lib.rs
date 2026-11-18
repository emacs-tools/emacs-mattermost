//! Mattermost REST API client.
//!
//! Supports explicit CA configuration for corporate environments.

pub mod auth;
pub mod client;
pub mod error;
pub mod rest;
pub mod tls;
pub mod ws;

pub use client::MattermostClient;
pub use error::ClientError;
