//! Integration tests for registry signing
//!
//! These tests validate:
//! - The committed registry.json is signed by the production curator key
//! - GET /registry serves the signed document verbatim, including its signature
//! - get_provenance exposes the signature and verification instructions
//! - Startup verification rejects unsigned and foreign-key registries
//! - Peer fetching marks unsigned, tampered, and foreign-key peers as InvalidSignature

mod common;

use axum::{Router, extract::State, routing::get};
use serde_json::{Value, json};
use std::net::SocketAddr;
use std::sync::Arc;
use three_good_sources::federation::{PeerCache, PeerStatus};
use three_good_sources::registry::types::Endorsement;
use three_good_sources::registry::signing::{SignatureError, sign_document, verify_raw};
use tokio::net::TcpListener;
use tokio::sync::RwLock;

/// Production curator pubkey for 3gs.ai (z-base-32)
const PRODUCTION_PUBKEY: &str = "qp6xytgxz4f1n3g3uiofktpbwuwzi89a4homcedbpb6ar4edbr5o";

const REGISTRY_JSON: &str = include_str!("../registry.json");

// ===== Committed registry =====

#[test]
fn test_committed_registry_signed_by_production_key() {
    // If this fails after editing registry.json, re-sign it: `cargo run -- sign-registry`
    let signature = verify_raw(REGISTRY_JSON, PRODUCTION_PUBKEY)
        .expect("registry.json must be signed by the production curator key");
    assert_eq!(signature.alg, "ed25519");
    assert_eq!(signature.sig.len(), 128);
}

#[test]
fn test_startup_check_rejects_unsigned_fixture() {
    let fixture = include_str!("fixtures/valid_registry.json");
    let doc: Value = serde_json::from_str(fixture).unwrap();
    let pubkey = doc["curator"]["pubkey"].as_str().unwrap();
    assert_eq!(verify_raw(fixture, pubkey), Err(SignatureError::Missing));
}

#[test]
fn test_startup_check_rejects_other_server_key() {
    let other = pkarr::Keypair::from_secret_key(&[3u8; 32]).public_key().to_z32();
    assert!(matches!(
        verify_raw(REGISTRY_JSON, &other),
        Err(SignatureError::PubkeyMismatch { .. })
    ));
}

#[test]
fn test_committed_registry_tamper_detected() {
    let tampered = REGISTRY_JSON.replacen("\"rank\": 1", "\"rank\": 2", 1);
    assert_ne!(tampered, REGISTRY_JSON);
    assert_eq!(verify_raw(&tampered, PRODUCTION_PUBKEY), Err(SignatureError::Invalid));
}

// ===== REST endpoint =====

#[tokio::test]
async fn test_registry_endpoint_serves_signed_document() {
    let addr = common::spawn_test_server().await;

    let body = reqwest::get(format!("http://{}/registry", addr))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert_eq!(body, REGISTRY_JSON, "GET /registry must serve registry.json verbatim");
    let doc: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(doc["signature"]["alg"], "ed25519");
    assert!(verify_raw(&body, PRODUCTION_PUBKEY).is_ok());
}

// ===== MCP tool =====

#[tokio::test]
async fn test_get_provenance_includes_signature() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    client
        .post(format!("http://{}/mcp", addr))
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": {"name": "test-client", "version": "1.0"}
            }
        }))
        .send()
        .await
        .unwrap();

    let body: Value = client
        .post(format!("http://{}/mcp", addr))
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {"name": "get_provenance", "arguments": {}}
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let text = body["result"]["content"][0]["text"].as_str().unwrap();
    let doc: Value = serde_json::from_str(REGISTRY_JSON).unwrap();
    let sig = doc["signature"]["sig"].as_str().unwrap();

    assert!(text.contains(&format!("Signature: ed25519 {}", sig)));
    assert!(text.contains(&format!("Curator Public Key: {}", PRODUCTION_PUBKEY)));
    assert!(text.contains("3gs-registry-v1"));
    assert!(!text.contains("not yet signed"));
}

// ===== Peer verification =====

const PEER_SECRET: [u8; 32] = [5u8; 32];

fn peer_pubkey() -> String {
    pkarr::Keypair::from_secret_key(&PEER_SECRET).public_key().to_z32()
}

/// A minimal peer registry signed with `secret`, claiming `curator_pubkey`.
fn peer_registry(curator_pubkey: &str, secret: &[u8; 32]) -> String {
    let mut doc = json!({
        "version": "0.1.0",
        "updated": "2026-11-01",
        "curator": {"name": "Peer", "pubkey": curator_pubkey},
        "endorsements": [],
        "categories": {
            "rust-learning": {
                "name": "Rust Learning",
                "description": "Learning Rust",
                "query_patterns": ["learn rust", "rust tutorial", "rust book"],
                "sources": [
                    {"rank": 1, "name": "A", "url": "https://a.example", "type": "book", "why": "a"},
                    {"rank": 2, "name": "B", "url": "https://b.example", "type": "book", "why": "b"},
                    {"rank": 3, "name": "C", "url": "https://c.example", "type": "book", "why": "c"}
                ]
            }
        },
        "future_field": {"unknown": true}
    });
    let signature = sign_document(&doc, secret).unwrap();
    doc["signature"] = serde_json::to_value(signature).unwrap();
    serde_json::to_string_pretty(&doc).unwrap()
}

/// Spawn a mock peer whose /registry body can be swapped between fetches.
async fn spawn_mock_peer(body: String) -> (SocketAddr, Arc<RwLock<String>>) {
    let state = Arc::new(RwLock::new(body));
    let app = Router::new()
        .route(
            "/registry",
            get(|State(body): State<Arc<RwLock<String>>>| async move { body.read().await.clone() }),
        )
        .with_state(Arc::clone(&state));

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (addr, state)
}

fn cache_for(addr: SocketAddr) -> PeerCache {
    let endorsement = Endorsement {
        pubkey: peer_pubkey(),
        url: format!("http://{}", addr),
        name: Some("Peer".to_string()),
        since: "2026-11-01".to_string(),
    };
    PeerCache::new(vec![endorsement], "local-key".to_string())
}

async fn fetch_and_snapshot(cache: &PeerCache) -> three_good_sources::federation::CachedPeerSnapshot {
    cache.fetch_peer(&peer_pubkey()).await;
    cache.get_all_cached().await.into_iter().next().unwrap()
}

#[tokio::test]
async fn test_peer_with_valid_signature_is_fresh() {
    let (addr, _) = spawn_mock_peer(peer_registry(&peer_pubkey(), &PEER_SECRET)).await;
    let cache = cache_for(addr);

    let snap = fetch_and_snapshot(&cache).await;
    assert_eq!(snap.status, PeerStatus::Fresh);
    let registry = snap.registry.expect("verified registry should be cached");
    assert!(registry.categories.contains_key("rust-learning"));
}

#[tokio::test]
async fn test_unsigned_peer_is_invalid() {
    let mut doc: Value = serde_json::from_str(&peer_registry(&peer_pubkey(), &PEER_SECRET)).unwrap();
    doc.as_object_mut().unwrap().remove("signature");
    let (addr, _) = spawn_mock_peer(doc.to_string()).await;
    let cache = cache_for(addr);

    let snap = fetch_and_snapshot(&cache).await;
    assert_eq!(snap.status, PeerStatus::InvalidSignature);
    assert!(snap.registry.is_none());
}

#[tokio::test]
async fn test_tampered_peer_is_invalid() {
    let tampered = peer_registry(&peer_pubkey(), &PEER_SECRET).replace("https://a.example", "https://evil.example");
    let (addr, _) = spawn_mock_peer(tampered).await;
    let cache = cache_for(addr);

    let snap = fetch_and_snapshot(&cache).await;
    assert_eq!(snap.status, PeerStatus::InvalidSignature);
    assert!(snap.registry.is_none());
}

#[tokio::test]
async fn test_peer_signed_by_other_key_is_invalid() {
    // Validly self-signed, but by a key other than the one we endorsed
    let impostor_secret = [6u8; 32];
    let impostor_pubkey = pkarr::Keypair::from_secret_key(&impostor_secret).public_key().to_z32();
    let (addr, _) = spawn_mock_peer(peer_registry(&impostor_pubkey, &impostor_secret)).await;
    let cache = cache_for(addr);

    let snap = fetch_and_snapshot(&cache).await;
    assert_eq!(snap.status, PeerStatus::InvalidSignature);
    assert!(snap.registry.is_none());
}

#[tokio::test]
async fn test_peer_that_turns_bad_is_dropped() {
    let good = peer_registry(&peer_pubkey(), &PEER_SECRET);
    let (addr, body) = spawn_mock_peer(good.clone()).await;
    let cache = cache_for(addr);

    assert_eq!(fetch_and_snapshot(&cache).await.status, PeerStatus::Fresh);

    *body.write().await = good.replace("\"Peer\"", "\"Mallory\"");
    let snap = fetch_and_snapshot(&cache).await;
    assert_eq!(snap.status, PeerStatus::InvalidSignature);
    assert!(snap.registry.is_none(), "previously cached data must not be trusted");

    *body.write().await = good;
    assert_eq!(fetch_and_snapshot(&cache).await.status, PeerStatus::Fresh);
}
