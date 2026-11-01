//! Integration tests for audit log REST endpoint and MCP tool
//!
//! These tests validate the complete audit log functionality:
//! - Committed audit_log.json: production key, intact hash chain
//! - Append-only: new entries chain onto the log, tampering is detected
//! - GET /audit returns all entries as raw JSON array
//! - Query filtering by action, category, since timestamp
//! - Combined filters
//! - Entry structure validation
//! - MCP get_audit_log tool via JSON-RPC

mod common;

use serde_json::Value;
use three_good_sources::audit::{self, hash_entry_json, AuditAction, AuditEntry, AuditFilterParams};
use three_good_sources::audit_append::append_to_contents;

/// Hex pubkey of the curator's production PKARR key (z32 qp6xytgxz4f1n3g3uiofktpbwuwzi89a4homcedbpb6ar4edbr5o).
const PRODUCTION_ACTOR: &str = "737cf044cfbe8b2164d99d605545a1a4e97a9ff8d720b62061687d8269030937";

/// Entries in the committed audit_log.json
fn committed() -> Vec<AuditEntry> {
    serde_json::from_str(include_str!("../audit_log.json")).unwrap()
}

/// How many committed entries match the given filters. Counts are derived from the file so
/// appending entries with `3gs audit-append` doesn't break these tests.
fn count(action: Option<&str>, category: Option<&str>, since: Option<&str>) -> usize {
    let params = AuditFilterParams {
        action: action.map(String::from),
        category: category.map(String::from),
        since: since.map(String::from),
    };
    audit::filter_entries(&committed(), &params).len()
}

// ===== Committed audit_log.json =====

#[tokio::test]
async fn test_committed_audit_log_signed_by_production_key() {
    let entries = audit::load(concat!(env!("CARGO_MANIFEST_DIR"), "/audit_log.json"))
        .await
        .expect("committed audit_log.json must load and verify");
    assert!(entries.len() >= 40, "the 40 retroactive entries must never be removed");
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

#[tokio::test]
async fn test_committed_audit_log_chain_verifies() {
    // Signatures cover previous_hash, so the chain itself is signed
    audit::verify_entries(&committed()).expect("committed audit log chain must verify");
}

#[test]
fn test_committed_audit_log_deleted_entry_detected() {
    let mut entries = committed();
    entries.remove(5);
    assert!(matches!(
        audit::verify_entries(&entries),
        Err(audit::AuditError::ChainBroken { .. })
    ));
}

#[test]
fn test_committed_audit_log_relinked_chain_detected() {
    // Hide a deleted entry by patching previous_hash: the signature no longer matches
    let mut entries = committed();
    entries.remove(5);
    entries[5].previous_hash = Some(hash_entry_json(&entries[4]));
    assert!(matches!(
        audit::verify_entries(&entries),
        Err(audit::AuditError::SignatureVerificationFailed { .. })
    ));
}

// ===== Append-only writes =====

#[tokio::test]
async fn test_appended_log_loads_and_keeps_history() {
    let key = ed25519_dalek::SigningKey::from_bytes(&[21u8; 32]);
    let (first, _) = append_to_contents(
        "[]",
        &key,
        AuditAction::CategoryAdded,
        Some("rust-learning".to_string()),
        serde_json::json!({"name": "Rust Learning"}),
    )
    .unwrap();
    let (second, added) = append_to_contents(
        &first,
        &key,
        AuditAction::SourceUpdated,
        Some("rust-learning".to_string()),
        serde_json::json!({"rank": 2}),
    )
    .unwrap();

    // Earlier bytes are untouched
    assert!(second.starts_with(first.trim_end().strip_suffix(']').unwrap().trim_end()));

    let tmp = std::env::temp_dir().join(format!("audit_append_{}.json", added.id));
    tokio::fs::write(&tmp, &second).await.unwrap();
    let loaded = audit::load(&tmp).await.expect("appended log must load");
    let _ = tokio::fs::remove_file(&tmp).await;

    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[1].id, added.id);
    assert_eq!(loaded[1].previous_hash, Some(hash_entry_json(&loaded[0])));
}

#[test]
fn test_append_refuses_committed_log_with_other_key() {
    // Only the production key may extend the committed log
    let key = ed25519_dalek::SigningKey::from_bytes(&[21u8; 32]);
    let err = append_to_contents(
        include_str!("../audit_log.json"),
        &key,
        AuditAction::SourceAdded,
        None,
        serde_json::json!({}),
    )
    .unwrap_err();
    assert!(err.contains("Refusing to append"), "got: {}", err);
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
    assert_eq!(entries.len(), committed().len(), "Should return all audit entries");
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
    assert_eq!(entries.len(), count(Some("category_added"), None, None));
    assert!(entries.len() >= 10, "Should include the 10 original category_added entries");
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
    assert_eq!(entries.len(), count(Some("source_added"), None, None));
    assert!(entries.len() >= 30, "Should include the 30 original source_added entries");
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
    assert_eq!(entries.len(), count(None, Some("rust-learning"), None));
    assert!(entries.len() >= 4, "Should include rust-learning's 1 category_added + 3 source_added");
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
    assert_eq!(entries.len(), count(Some("source_added"), Some("rust-learning"), None));
    assert!(entries.len() >= 3, "Should include rust-learning's 3 original sources");
}

#[tokio::test]
async fn test_audit_filter_since_future() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!(
            "http://{}/audit?since=2100-01-01T00:00:00Z",
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
        "No entries are from 2100 or later"
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
        committed().len(),
        "All entries are after 2026-02-02, so should return all of them"
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
    let total = committed().len();
    assert!(
        text.contains(&format!("{} entries", total)),
        "Should mention {} entries in response, got: {}",
        total,
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
    let expected = count(None, Some("rust-learning"), None);
    assert!(
        text.contains(&format!("{} entries", expected)),
        "Should mention {} entries for rust-learning filter, got: {}",
        expected,
        &text[..100.min(text.len())]
    );
}
