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
