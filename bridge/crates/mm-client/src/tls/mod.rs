use crate::error::ClientError;
use reqwest::Certificate;
use std::fs;
use std::path::Path;

/// Build a reqwest ClientBuilder with optional explicit CA.
pub fn builder(ca_file: Option<&Path>) -> Result<reqwest::ClientBuilder, ClientError> {
    let mut builder = reqwest::Client::builder();

    if let Some(path) = ca_file {
        let cert_bytes = fs::read(path).map_err(|e| {
            ClientError::Tls(format!("Failed to read CA file {}: {}", path.display(), e))
        })?;
        let cert = Certificate::from_pem(&cert_bytes).map_err(|e| {
            ClientError::Tls(format!(
                "Invalid CA certificate in {}: {}",
                path.display(),
                e
            ))
        })?;
        builder = builder.add_root_certificate(cert);
    }

    Ok(builder)
}
