use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level registry structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Registry {
    /// Semver version string (e.g., "0.1.0")
    pub version: String,
    /// ISO 8601 date string (e.g., "2026-02-01")
    pub updated: String,
    /// Registry curator information
    pub curator: Curator,
    /// Endorsements (required field, can be empty)
    pub endorsements: Vec<Endorsement>,
    /// Categories keyed by slug (e.g., "rust-learning")
    pub categories: HashMap<String, Category>,
}

/// Curator identity information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Curator {
    /// Curator display name
    pub name: String,
    /// PKARR public key
    pub pubkey: String,
}

/// Endorsement placeholder for future use
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Endorsement {}

/// Category containing sources for a specific topic
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Category {
    /// Category display name
    pub name: String,
    /// Category description
    pub description: String,
    /// Query patterns for matching user requests
    pub query_patterns: Vec<String>,
    /// List of curated sources (exactly 3 for v1)
    pub sources: Vec<Source>,
}

/// Individual source within a category
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    /// Rank order (1-3 for three sources)
    pub rank: u8,
    /// Source display name
    pub name: String,
    /// Source URL
    pub url: String,
    /// Source type
    #[serde(rename = "type")]
    pub source_type: SourceType,
    /// Curator's explanation of why this source is valuable
    pub why: String,
}

/// Source type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    Documentation,
    Tutorial,
    Video,
    Article,
    Tool,
    Repo,
    Forum,
    Book,
    Course,
    Api,
}
