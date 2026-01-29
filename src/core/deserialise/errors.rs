//! Error types for deserialization operations.

use thiserror::Error;

/// Errors that can occur during configuration deserialization.
#[derive(Debug, Error)]
pub enum DeserializeError {
    /// I/O error reading the configuration file.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON deserialization error.
    #[cfg(feature = "json-config")]
    #[error("JSON deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    /// YAML deserialization error.
    #[cfg(feature = "yaml-config")]
    #[error("YAML deserialization error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// Backend error during metric registration.
    #[error("Backend registration error: {0}")]
    BackendError(String),

    /// Invalid file path.
    #[error("Invalid file path: {0}")]
    InvalidFilePath(String),

    /// Unsupported file type.
    #[error("Unsupported file type: {0}")]
    UnsupportedFileType(String),

    /// Path contains or is a symlink; symlinks are not allowed.
    #[error("Symlinks are not allowed: {0}")]
    SymlinkNotAllowed(String),

    /// Path is outside allowed directories (e.g. XDG config, current dir).
    #[error("Path outside allowed directory: {0}")]
    PathOutsideAllowedDirectory(String),

    /// Format requires a feature that is not enabled (e.g. enable json-config to load JSON).
    #[error("Feature not enabled: {0}")]
    FeatureNotEnabled(String),
}

// Helper trait to convert backend errors to DeserializeError
pub trait BackendErrorExt {
    fn into_deserialize_error(self) -> DeserializeError;
}

impl<E: std::error::Error + Send + Sync + 'static> BackendErrorExt for E {
    fn into_deserialize_error(self) -> DeserializeError {
        DeserializeError::BackendError(self.to_string())
    }
}
