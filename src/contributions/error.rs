use thiserror::Error;

/// Contribution-specific errors
#[derive(Debug, Error)]
pub enum ContributionError {
    /// Failed to read contributions file
    #[error("Failed to read contributions file at {path}: {error}")]
    FileRead { path: String, error: String },

    /// Failed to parse JSON
    #[error("Failed to parse contributions JSON from {path}: {error}")]
    JsonParse { path: String, error: String },

    /// Vote references a pubkey not found in identities
    #[error("Unknown voter pubkey {voter_pubkey} in proposal {proposal_id}")]
    UnknownVoter {
        voter_pubkey: String,
        proposal_id: String,
    },
}
