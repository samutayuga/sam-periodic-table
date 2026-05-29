//! Data-loading errors.

/// Error returned while loading or validating element data.
#[derive(Debug, thiserror::Error)]
pub enum DataError {
    #[error("failed to read {path}: {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse {file}: {source}")]
    Parse {
        file: String,
        source: serde_yaml_ng::Error,
    },
    #[error("validation error in {file}: {message}")]
    Validation { file: String, message: String },
    #[error("duplicate atomic number {0}")]
    DuplicateAtomicNumber(u8),
    #[error("duplicate symbol {0}")]
    DuplicateSymbol(String),
    #[error("duplicate name {0}")]
    DuplicateName(String),
    #[error("no element files found in {0}")]
    EmptyDataDir(String),
}
