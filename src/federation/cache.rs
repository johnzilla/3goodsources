use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::registry::types::Endorsement;
use super::types::{CachedPeer, PeerStatus};

/// Cache of endorsed peer registries.
/// Filters out self-endorsements at construction time.
/// Phase 16 will add reqwest::Client, refresh_loop(), and fetch logic.
pub struct PeerCache {
    peers: RwLock<HashMap<String, CachedPeer>>,
    local_pubkey: String,
}

impl PeerCache {
    /// Create a new PeerCache from endorsements, filtering out self-endorsements.
    /// Self-endorsements (where endorsement.pubkey == local_pubkey) are logged at WARN level.
    pub fn new(endorsements: Vec<Endorsement>, local_pubkey: String) -> Self {
        let mut peers = HashMap::new();

        for endorsement in endorsements {
            if endorsement.pubkey == local_pubkey {
                tracing::warn!(
                    pubkey = %endorsement.pubkey,
                    "Self-endorsement detected and filtered from peer cache"
                );
                continue;
            }

            let cached = CachedPeer {
                pubkey: endorsement.pubkey.clone(),
                url: endorsement.url.clone(),
                name: endorsement.name.clone(),
                since: endorsement.since.clone(),
                registry: None,
                last_success: None,
                last_attempt: None,
                status: PeerStatus::Unreachable,
            };

            peers.insert(endorsement.pubkey.clone(), cached);
        }

        Self {
            peers: RwLock::new(peers),
            local_pubkey,
        }
    }

    /// Returns the number of peers in the cache (excluding self)
    pub async fn peer_count(&self) -> usize {
        self.peers.read().await.len()
    }

    /// Returns the local node's pubkey
    pub fn local_pubkey(&self) -> &str {
        &self.local_pubkey
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::types::Endorsement;

    fn make_endorsement(pubkey: &str, url: &str) -> Endorsement {
        Endorsement {
            pubkey: pubkey.to_string(),
            url: url.to_string(),
            name: None,
            since: "2026-04-03".to_string(),
        }
    }

    #[tokio::test]
    async fn test_empty_endorsements() {
        let cache = PeerCache::new(vec![], "local-key".to_string());
        assert_eq!(cache.peer_count().await, 0);
    }

    #[tokio::test]
    async fn test_no_self_endorsement() {
        let endorsements = vec![
            make_endorsement("peer-a", "http://a.example.com"),
            make_endorsement("peer-b", "http://b.example.com"),
        ];
        let cache = PeerCache::new(endorsements, "local-key".to_string());
        assert_eq!(cache.peer_count().await, 2);
    }

    #[tokio::test]
    async fn test_self_endorsement_filtered() {
        let endorsements = vec![
            make_endorsement("peer-a", "http://a.example.com"),
            make_endorsement("local-key", "http://self.example.com"),
            make_endorsement("peer-b", "http://b.example.com"),
        ];
        let cache = PeerCache::new(endorsements, "local-key".to_string());
        assert_eq!(cache.peer_count().await, 2);
    }

    #[tokio::test]
    async fn test_only_self_endorsement() {
        let endorsements = vec![make_endorsement("local-key", "http://self.example.com")];
        let cache = PeerCache::new(endorsements, "local-key".to_string());
        assert_eq!(cache.peer_count().await, 0);
    }

    #[tokio::test]
    async fn test_peer_count_after_filtering() {
        let endorsements = vec![
            make_endorsement("peer-a", "http://a.example.com"),
            make_endorsement("local-key", "http://self.example.com"),
            make_endorsement("peer-b", "http://b.example.com"),
            make_endorsement("peer-c", "http://c.example.com"),
        ];
        let cache = PeerCache::new(endorsements, "local-key".to_string());
        assert_eq!(cache.peer_count().await, 3);
    }
}
