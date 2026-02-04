//! Integration tests for query matching accuracy
//!
//! These tests validate query-to-category matching through real HTTP requests:
//! HTTP POST → MCP handler → query matching → category response
//!
//! Coverage:
//! - TEST-01: Expected category matches (5 tests)
//! - TEST-02: No-match scenarios (3 tests)
//! - Edge cases: high threshold, comprehensive category coverage (2 tests)

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

    response.json().await.unwrap()
}

/// Helper to call get_sources tool
async fn get_sources(client: &reqwest::Client, addr: &std::net::SocketAddr, query: &str) -> Value {
    let response = client
        .post(format!("http://{}/mcp", addr))
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "get_sources",
                "arguments": {"query": query}
            }
        }))
        .send()
        .await
        .unwrap();

    response.json().await.unwrap()
}

/// Helper to call get_sources with custom threshold
async fn get_sources_with_threshold(
    client: &reqwest::Client,
    addr: &std::net::SocketAddr,
    query: &str,
    threshold: f64,
) -> Value {
    let response = client
        .post(format!("http://{}/mcp", addr))
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "get_sources",
                "arguments": {
                    "query": query,
                    "threshold": threshold
                }
            }
        }))
        .send()
        .await
        .unwrap();

    response.json().await.unwrap()
}

// ===== TEST-01: Expected category matches =====

#[tokio::test]
async fn test_learn_rust_matches_rust_learning() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    initialize(&client, &addr).await;
    let response = get_sources(&client, &addr, "learn rust").await;

    assert_eq!(response["result"]["isError"], false);
    let text = response["result"]["content"][0]["text"].as_str().unwrap();

    assert!(text.contains("Rust Learning"), "Should match Rust Learning category");
    assert!(text.matches("http").count() >= 3, "Should contain at least 3 URLs");
}

#[tokio::test]
async fn test_bitcoin_node_matches_bitcoin_node_setup() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    initialize(&client, &addr).await;
    let response = get_sources(&client, &addr, "bitcoin node").await;

    assert_eq!(response["result"]["isError"], false);
    let text = response["result"]["content"][0]["text"].as_str().unwrap();

    assert!(text.contains("Bitcoin Node Setup"), "Should match Bitcoin Node Setup category");
}

#[tokio::test]
async fn test_email_server_matches_self_hosted_email() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    initialize(&client, &addr).await;
    let response = get_sources(&client, &addr, "email server").await;

    assert_eq!(response["result"]["isError"], false);
    let text = response["result"]["content"][0]["text"].as_str().unwrap();

    assert!(text.contains("Self-Hosted Email"), "Should match Self-Hosted Email category");
}

#[tokio::test]
async fn test_password_manager_matches_password_management() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    initialize(&client, &addr).await;
    let response = get_sources(&client, &addr, "password manager").await;

    assert_eq!(response["result"]["isError"], false);
    let text = response["result"]["content"][0]["text"].as_str().unwrap();

    assert!(
        text.contains("Password Management"),
        "Should match Password Management category"
    );
}

#[tokio::test]
async fn test_sources_contain_real_urls() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    initialize(&client, &addr).await;
    let response = get_sources(&client, &addr, "learn rust").await;

    assert_eq!(response["result"]["isError"], false);
    let text = response["result"]["content"][0]["text"].as_str().unwrap();

    // Extract all URLs
    let url_count = text.matches("http").count();
    assert!(
        url_count >= 3,
        "Should contain at least 3 distinct HTTP URLs, found {}",
        url_count
    );

    // Verify these are real URLs from registry
    assert!(text.contains("https://doc.rust-lang.org/book/"));
    assert!(text.contains("https://doc.rust-lang.org/rust-by-example/"));
    assert!(text.contains("https://www.zero2prod.com/"));
}

// ===== TEST-02: No-match scenarios =====

#[tokio::test]
async fn test_unrelated_query_returns_no_match() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    initialize(&client, &addr).await;
    let response = get_sources(&client, &addr, "quantum physics supercollider").await;

    assert_eq!(response["result"]["isError"], true);
    let text = response["result"]["content"][0]["text"].as_str().unwrap();

    assert!(
        text.contains("No matching category") || text.contains("Available categories"),
        "Should explain no match and show available categories"
    );
}

#[tokio::test]
async fn test_gibberish_query_returns_no_match() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    initialize(&client, &addr).await;
    let response = get_sources(&client, &addr, "xyzzy plugh foobar").await;

    assert_eq!(response["result"]["isError"], true);
}

#[tokio::test]
async fn test_empty_query_returns_error() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    initialize(&client, &addr).await;
    let response = get_sources(&client, &addr, "").await;

    assert_eq!(response["result"]["isError"], true);
    let text = response["result"]["content"][0]["text"].as_str().unwrap();

    assert!(
        text.contains("empty") || text.contains("cannot be empty"),
        "Should explain empty query error"
    );
}

// ===== Edge cases =====

#[tokio::test]
async fn test_high_threshold_rejects_partial_match() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    initialize(&client, &addr).await;

    // With threshold 0.99, even "learn rust" should fail to match
    let response = get_sources_with_threshold(&client, &addr, "learn rust", 0.99).await;

    assert_eq!(
        response["result"]["isError"], true,
        "High threshold should reject even good matches"
    );
}

#[tokio::test]
async fn test_each_seed_category_is_matchable() {
    let addr = common::spawn_test_server().await;
    let client = reqwest::Client::new();

    initialize(&client, &addr).await;

    // All 10 seed categories with their exact names
    let category_queries = vec![
        "Rust Learning",
        "Bitcoin Node Setup",
        "Self-Hosted Email",
        "Privacy-Respecting Home Automation",
        "Password Management",
        "Linux Security Hardening",
        "Threat Modeling",
        "Nostr Protocol Development",
        "Pubky Development",
        "MCP Development",
    ];

    let mut success_count = 0;

    for query in &category_queries {
        let response = get_sources(&client, &addr, query).await;
        if response["result"]["isError"] == false {
            success_count += 1;
        }
    }

    assert!(
        success_count >= 8,
        "At least 8 out of 10 seed categories should be matchable by name. Found: {}/10",
        success_count
    );
}
