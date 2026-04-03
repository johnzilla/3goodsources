use thiserror::Error;

/// Federation-specific errors
#[derive(Debug, Error)]
pub enum FederationError {
    /// Failed to fetch peer registry
    #[error("Failed to fetch peer registry from {url}: {reason}")]
    PeerFetchError { url: String, reason: String },

    /// Failed to parse peer registry JSON
    #[error("Failed to parse peer registry from {url}: {reason}")]
    PeerParseError { url: String, reason: String },

    /// Peer request timed out
    #[error("Peer request to {url} timed out after {timeout_secs}s")]
    PeerTimeout { url: String, timeout_secs: u64 },

    /// Self-endorsement detected
    #[error("Self-endorsement detected for pubkey {pubkey}")]
    SelfEndorsement { pubkey: String },
}
