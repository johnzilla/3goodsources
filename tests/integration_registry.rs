mod common;

use serde_json::Value;

#[tokio::test]
async fn test_registry_endpoint_returns_200() {
    let addr = common::spawn_test_server().await;
    let response = reqwest::get(format!("http://{}/registry", addr))
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_registry_has_expected_categories() {
    let addr = common::spawn_test_server().await;
    let response = reqwest::get(format!("http://{}/registry", addr))
        .await
        .unwrap();
    let registry: Value = response.json().await.unwrap();
    let categories = registry["categories"].as_object().unwrap();

    // Assert specific seed categories exist
    assert!(categories.contains_key("rust-learning"), "Missing rust-learning");
    assert!(categories.contains_key("bitcoin-node-setup"), "Missing bitcoin-node-setup");
    assert!(categories.contains_key("self-hosted-email"), "Missing self-hosted-email");
    assert!(categories.contains_key("home-automation-private"), "Missing home-automation-private");
    assert!(categories.contains_key("password-management"), "Missing password-management");
    assert!(categories.contains_key("linux-hardening"), "Missing linux-hardening");
    assert!(categories.contains_key("threat-modeling"), "Missing threat-modeling");
    assert!(categories.contains_key("nostr-development"), "Missing nostr-development");
    assert!(categories.contains_key("pubky-development"), "Missing pubky-development");
    assert!(categories.contains_key("mcp-development"), "Missing mcp-development");
    assert_eq!(categories.len(), 10, "Should have exactly 10 categories");
}

#[tokio::test]
async fn test_each_category_has_three_sources() {
    let addr = common::spawn_test_server().await;
    let response = reqwest::get(format!("http://{}/registry", addr))
        .await
        .unwrap();
    let registry: Value = response.json().await.unwrap();
    let categories = registry["categories"].as_object().unwrap();

    for (slug, category) in categories {
        let sources = category["sources"].as_array()
            .unwrap_or_else(|| panic!("Category {} missing sources array", slug));
        assert_eq!(sources.len(), 3, "Category {} should have exactly 3 sources", slug);
    }
}

#[tokio::test]
async fn test_sources_have_valid_structure() {
    let addr = common::spawn_test_server().await;
    let response = reqwest::get(format!("http://{}/registry", addr))
        .await
        .unwrap();
    let registry: Value = response.json().await.unwrap();
    let categories = registry["categories"].as_object().unwrap();

    for (slug, category) in categories {
        let sources = category["sources"].as_array().unwrap();
        for source in sources {
            assert!(source["rank"].is_number(), "{}: source missing rank", slug);
            assert!(source["name"].is_string(), "{}: source missing name", slug);
            assert!(source["url"].as_str().unwrap().starts_with("http"), "{}: source URL invalid", slug);
            assert!(source["type"].is_string(), "{}: source missing type", slug);
            assert!(source["why"].as_str().unwrap().len() > 10, "{}: source 'why' too short", slug);
        }
    }
}

#[tokio::test]
async fn test_sources_have_sequential_ranks() {
    let addr = common::spawn_test_server().await;
    let response = reqwest::get(format!("http://{}/registry", addr))
        .await
        .unwrap();
    let registry: Value = response.json().await.unwrap();
    let categories = registry["categories"].as_object().unwrap();

    for (slug, category) in categories {
        let sources = category["sources"].as_array().unwrap();
        let mut ranks: Vec<u64> = sources.iter()
            .map(|s| s["rank"].as_u64().unwrap())
            .collect();
        ranks.sort();
        assert_eq!(ranks, vec![1, 2, 3], "Category {} should have ranks 1,2,3", slug);
    }
}

#[tokio::test]
async fn test_registry_has_version_and_curator() {
    let addr = common::spawn_test_server().await;
    let response = reqwest::get(format!("http://{}/registry", addr))
        .await
        .unwrap();
    let registry: Value = response.json().await.unwrap();

    assert!(registry["version"].is_string(), "Missing version");
    assert!(registry["curator"]["name"].is_string(), "Missing curator name");
    assert!(registry["curator"]["pubkey"].is_string(), "Missing curator pubkey");
    assert!(registry["updated"].is_string(), "Missing updated date");
}

#[tokio::test]
async fn test_health_endpoint_returns_status_ok() {
    let addr = common::spawn_test_server().await;
    let response = reqwest::get(format!("http://{}/health", addr))
        .await
        .unwrap();
    assert_eq!(response.status(), 200);

    let body: Value = response.json().await.unwrap();
    assert_eq!(body["status"], "ok");
    assert!(body["version"].is_string(), "Missing version");
    assert!(body["pubkey"].is_string(), "Missing pubkey");
    // Pubkey should be z-base-32 (52 chars)
    let pubkey = body["pubkey"].as_str().unwrap();
    assert!(pubkey.len() > 40, "Pubkey too short: {}", pubkey);
}
