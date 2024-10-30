
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] rocksdb::Error),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Password hash error: {0}")]
    PasswordHashError(#[from] argon2::password_hash::Error),

    #[error("Image processing error: {0}")]
    Image(#[from] image::ImageError),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}



