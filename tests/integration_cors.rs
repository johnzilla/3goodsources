mod common;

use reqwest::Client;

/// Test CORS preflight (OPTIONS) for allowed origin https://3gs.ai
#[tokio::test]
async fn test_cors_preflight_allowed_origin() {
    let addr = common::spawn_test_server().await;
    let url = format!("http://{}/mcp", addr);
    let client = Client::new();

    let response = client
        .request(reqwest::Method::OPTIONS, &url)
        .header("Origin", "https://3gs.ai")
        .header("Access-Control-Request-Method", "POST")
        .header("Access-Control-Request-Headers", "content-type")
        .send()
        .await
        .expect("Failed to send OPTIONS request");

    assert_eq!(response.status(), 200, "Preflight should return 200");

    let allow_origin = response
        .headers()
        .get("access-control-allow-origin")
        .expect("Missing access-control-allow-origin header")
        .to_str()
        .unwrap();
    assert_eq!(allow_origin, "https://3gs.ai");

    let allow_methods = response
        .headers()
        .get("access-control-allow-methods")
        .expect("Missing access-control-allow-methods header")
        .to_str()
        .unwrap();
    assert!(
        allow_methods.contains("POST"),
        "Should allow POST method, got: {}",
        allow_methods
    );

    let allow_headers = response
        .headers()
        .get("access-control-allow-headers")
        .expect("Missing access-control-allow-headers header")
        .to_str()
        .unwrap()
        .to_lowercase();
    assert!(
        allow_headers.contains("content-type"),
        "Should allow content-type header, got: {}",
        allow_headers
    );

    let max_age = response
        .headers()
        .get("access-control-max-age")
        .expect("Missing access-control-max-age header")
        .to_str()
        .unwrap();
    assert_eq!(max_age, "3600");
}

/// Test CORS preflight (OPTIONS) for allowed origin https://api.3gs.ai
#[tokio::test]
async fn test_cors_preflight_api_origin() {
    let addr = common::spawn_test_server().await;
    let url = format!("http://{}/mcp", addr);
    let client = Client::new();

    let response = client
        .request(reqwest::Method::OPTIONS, &url)
        .header("Origin", "https://api.3gs.ai")
        .header("Access-Control-Request-Method", "POST")
        .send()
        .await
        .expect("Failed to send OPTIONS request");

    assert_eq!(response.status(), 200);

    let allow_origin = response
        .headers()
        .get("access-control-allow-origin")
        .expect("Missing access-control-allow-origin header")
        .to_str()
        .unwrap();
    assert_eq!(allow_origin, "https://api.3gs.ai");
}

/// Test actual POST request with allowed origin receives CORS headers
#[tokio::test]
async fn test_cors_actual_post_allowed_origin() {
    let addr = common::spawn_test_server().await;
    let url = format!("http://{}/mcp", addr);
    let client = Client::new();

    let initialize_request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        },
        "id": 1
    });

    let response = client
        .post(&url)
        .header("Origin", "https://3gs.ai")
        .header("Content-Type", "application/json")
        .json(&initialize_request)
        .send()
        .await
        .expect("Failed to send POST request");

    assert_eq!(response.status(), 200);

    let allow_origin = response
        .headers()
        .get("access-control-allow-origin")
        .expect("Missing access-control-allow-origin header")
        .to_str()
        .unwrap();
    assert_eq!(allow_origin, "https://3gs.ai");

    // Verify response is valid JSON-RPC
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert!(body.get("jsonrpc").is_some(), "Response should have jsonrpc field");
}

/// Test that unlisted origins do NOT receive access-control-allow-origin header
#[tokio::test]
async fn test_cors_rejects_unlisted_origin() {
    let addr = common::spawn_test_server().await;
    let url = format!("http://{}/mcp", addr);
    let client = Client::new();

    let initialize_request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        },
        "id": 1
    });

    let response = client
        .post(&url)
        .header("Origin", "https://evil.com")
        .header("Content-Type", "application/json")
        .json(&initialize_request)
        .send()
        .await
        .expect("Failed to send POST request");

    // Request itself still succeeds (CORS is browser-enforced)
    assert_eq!(response.status(), 200);

    // But no CORS header should be present
    assert!(
        response.headers().get("access-control-allow-origin").is_none(),
        "Unlisted origin should not receive access-control-allow-origin header"
    );
}

/// Test that custom headers are exposed via access-control-expose-headers
#[tokio::test]
async fn test_cors_exposes_custom_headers() {
    let addr = common::spawn_test_server().await;
    let url = format!("http://{}/mcp", addr);
    let client = Client::new();

    // Use actual POST request (not preflight) to check expose-headers
    let initialize_request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        },
        "id": 1
    });

    let response = client
        .post(&url)
        .header("Origin", "https://3gs.ai")
        .header("Content-Type", "application/json")
        .json(&initialize_request)
        .send()
        .await
        .expect("Failed to send POST request");

    assert_eq!(response.status(), 200);

    let expose_headers = response
        .headers()
        .get("access-control-expose-headers")
        .expect("Missing access-control-expose-headers header")
        .to_str()
        .unwrap()
        .to_lowercase();

    assert!(
        expose_headers.contains("mcp-session-id"),
        "Should expose mcp-session-id header, got: {}",
        expose_headers
    );
    assert!(
        expose_headers.contains("x-request-id"),
        "Should expose x-request-id header, got: {}",
        expose_headers
    );
}

/// Test that CORS applies to all routes (using /health as example)
#[tokio::test]
async fn test_cors_health_endpoint() {
    let addr = common::spawn_test_server().await;
    let url = format!("http://{}/health", addr);
    let client = Client::new();

    let response = client
        .get(&url)
        .header("Origin", "https://3gs.ai")
        .send()
        .await
        .expect("Failed to send GET request");

    assert_eq!(response.status(), 200);

    let allow_origin = response
        .headers()
        .get("access-control-allow-origin")
        .expect("Missing access-control-allow-origin header on /health")
        .to_str()
        .unwrap();
    assert_eq!(allow_origin, "https://3gs.ai");
}
