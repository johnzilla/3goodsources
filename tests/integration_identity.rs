//! Integration tests for identity REST endpoints and MCP tool
//!
//! These tests validate the complete identity lookup functionality:
//! - GET /identities returns all identities as JSON object keyed by pubkey
//! - GET /identities/{pubkey} returns single identity or 404
//! - get_identity MCP tool via JSON-RPC
//! - MCP tool error handling for missing params and unknown pubkeys

mod common;

use serde_json::Value;

const TEST_PUBKEY: &str = "197f6b23e16c8532c6abc838facd5ea789be0c76b2920334039bfa8b3d368d61";

// ===== REST Endpoint Tests =====

#[tokio::test]
async fn test_get_identities_returns_all() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://{}/identities", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();

    assert!(body.is_object(), "Response should be a JSON object");
    assert!(
        body.get(TEST_PUBKEY).is_some(),
        "Should contain John Turner's identity keyed by pubkey"
    );
}

#[tokio::test]
async fn test_get_identities_has_correct_structure() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://{}/identities", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();

    let identity = &body[TEST_PUBKEY];
    assert!(identity["name"].is_string(), "Identity should have name");
    assert!(identity["type"].is_string(), "Identity should have type");
    assert!(identity["platforms"].is_array(), "Identity should have platforms array");
}

#[tokio::test]
async fn test_get_identity_by_pubkey_found() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://{}/identities/{}", addr, TEST_PUBKEY))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();

    assert_eq!(body["name"], "John Turner", "Should return John Turner's identity");
    assert_eq!(body["type"], "human");
}

#[tokio::test]
async fn test_get_identity_by_pubkey_has_platforms() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://{}/identities/{}", addr, TEST_PUBKEY))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();

    let platforms = body["platforms"].as_array().unwrap();
    assert_eq!(platforms.len(), 3, "Should have 3 platform claims (x, github, nostr)");

    let platform_names: Vec<&str> = platforms
        .iter()
        .map(|p| p["platform"].as_str().unwrap())
        .collect();
    assert!(platform_names.contains(&"x"));
    assert!(platform_names.contains(&"github"));
    assert!(platform_names.contains(&"nostr"));
}

#[tokio::test]
async fn test_get_identity_by_pubkey_not_found() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!(
            "http://{}/identities/0000000000000000000000000000000000000000000000000000000000000000",
            addr
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 404);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["error"], "Identity not found");
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
async fn test_get_identity_mcp_tool_found() {
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
                "name": "get_identity",
                "arguments": {"pubkey": TEST_PUBKEY}
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
    assert!(text.contains("John Turner"), "Should contain identity name");
    assert!(text.contains("human"), "Should contain identity type");
    assert!(text.contains("Platforms:"), "Should contain platforms section");
}

#[tokio::test]
async fn test_get_identity_mcp_tool_not_found() {
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
                "name": "get_identity",
                "arguments": {"pubkey": "0000000000000000000000000000000000000000000000000000000000000000"}
            }
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();

    assert_eq!(body["result"]["isError"], true);
    let text = body["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("No identity found"), "Should indicate identity not found");
}

#[tokio::test]
async fn test_get_identity_mcp_tool_missing_params() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    initialize_mcp(&client, &addr).await;

    let response = client
        .post(format!("http://{}/mcp", addr))
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "get_identity",
                "arguments": {}
            }
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();

    assert_eq!(body["error"]["code"], -32602, "Should return Invalid params error");
}
