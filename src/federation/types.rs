use serde::Deserialize;
use std::collections::HashMap;
use std::time::Instant;
use crate::registry::types::{Category, Source};

/// Lax peer registry for forward-compatible federation deserialization.
/// No deny_unknown_fields — newer peers may add fields older nodes don't know about.
#[derive(Debug, Clone, Deserialize)]
pub struct PeerRegistry {
    pub version: String,
    pub updated: String,
    pub curator: PeerCurator,
    #[serde(default)]
    pub endorsements: Vec<PeerEndorsement>,
    #[serde(default)]
    pub categories: HashMap<String, Category>,
}

/// Lax curator type for peer data
#[derive(Debug, Clone, Deserialize)]
pub struct PeerCurator {
    pub name: String,
    pub pubkey: String,
}

/// Lax endorsement type for peer data (separate from local Endorsement per D-05)
#[derive(Debug, Clone, Deserialize)]
pub struct PeerEndorsement {
    pub pubkey: String,
    pub url: String,
    pub name: Option<String>,
    pub since: String,
}

/// A query match from a federated peer
#[derive(Debug, Clone)]
pub struct FederatedMatch {
    pub curator_name: String,
    pub curator_pubkey: String,
    pub source_url: String,
    pub trust: TrustLevel,
    pub stale: bool,
    pub slug: String,
    pub category_name: String,
    pub category_description: String,
    pub sources: Vec<Source>,
}

/// Trust level for federated results
#[derive(Debug, Clone, PartialEq)]
pub enum TrustLevel {
    /// Local node's own registry
    Direct,
    /// Endorsed peer's registry
    Endorsed,
}

/// Status of a cached peer
#[derive(Debug, Clone, PartialEq)]
pub enum PeerStatus {
    /// Successfully fetched within the last hour
    Fresh,
    /// Last successful fetch was more than 1 hour ago
    Stale,
    /// Never successfully fetched or repeated failures
    Unreachable,
}

/// A cached peer entry in the peer cache
#[derive(Debug, Clone)]
pub struct CachedPeer {
    pub pubkey: String,
    pub url: String,
    pub name: Option<String>,
    pub since: String,
    pub registry: Option<PeerRegistry>,
    pub last_success: Option<Instant>,
    pub last_attempt: Option<Instant>,
    pub status: PeerStatus,
}
