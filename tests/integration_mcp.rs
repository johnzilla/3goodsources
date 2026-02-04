//! Integration tests for MCP protocol JSON-RPC compliance
//!
//! These tests validate the complete request flow through real HTTP:
//! HTTP POST → JSON-RPC parsing → MCP handler → response
//!
//! Coverage:
//! - TEST-03: MCP protocol validation (initialize, tools/list, tools/call, error handling)
//! - JSON-RPC 2.0 compliance (parse errors, batch rejection, notifications)
//! - Pre-initialization gating
//! - All 4 MCP tools: get_sources, list_categories, get_provenance, get_endorsements

mod common;

use serde_json::Value;

/// Helper to initialize MCP handler
async fn initialize(client: &reqwest::Client, addr: &std::net::SocketAddr) -> Value {
    let response = client
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

    assert_eq!(response.status(), 200);
    response.json().await.unwrap()
}

#[tokio::test]
async fn test_initialize_returns_protocol_version() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    let response = initialize(&client, &addr).await;

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["result"]["protocolVersion"], "2025-11-25");
    assert!(response["result"]["capabilities"]["tools"].is_object());
    assert_eq!(response["result"]["serverInfo"]["name"], "three-good-sources");
    assert!(response["result"]["serverInfo"]["version"].is_string());
}

#[tokio::test]
async fn test_tools_list_returns_four_tools() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    // Initialize first
    initialize(&client, &addr).await;

    // Now request tools/list
    let response = client
        .post(format!("http://{}/mcp", addr))
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();

    assert_eq!(body["jsonrpc"], "2.0");
    assert!(body["result"]["tools"].is_array());

    let tools = body["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 4, "Should return exactly 4 tools");

    // Verify tool names
    let tool_names: Vec<&str> = tools
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();

    assert!(tool_names.contains(&"get_sources"));
    assert!(tool_names.contains(&"list_categories"));
    assert!(tool_names.contains(&"get_provenance"));
    assert!(tool_names.contains(&"get_endorsements"));

    // Verify schema structure
    for tool in tools {
        assert!(tool["name"].is_string());
        assert!(tool["description"].is_string());
        assert!(tool["inputSchema"].is_object());
        assert_eq!(tool["inputSchema"]["type"], "object");
    }
}

#[tokio::test]
async fn test_tools_call_get_sources() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    initialize(&client, &addr).await;

    let response = client
        .post(format!("http://{}/mcp", addr))
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "get_sources",
                "arguments": {"query": "learn rust"}
            }
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();

    assert_eq!(body["jsonrpc"], "2.0");
    assert!(body["result"]["content"].is_array());
    assert_eq!(body["result"]["isError"], false);

    let content = &body["result"]["content"][0];
    assert_eq!(content["type"], "text");

    let text = content["text"].as_str().unwrap();
    assert!(text.contains("Rust Learning"), "Should contain category name");
    assert!(text.matches("http").count() >= 3, "Should contain at least 3 URLs");
}

#[tokio::test]
async fn test_tools_call_list_categories() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    initialize(&client, &addr).await;

    let response = client
        .post(format!("http://{}/mcp", addr))
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "list_categories",
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

    assert!(text.contains("rust-learning"));
    assert!(text.contains("bitcoin-node-setup"));
}

#[tokio::test]
async fn test_tools_call_get_provenance() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    initialize(&client, &addr).await;

    let response = client
        .post(format!("http://{}/mcp", addr))
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": {
                "name": "get_provenance",
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

    assert!(text.contains("Curator:"));
    assert!(text.contains("3GS Curator"));
}

#[tokio::test]
async fn test_tools_call_get_endorsements() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    initialize(&client, &addr).await;

    let response = client
        .post(format!("http://{}/mcp", addr))
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "tools/call",
            "params": {
                "name": "get_endorsements",
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

    assert!(
        text.contains("Endorsements: 0") || text.contains("no endorsements"),
        "Should indicate no endorsements"
    );
}

#[tokio::test]
async fn test_malformed_json_returns_parse_error() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    let response = client
        .post(format!("http://{}/mcp", addr))
        .header("content-type", "application/json")
        .body("{ invalid json }")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();

    assert_eq!(body["jsonrpc"], "2.0");
    assert_eq!(body["error"]["code"], -32700, "Should be Parse error");
}

#[tokio::test]
async fn test_batch_request_rejected() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    let response = client
        .post(format!("http://{}/mcp", addr))
        .json(&serde_json::json!([
            {"jsonrpc": "2.0", "id": 1, "method": "initialize"},
            {"jsonrpc": "2.0", "id": 2, "method": "tools/list"}
        ]))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();

    assert_eq!(body["jsonrpc"], "2.0");
    assert_eq!(body["error"]["code"], -32600, "Should be Invalid request");
    assert!(body["error"]["message"].as_str().unwrap().contains("Batch"));
}

#[tokio::test]
async fn test_pre_init_tools_list_rejected() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    // Don't initialize - directly call tools/list
    let response = client
        .post(format!("http://{}/mcp", addr))
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();

    assert_eq!(body["error"]["code"], -32002, "Should be Not initialized");
}

#[tokio::test]
async fn test_unknown_method_returns_error() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    initialize(&client, &addr).await;

    let response = client
        .post(format!("http://{}/mcp", addr))
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "unknown/method"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();

    assert_eq!(body["error"]["code"], -32601, "Should be Method not found");
}

#[tokio::test]
async fn test_notification_returns_204() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    // Send notification (no id field)
    let response = client
        .post(format!("http://{}/mcp", addr))
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 204, "Notifications should return 204 No Content");

    // Body should be empty
    let body = response.bytes().await.unwrap();
    assert!(body.is_empty(), "Notification response should have empty body");
}

#[tokio::test]
async fn test_all_responses_have_jsonrpc_field() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    // Test parse error
    let response = client
        .post(format!("http://{}/mcp", addr))
        .header("content-type", "application/json")
        .body("{ invalid }")
        .send()
        .await
        .unwrap();
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["jsonrpc"], "2.0");

    // Test batch rejection
    let response = client
        .post(format!("http://{}/mcp", addr))
        .json(&serde_json::json!([]))
        .send()
        .await
        .unwrap();
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["jsonrpc"], "2.0");

    // Test pre-init rejection
    let response = client
        .post(format!("http://{}/mcp", addr))
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list"
        }))
        .send()
        .await
        .unwrap();
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["jsonrpc"], "2.0");

    // Test successful initialize
    let body = initialize(&client, &addr).await;
    assert_eq!(body["jsonrpc"], "2.0");
}
