use thiserror::Error;
use uuid::Uuid;

/// Audit log specific errors
#[derive(Debug, Error)]
pub enum AuditError {
    /// Failed to read audit log file
    #[error("Failed to read audit log file at {path}: {error}")]
    FileRead { path: String, error: String },

    /// Failed to parse JSON
    #[error("Failed to parse audit log JSON from {path}: {error}")]
    JsonParse { path: String, error: String },

    /// Invalid actor public key format
    #[error("Invalid actor public key in audit entry {id}")]
    InvalidActorKey { id: Uuid },

    /// Invalid signature format
    #[error("Invalid signature format in audit entry {id}")]
    InvalidSignature { id: Uuid },

    /// Signature verification failed
    #[error("Signature verification failed for audit entry {id}")]
    SignatureVerificationFailed { id: Uuid },

    /// Entry was signed by a key other than the server's own
    #[error("Audit entry {id} has actor {actual}, expected server key {expected}")]
    ActorMismatch {
        id: Uuid,
        expected: String,
        actual: String,
    },

    /// previous_hash doesn't match the hash of the preceding entry
    #[error("Audit entry {id} breaks the hash chain: previous_hash {actual:?}, expected {expected:?}")]
    ChainBroken {
        id: Uuid,
        expected: Option<String>,
        actual: Option<String>,
    },

    /// Entry timestamp is earlier than the entry before it
    #[error("Audit entry {id} is timestamped before the entry preceding it")]
    TimestampOutOfOrder { id: Uuid },

    /// Same entry id appears twice
    #[error("Audit entry id {id} appears more than once")]
    DuplicateId { id: Uuid },

    /// Rejected new entry (bad action, data, etc.)
    #[error("Invalid new audit entry: {0}")]
    InvalidEntry(String),
}
