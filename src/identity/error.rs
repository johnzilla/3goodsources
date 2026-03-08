use thiserror::Error;

/// Identity-specific errors
#[derive(Debug, Error)]
pub enum IdentityError {
    /// Failed to read identities file
    #[error("Failed to read identities file at {path}: {error}")]
    FileRead { path: String, error: String },

    /// Failed to parse JSON
    #[error("Failed to parse identities JSON from {path}: {error}")]
    JsonParse { path: String, error: String },

    /// Bot identity has operator_pubkey pointing to invalid identity
    #[error("Bot identity {pubkey} has invalid operator {operator_pubkey}: must reference an existing human identity")]
    InvalidOperator {
        pubkey: String,
        operator_pubkey: String,
    },

    /// Bot identity is missing required operator_pubkey
    #[error("Bot identity {pubkey} is missing required operator_pubkey")]
    MissingOperator { pubkey: String },
}
