use crate::session::{ensure_dir, list_stems, validate_server_id};
use mm_core::{BridgeError, ErrorKind, ServerConfig, ServerId};
use std::fs;
use std::path::{Path, PathBuf};

/// Non-secret per-server configuration storage.
#[derive(Debug, Clone)]
pub struct ServerStore {
    dir: PathBuf,
}

impl ServerStore {
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        Self {
            dir: data_dir.as_ref().join("servers"),
        }
    }

    pub fn load(&self, server_id: &ServerId) -> Result<Option<ServerConfig>, BridgeError> {
        let path = self.path(server_id)?;
        match fs::read_to_string(&path) {
            Ok(text) => {
                let config = serde_json::from_str(&text).map_err(|err| {
                    BridgeError::new(
                        ErrorKind::InternalError,
                        format!("invalid server config at {}: {err}", path.display()),
                    )
                })?;
                Ok(Some(config))
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(BridgeError::new(
                ErrorKind::InternalError,
                format!("read server config at {}: {err}", path.display()),
            )),
        }
    }

    pub fn save(&self, config: &ServerConfig) -> Result<(), BridgeError> {
        validate_server_id(&config.server_id)?;
        if config.url.trim().is_empty() {
            return Err(BridgeError::new(
                ErrorKind::InvalidRequest,
                "server url must not be empty",
            ));
        }

        let path = self.path(&config.server_id)?;
        ensure_dir(&self.dir)?;

        let text = serde_json::to_string_pretty(config).map_err(|err| {
            BridgeError::new(ErrorKind::InternalError, format!("encode config: {err}"))
        })?;

        let tmp_path = path.with_extension("json.tmp");
        fs::write(&tmp_path, text).map_err(|err| {
            BridgeError::new(
                ErrorKind::InternalError,
                format!("write config at {}: {err}", tmp_path.display()),
            )
        })?;
        fs::rename(&tmp_path, &path).map_err(|err| {
            BridgeError::new(
                ErrorKind::InternalError,
                format!("rename config at {}: {err}", path.display()),
            )
        })?;
        Ok(())
    }

    pub fn list_servers(&self) -> Result<Vec<ServerId>, BridgeError> {
        list_stems(&self.dir, "json")
    }

    fn path(&self, server_id: &ServerId) -> Result<PathBuf, BridgeError> {
        validate_server_id(server_id)?;
        Ok(self.dir.join(format!("{}.json", server_id.as_str())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(id: &str) -> ServerConfig {
        ServerConfig {
            server_id: ServerId::new(id),
            url: "https://mm.example.local".to_string(),
        }
    }

    #[test]
    fn missing_config_is_none() {
        let dir = tempfile::tempdir().unwrap();
        let store = ServerStore::new(dir.path());
        assert!(store.load(&ServerId::new("work")).unwrap().is_none());
    }

    #[test]
    fn save_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let store = ServerStore::new(dir.path());
        store.save(&config("work")).unwrap();
        let loaded = store.load(&ServerId::new("work")).unwrap().unwrap();
        assert_eq!(loaded.url, "https://mm.example.local");
    }

    #[test]
    fn lists_sorted_servers() {
        let dir = tempfile::tempdir().unwrap();
        let store = ServerStore::new(dir.path());
        store.save(&config("zeta")).unwrap();
        store.save(&config("alpha")).unwrap();
        let names: Vec<String> = store
            .list_servers()
            .unwrap()
            .into_iter()
            .map(|id| id.as_str().to_string())
            .collect();
        assert_eq!(names, vec!["alpha", "zeta"]);
    }
}
