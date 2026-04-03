use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::registry::types::Endorsement;
use super::types::{CachedPeer, PeerRegistry, PeerStatus};

/// Snapshot of a cached peer for read-only consumers.
/// Avoids holding the lock while callers process results.
#[derive(Debug, Clone)]
pub struct CachedPeerSnapshot {
    pub pubkey: String,
    pub url: String,
    pub name: Option<String>,
    pub registry: Option<PeerRegistry>,
    pub stale: bool,
    pub status: PeerStatus,
}

/// Cache of endorsed peer registries.
/// Filters out self-endorsements at construction time.
pub struct PeerCache {
    peers: RwLock<HashMap<String, CachedPeer>>,
    local_pubkey: String,
    client: reqwest::Client,
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

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            peers: RwLock::new(peers),
            local_pubkey,
            client,
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

    /// Fetch the /registry endpoint from a single peer and update its cached state.
    /// On success: sets status to Fresh, stores PeerRegistry, updates last_success.
    /// On failure: logs WARN, keeps existing registry, marks Stale if >1hr since last success.
    pub async fn fetch_peer(&self, pubkey: &str) {
        // Acquire read lock to get peer URL, then release before HTTP call
        let peer_url = {
            let peers = self.peers.read().await;
            peers.get(pubkey).map(|p| p.url.clone())
        };

        let url = match peer_url {
            Some(u) => u,
            None => {
                tracing::warn!(pubkey = %pubkey, "fetch_peer called for unknown pubkey");
                return;
            }
        };

        let registry_url = format!("{}/registry", url.trim_end_matches('/'));

        match self.client.get(&registry_url).send().await {
            Ok(response) => {
                match response.json::<PeerRegistry>().await {
                    Ok(parsed) => {
                        let mut peers = self.peers.write().await;
                        if let Some(peer) = peers.get_mut(pubkey) {
                            peer.registry = Some(parsed);
                            peer.last_success = Some(Instant::now());
                            peer.last_attempt = Some(Instant::now());
                            peer.status = PeerStatus::Fresh;
                            tracing::debug!(pubkey = %pubkey, url = %registry_url, "Peer registry fetched successfully");
                        }
                    }
                    Err(err) => {
                        tracing::warn!(pubkey = %pubkey, url = %registry_url, error = %err, "Failed to parse peer registry");
                        let mut peers = self.peers.write().await;
                        if let Some(peer) = peers.get_mut(pubkey) {
                            peer.last_attempt = Some(Instant::now());
                            let stale_threshold = Duration::from_secs(3600);
                            let is_stale = peer.last_success
                                .map(|t| t.elapsed() > stale_threshold)
                                .unwrap_or(true);
                            if is_stale {
                                peer.status = if peer.registry.is_some() {
                                    PeerStatus::Stale
                                } else {
                                    PeerStatus::Unreachable
                                };
                            }
                        }
                    }
                }
            }
            Err(err) => {
                tracing::warn!(pubkey = %pubkey, url = %registry_url, error = %err, "Failed to fetch peer registry");
                let mut peers = self.peers.write().await;
                if let Some(peer) = peers.get_mut(pubkey) {
                    peer.last_attempt = Some(Instant::now());
                    let stale_threshold = Duration::from_secs(3600);
                    let is_stale = peer.last_success
                        .map(|t| t.elapsed() > stale_threshold)
                        .unwrap_or(true);
                    if is_stale {
                        peer.status = if peer.registry.is_some() {
                            PeerStatus::Stale
                        } else {
                            PeerStatus::Unreachable
                        };
                    }
                }
            }
        }
    }

    /// Refresh all peers sequentially by fetching their /registry endpoints.
    pub async fn refresh_all(&self) {
        // Collect pubkeys while holding the read lock, then release
        let pubkeys: Vec<String> = {
            let peers = self.peers.read().await;
            peers.keys().cloned().collect()
        };

        tracing::info!(count = pubkeys.len(), "Refreshing peer cache ({} peers)", pubkeys.len());

        for pubkey in pubkeys {
            self.fetch_peer(&pubkey).await;
        }
    }

    /// Returns a snapshot of all cached peers.
    /// `stale` is true only when status == PeerStatus::Stale.
    pub async fn get_all_cached(&self) -> Vec<CachedPeerSnapshot> {
        let peers = self.peers.read().await;
        peers.values().map(|peer| CachedPeerSnapshot {
            pubkey: peer.pubkey.clone(),
            url: peer.url.clone(),
            name: peer.name.clone(),
            registry: peer.registry.clone(),
            stale: peer.status == PeerStatus::Stale,
            status: peer.status.clone(),
        }).collect()
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

    fn make_endorsement_with_name(pubkey: &str, url: &str, name: &str) -> Endorsement {
        Endorsement {
            pubkey: pubkey.to_string(),
            url: url.to_string(),
            name: Some(name.to_string()),
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

    // Task 2: Tests for networking data structures and state transitions

    #[tokio::test]
    async fn test_get_all_cached_empty() {
        let cache = PeerCache::new(vec![], "local-key".to_string());
        let snapshots = cache.get_all_cached().await;
        assert!(snapshots.is_empty());
    }

    #[tokio::test]
    async fn test_get_all_cached_returns_peers() {
        let endorsements = vec![
            make_endorsement("peer-a", "http://a.example.com"),
            make_endorsement("peer-b", "http://b.example.com"),
        ];
        let cache = PeerCache::new(endorsements, "local-key".to_string());
        let snapshots = cache.get_all_cached().await;
        assert_eq!(snapshots.len(), 2);
        for snap in &snapshots {
            assert_eq!(snap.status, PeerStatus::Unreachable);
            assert!(!snap.stale); // Unreachable != Stale
            assert!(snap.registry.is_none());
        }
    }

    #[tokio::test]
    async fn test_cached_peer_snapshot_fields() {
        let endorsements = vec![
            make_endorsement_with_name("peer-a", "http://a.example.com", "Alice"),
        ];
        let cache = PeerCache::new(endorsements, "local-key".to_string());
        let snapshots = cache.get_all_cached().await;
        assert_eq!(snapshots.len(), 1);
        let snap = &snapshots[0];
        assert_eq!(snap.pubkey, "peer-a");
        assert_eq!(snap.url, "http://a.example.com");
        assert_eq!(snap.name, Some("Alice".to_string()));
        assert!(!snap.stale);
        assert_eq!(snap.status, PeerStatus::Unreachable);
        assert!(snap.registry.is_none());
    }

    #[tokio::test]
    async fn test_peer_cache_has_client() {
        // Verifies PeerCache::new() succeeds (implicitly tests client creation doesn't panic)
        let cache = PeerCache::new(vec![], "local-key".to_string());
        assert_eq!(cache.peer_count().await, 0);
        assert_eq!(cache.local_pubkey(), "local-key");
    }

    #[tokio::test]
    async fn test_refresh_all_with_no_peers() {
        let cache = PeerCache::new(vec![], "local-key".to_string());
        // Should complete without panic
        cache.refresh_all().await;
        assert!(cache.get_all_cached().await.is_empty());
    }
}
