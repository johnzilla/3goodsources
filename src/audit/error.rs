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
}
