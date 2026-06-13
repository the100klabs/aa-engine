use thiserror::Error;

/// Errors raised while loading or validating AA Engine assets.
#[derive(Debug, Error)]
pub enum AssetError {
    #[error("IO error reading asset: {0}")]
    Io(#[from] std::io::Error),

    #[error("RON parse error: {0}")]
    Ron(#[from] ron::error::SpannedError),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("unsupported schema version {found}, expected {expected}")]
    UnsupportedSchemaVersion { found: u32, expected: u32 },

    #[error("unknown gameplay tag: {0}")]
    UnknownTag(String),
}
