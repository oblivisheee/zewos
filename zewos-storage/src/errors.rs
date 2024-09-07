use thiserror::Error;
#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Failed to insert fragment: {0}")]
    InsertionError(String),
    #[error("Failed to load from backup: {0}")]
    BackupLoadError(String),
}

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Compression error: {0}")]
    Compression(String),
    #[error("Decompression error: {0}")]
    Decompression(String),
    #[error("Key not found")]
    KeyNotFound,
    #[error("Version not found")]
    VersionNotFound,
    #[error("Backup error: {0}")]
    BackupError(#[from] BackupError),
    #[error("Fragment error: {0}")]
    ObjectError(#[from] ObjectError),
    #[error("Cache error: {0}")]
    CacheError(#[from] CacheError),
}

#[derive(Error, Debug)]
pub enum BackupError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Deserialization error: {0}")]
    DeserializationError(#[from] bincode::Error),
    #[error("Fragment error: {0}")]
    ObjectError(#[from] ObjectError),
    #[error("No versions found")]
    NoVersionsFound,
}

#[derive(Debug, Error)]
pub enum ObjectError {
    #[error("Invalid name: {0}")]
    InvalidName(String),
    #[error("Invalid size: {0}")]
    InvalidSize(usize),
    #[error("Invalid data")]
    InvalidData,
    #[error("Serialization error: {0}")]
    SerializeError(#[from] bincode::Error),
}
