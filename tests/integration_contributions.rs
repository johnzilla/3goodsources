//! Integration tests for community contributions REST endpoints and MCP tools
//!
//! These tests validate the complete contributions functionality:
//! - GET /proposals returns filtered proposal summaries
//! - GET /proposals/{id} returns full proposal detail with votes
//! - list_proposals MCP tool via JSON-RPC
//! - get_proposal MCP tool via JSON-RPC
//! - MCP tools/list returns 8 tools

mod common;

use serde_json::Value;

const DEMO_UUID: &str = "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d";

// ===== REST Endpoint Tests =====

#[tokio::test]
async fn test_proposals_list_returns_json() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://{}/proposals", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Vec<Value> = response.json().await.unwrap();

    assert_eq!(body.len(), 1, "Should return 1 proposal summary");

    let first = &body[0];
    assert!(first.get("id").is_some(), "Should have id field");
    assert!(first.get("action").is_some(), "Should have action field");
    assert!(first.get("status").is_some(), "Should have status field");
    assert!(first.get("category").is_some(), "Should have category field");
    assert!(first.get("proposer").is_some(), "Should have proposer field");
    assert!(first.get("created_at").is_some(), "Should have created_at field");
    assert!(first.get("votes").is_none(), "Should NOT have votes in summary");
}

#[tokio::test]
async fn test_proposals_filter_by_status_pending() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://{}/proposals?status=pending", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Vec<Value> = response.json().await.unwrap();
    assert_eq!(body.len(), 1, "Should return 1 pending proposal");
}

#[tokio::test]
async fn test_proposals_filter_by_status_approved() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://{}/proposals?status=approved", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Vec<Value> = response.json().await.unwrap();
    assert_eq!(body.len(), 0, "Should return empty array for approved status");
}

#[tokio::test]
async fn test_proposals_filter_by_invalid_status() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://{}/proposals?status=bogus", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Vec<Value> = response.json().await.unwrap();
    assert_eq!(body.len(), 0, "Should return empty array for invalid status (lenient)");
}

#[tokio::test]
async fn test_proposals_filter_by_category() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://{}/proposals?category=rust-learning", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Vec<Value> = response.json().await.unwrap();
    assert_eq!(body.len(), 1, "Should return 1 rust-learning proposal");
}

#[tokio::test]
async fn test_proposals_filter_by_unknown_category() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://{}/proposals?category=nonexistent", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Vec<Value> = response.json().await.unwrap();
    assert_eq!(body.len(), 0, "Should return empty array for unknown category");
}

#[tokio::test]
async fn test_proposal_by_id_returns_detail() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://{}/proposals/{}", addr, DEMO_UUID))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();

    assert!(body.get("id").is_some(), "Should have injected id field");
    assert_eq!(body["id"], DEMO_UUID, "id should match requested UUID");
    assert!(body.get("votes").is_some(), "Should have votes array");

    let votes = body["votes"].as_array().unwrap();
    assert_eq!(votes.len(), 1, "Should have 1 vote");
}

#[tokio::test]
async fn test_proposal_by_id_not_found() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!(
            "http://{}/proposals/00000000-0000-4000-8000-000000000000",
            addr
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 404);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["error"], "Proposal not found");
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
async fn test_mcp_list_proposals() {
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
                "name": "list_proposals",
                "arguments": {}
            }
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();

    assert_eq!(body["result"]["isError"], false);
    let text = body["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Proposals (1):"), "Should show 1 proposal in header");
}

#[tokio::test]
async fn test_mcp_list_proposals_filtered() {
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
                "name": "list_proposals",
                "arguments": {"status": "approved"}
            }
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();

    assert_eq!(body["result"]["isError"], false);
    let text = body["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Proposals (0):"), "Should show 0 proposals for approved filter");
}

#[tokio::test]
async fn test_mcp_get_proposal() {
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
                "name": "get_proposal",
                "arguments": {"id": DEMO_UUID}
            }
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();

    assert_eq!(body["result"]["isError"], false);
    let text = body["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Proposal:"), "Should contain Proposal header");
    assert!(text.contains("Votes"), "Should contain Votes section");
}

#[tokio::test]
async fn test_mcp_get_proposal_not_found() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    initialize_mcp(&client, &addr).await;

    let response = client
        .post(format!("http://{}/mcp", addr))
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": {
                "name": "get_proposal",
                "arguments": {"id": "00000000-0000-4000-8000-000000000000"}
            }
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();

    assert_eq!(body["result"]["isError"], true);
    let text = body["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("No proposal found"), "Should indicate not found");
}

#[tokio::test]
async fn test_mcp_tools_list_returns_nine() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    initialize_mcp(&client, &addr).await;

    let response = client
        .post(format!("http://{}/mcp", addr))
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "tools/list"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();

    let tools = body["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 9, "Should return 9 tools");

    let tool_names: Vec<&str> = tools
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert!(tool_names.contains(&"list_proposals"), "Should include list_proposals");
    assert!(tool_names.contains(&"get_proposal"), "Should include get_proposal");
    assert!(tool_names.contains(&"get_federated_sources"), "Should include get_federated_sources");
}
