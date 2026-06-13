use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("parse error at {key}: {source}")]
    Parse {
        key: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
