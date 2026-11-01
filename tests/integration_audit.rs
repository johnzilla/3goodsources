//! Integration tests for audit log REST endpoint and MCP tool
//!
//! These tests validate the complete audit log functionality:
//! - GET /audit returns all 40 entries as raw JSON array
//! - Query filtering by action, category, since timestamp
//! - Combined filters
//! - Entry structure validation
//! - MCP get_audit_log tool via JSON-RPC

mod common;

use serde_json::Value;
use three_good_sources::audit::{self, hash_entry_json, AuditEntry};

/// Hex pubkey of the curator's production PKARR key (z32 qp6xytgxz4f1n3g3uiofktpbwuwzi89a4homcedbpb6ar4edbr5o).
const PRODUCTION_ACTOR: &str = "737cf044cfbe8b2164d99d605545a1a4e97a9ff8d720b62061687d8269030937";

// ===== Committed audit_log.json =====

#[tokio::test]
async fn test_committed_audit_log_signed_by_production_key() {
    let entries = audit::load(concat!(env!("CARGO_MANIFEST_DIR"), "/audit_log.json"))
        .await
        .expect("committed audit_log.json must load and verify");
    assert_eq!(entries.len(), 40);
    for entry in &entries {
        assert_eq!(
            entry.actor, PRODUCTION_ACTOR,
            "audit entry {} must be signed by the production key",
            entry.id
        );
    }
    audit::verify_actors(&entries, PRODUCTION_ACTOR).unwrap();
}

#[test]
fn test_committed_audit_log_hash_chain_intact() {
    let entries: Vec<AuditEntry> = serde_json::from_str(include_str!("../audit_log.json")).unwrap();
    let mut previous_hash: Option<String> = None;
    for entry in &entries {
        assert_eq!(entry.previous_hash, previous_hash, "chain broken at entry {}", entry.id);
        previous_hash = Some(hash_entry_json(entry));
    }
}

#[test]
fn test_startup_check_rejects_test_key_actor() {
    let entries: Vec<AuditEntry> = serde_json::from_str(include_str!("../audit_log.json")).unwrap();
    let test_key_actor = "197f6b23e16c8532c6abc838facd5ea789be0c76b2920334039bfa8b3d368d61";
    assert!(matches!(
        audit::verify_actors(&entries, test_key_actor),
        Err(audit::AuditError::ActorMismatch { .. })
    ));
}

// ===== REST Endpoint Tests =====

#[tokio::test]
async fn test_audit_returns_all_entries() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://{}/audit", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let entries: Vec<Value> = response.json().await.unwrap();
    assert_eq!(entries.len(), 40, "Should return all 40 audit entries");
}

#[tokio::test]
async fn test_audit_returns_json_array() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://{}/audit", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let content_type = response
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(content_type, "application/json");

    let text = response.text().await.unwrap();
    assert!(
        text.starts_with('['),
        "Response should be a raw JSON array, got: {}",
        &text[..50.min(text.len())]
    );
}

#[tokio::test]
async fn test_audit_filter_by_action_category_added() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://{}/audit?action=category_added", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let entries: Vec<Value> = response.json().await.unwrap();
    assert_eq!(entries.len(), 10, "Should return exactly 10 category_added entries");
}

#[tokio::test]
async fn test_audit_filter_by_action_source_added() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://{}/audit?action=source_added", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let entries: Vec<Value> = response.json().await.unwrap();
    assert_eq!(entries.len(), 30, "Should return exactly 30 source_added entries");
}

#[tokio::test]
async fn test_audit_filter_by_category() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://{}/audit?category=rust-learning", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let entries: Vec<Value> = response.json().await.unwrap();
    assert_eq!(
        entries.len(),
        4,
        "Should return 4 entries for rust-learning (1 category_added + 3 source_added)"
    );
}

#[tokio::test]
async fn test_audit_combined_filters() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!(
            "http://{}/audit?category=rust-learning&action=source_added",
            addr
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let entries: Vec<Value> = response.json().await.unwrap();
    assert_eq!(
        entries.len(),
        3,
        "Should return exactly 3 source_added entries for rust-learning"
    );
}

#[tokio::test]
async fn test_audit_filter_since_future() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!(
            "http://{}/audit?since=2026-02-04T00:00:00Z",
            addr
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let entries: Vec<Value> = response.json().await.unwrap();
    assert_eq!(
        entries.len(),
        0,
        "All entries are 2026-02-03, so since 2026-02-04 should return 0"
    );
}

#[tokio::test]
async fn test_audit_filter_since_past() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!(
            "http://{}/audit?since=2026-02-02T00:00:00Z",
            addr
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let entries: Vec<Value> = response.json().await.unwrap();
    assert_eq!(
        entries.len(),
        40,
        "All entries are after 2026-02-02, so should return all 40"
    );
}

#[tokio::test]
async fn test_audit_entry_structure() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://{}/audit", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let entries: Vec<Value> = response.json().await.unwrap();

    let first = &entries[0];
    assert!(first["id"].is_string(), "Entry should have string id");
    assert!(first["timestamp"].is_string(), "Entry should have string timestamp");
    assert!(first["action"].is_string(), "Entry should have string action");
    assert!(first["actor"].is_string(), "Entry should have string actor");
    assert!(first["signature"].is_string(), "Entry should have string signature");

    // First entry should have null previous_hash (start of chain)
    assert!(
        first["previous_hash"].is_null(),
        "First entry should have null previous_hash"
    );

    // First entry should be category_added
    assert_eq!(
        first["action"], "category_added",
        "First entry should be category_added"
    );
}

// ===== MCP Tool Tests =====

/// Helper to initialize MCP handler
async fn initialize_mcp(client: &reqwest::Client, addr: &std::net::SocketAddr) {
    client
        .post(format!("http://{}/mcp", addr))
        .json(&serde_json::json!({
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
}

#[tokio::test]
async fn test_audit_mcp_get_audit_log() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    initialize_mcp(&client, &addr).await;

    let response = client
        .post(format!("http://{}/mcp", addr))
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "get_audit_log",
                "arguments": {}
            }
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();

    assert_eq!(body["result"]["isError"], false);
    assert!(body["result"]["content"].is_array());

    let text = body["result"]["content"][0]["text"].as_str().unwrap();
    assert!(
        text.contains("40 entries"),
        "Should mention 40 entries in response, got: {}",
        &text[..100.min(text.len())]
    );
}

#[tokio::test]
async fn test_audit_mcp_get_audit_log_filtered() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    initialize_mcp(&client, &addr).await;

    let response = client
        .post(format!("http://{}/mcp", addr))
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "get_audit_log",
                "arguments": {"category": "rust-learning"}
            }
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();

    assert_eq!(body["result"]["isError"], false);

    let text = body["result"]["content"][0]["text"].as_str().unwrap();
    assert!(
        text.contains("4 entries"),
        "Should mention 4 entries for rust-learning filter, got: {}",
        &text[..100.min(text.len())]
    );
}
