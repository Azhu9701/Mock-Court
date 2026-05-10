use thiserror::Error;

#[derive(Debug, Error)]
pub enum FoundationError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Config error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("UUID error: {0}")]
    Uuid(#[from] uuid::Error),

    #[error("Soul not found: {0}")]
    SoulNotFound(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Duplicate soul: {0}")]
    DuplicateSoul(String),

    #[error("Archive inconsistent: expected {expected} files, got {actual}")]
    ArchiveInconsistent { expected: u32, actual: u32 },
}

pub type Result<T> = std::result::Result<T, FoundationError>;
