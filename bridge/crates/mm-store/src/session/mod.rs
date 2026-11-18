use mm_core::{BridgeError, ErrorKind, ServerId};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Plain-file bearer token storage, one file per server.
///
/// Files are written atomically (temp file + rename).
/// On Unix the file mode is 0600. Tokens are never logged.
#[derive(Debug, Clone)]
pub struct TokenStore {
    dir: PathBuf,
}

impl TokenStore {
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        Self {
            dir: data_dir.as_ref().join("tokens"),
        }
    }

    pub fn load(&self, server_id: &ServerId) -> Result<Option<String>, BridgeError> {
        let path = self.path(server_id)?;
        match fs::read_to_string(&path) {
            Ok(text) => {
                let token = text.trim().to_string();
                if token.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(token))
                }
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(io_error("read token", &path, err)),
        }
    }

    pub fn save(&self, server_id: &ServerId, token: &str) -> Result<(), BridgeError> {
        let token = token.trim();
        if token.is_empty() {
            return Err(BridgeError::new(
                ErrorKind::InvalidRequest,
                "token must not be empty",
            ));
        }

        let path = self.path(server_id)?;
        ensure_dir(&self.dir)?;

        let tmp_path = path.with_extension("token.tmp");
        write_private(&tmp_path, token)?;
        fs::rename(&tmp_path, &path).map_err(|err| io_error("rename token", &path, err))?;
        Ok(())
    }

    pub fn delete(&self, server_id: &ServerId) -> Result<(), BridgeError> {
        let path = self.path(server_id)?;
        match fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(io_error("delete token", &path, err)),
        }
    }

    /// Server ids that currently have a stored token. Never returns tokens.
    pub fn list_servers(&self) -> Result<Vec<ServerId>, BridgeError> {
        list_stems(&self.dir, "token")
    }

    fn path(&self, server_id: &ServerId) -> Result<PathBuf, BridgeError> {
        validate_server_id(server_id)?;
        Ok(self.dir.join(format!("{}.token", server_id.as_str())))
    }
}

pub(crate) fn validate_server_id(server_id: &ServerId) -> Result<(), BridgeError> {
    let value = server_id.as_str();
    let ok = !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if ok {
        Ok(())
    } else {
        Err(BridgeError::new(
            ErrorKind::InvalidRequest,
            "server_id must contain only [A-Za-z0-9_-]",
        ))
    }
}

pub(crate) fn ensure_dir(dir: &Path) -> Result<(), BridgeError> {
    fs::create_dir_all(dir).map_err(|err| io_error("create dir", dir, err))
}

pub(crate) fn list_stems(dir: &Path, extension: &str) -> Result<Vec<ServerId>, BridgeError> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(io_error("list dir", dir, err)),
    };

    let mut ids = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| io_error("list entry", dir, err))?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some(extension) {
            continue;
        }
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            ids.push(ServerId::new(stem));
        }
    }
    ids.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    Ok(ids)
}

fn write_private(path: &Path, token: &str) -> Result<(), BridgeError> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }

    let mut file = options
        .open(path)
        .map_err(|err| io_error("open token", path, err))?;
    file.write_all(token.as_bytes())
        .map_err(|err| io_error("write token", path, err))?;
    file.sync_all()
        .map_err(|err| io_error("sync token", path, err))?;
    Ok(())
}

fn io_error(action: &str, path: &Path, err: std::io::Error) -> BridgeError {
    // Path may contain a server id but never a token; safe to include.
    BridgeError::new(
        ErrorKind::InternalError,
        format!("{action} failed at {}: {err}", path.display()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> (tempfile::TempDir, TokenStore) {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = TokenStore::new(dir.path());
        (dir, store)
    }

    #[test]
    fn missing_token_is_none() {
        let (_dir, store) = store();
        assert_eq!(store.load(&ServerId::new("work")).unwrap(), None);
    }

    #[test]
    fn save_load_roundtrip() {
        let (_dir, store) = store();
        let id = ServerId::new("work");
        store.save(&id, "secret-token").unwrap();
        assert_eq!(store.load(&id).unwrap().as_deref(), Some("secret-token"));
    }

    #[test]
    fn overwrite_replaces_token() {
        let (_dir, store) = store();
        let id = ServerId::new("work");
        store.save(&id, "old").unwrap();
        store.save(&id, "new").unwrap();
        assert_eq!(store.load(&id).unwrap().as_deref(), Some("new"));
    }

    #[test]
    fn delete_is_idempotent() {
        let (_dir, store) = store();
        let id = ServerId::new("work");
        store.delete(&id).unwrap();
        store.save(&id, "t").unwrap();
        store.delete(&id).unwrap();
        assert_eq!(store.load(&id).unwrap(), None);
    }

    #[test]
    fn rejects_bad_server_id() {
        let (_dir, store) = store();
        assert!(store.save(&ServerId::new("../evil"), "t").is_err());
        assert!(store.save(&ServerId::new("a/b"), "t").is_err());
        assert!(store.save(&ServerId::new(""), "t").is_err());
    }

    #[test]
    fn rejects_empty_token() {
        let (_dir, store) = store();
        assert!(store.save(&ServerId::new("work"), "   ").is_err());
    }

    #[test]
    fn lists_servers_without_tokens() {
        let (_dir, store) = store();
        store.save(&ServerId::new("zeta"), "t1").unwrap();
        store.save(&ServerId::new("alpha"), "t2").unwrap();
        let ids = store.list_servers().unwrap();
        let names: Vec<&str> = ids.iter().map(ServerId::as_str).collect();
        assert_eq!(names, vec!["alpha", "zeta"]);
    }

    #[cfg(unix)]
    #[test]
    fn token_file_is_private() {
        use std::os::unix::fs::PermissionsExt;
        let (dir, store) = store();
        let id = ServerId::new("work");
        store.save(&id, "t").unwrap();
        let path = dir.path().join("tokens/work.token");
        let mode = fs::metadata(path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
}
