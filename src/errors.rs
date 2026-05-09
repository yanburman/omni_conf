use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Platform not supported for automatic path resolution")]
    UnsupportedPlatform,

    #[error("I/O Error at {path:?}: {source}")]
    Io {
        path: std::path::PathBuf,
        source: std::io::Error,
    },

    #[error("Serialization Error: {0}")]
    Serialization(String),

    #[error("Deserialization Error: {0}")]
    Deserialization(String),

    #[error("Feature not enabled: {0}")]
    FeatureMissing(String),
}

pub type Result<T> = std::result::Result<T, ConfigError>;
