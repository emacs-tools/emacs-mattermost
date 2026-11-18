//! Local cache and session storage.
//!
//! Layout under the data directory:
//!
//!   <data_dir>/tokens/<server_id>.token  - plain bearer token, private mode
//!   <data_dir>/servers/<server_id>.json  - non-secret server config
//!
//! MVP storage is unencrypted by design. Secrets must never be logged.

pub mod cache;
pub mod session;

pub use cache::ServerStore;
pub use session::TokenStore;
