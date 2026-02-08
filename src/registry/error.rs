use thiserror::Error;

/// Registry-specific errors
#[derive(Debug, Error)]
pub enum RegistryError {
    /// Failed to read registry file
    #[error("Failed to read registry file at {path}: {error}")]
    FileRead { path: String, error: String },

    /// Failed to parse JSON
    #[error("Failed to parse JSON from {path} at line {line}, column {column}: {error}")]
    JsonParse {
        path: String,
        error: String,
        line: usize,
        column: usize,
    },

    /// Invalid category slug format
    #[error("Invalid category slug '{slug}': must be lowercase alphanumeric with hyphens")]
    InvalidSlug { slug: String },

    /// Source count doesn't match expected count
    #[error("Category '{category}' has {actual} sources, expected {expected}")]
    InvalidSourceCount {
        category: String,
        expected: usize,
        actual: usize,
    },

    /// Insufficient query patterns
    #[error("Category '{category}' has {actual} query patterns, minimum {minimum} required")]
    InsufficientQueryPatterns {
        category: String,
        minimum: usize,
        actual: usize,
    },

    /// Invalid source ranks (must be sequential 1, 2, 3)
    #[error("Category '{category}' has invalid ranks: {actual:?}, expected sequential ranks 1-{expected}")]
    InvalidRanks {
        category: String,
        actual: Vec<u8>,
        expected: usize,
    },
}
