use thiserror::Error;

#[derive(Error, Debug)]
pub enum SignatureError {
    #[error("Failed to sign metadata: {0}")]
    SigningError(#[from] ecdsa::Error),
    #[error("Failed to verify metadata")]
    InvalidSignature,
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    #[error("Signature verification failed: {0}")]
    VerificationFailed(String),
    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),
    #[error("Missing data")]
    MissingData,
}

#[derive(Error, Debug)]
pub enum KeypairError {
    #[error("Failed to generate keypair: {0}")]
    KeypairError(#[from] ecdsa::Error),
    #[error("Verification failed")]
    ErrorVerify,
    #[error("Failed to serialize keypair")]
    SerializationError,
    #[error("Failed to deserialize keypair")]
    DeserializationError,
    #[error("Invalid keypair format")]
    InvalidFormat,
    #[error("Keypair not found")]
    KeypairNotFound,
}
