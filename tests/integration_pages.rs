mod common;

use reqwest::Client;

/// Every static HTML page returns 200 with the shared header, footer, and stylesheet link
#[tokio::test]
async fn test_static_pages_have_header_and_footer() {
    let addr = common::spawn_test_server().await;
    let client = Client::new();

    for path in ["/", "/about"] {
        let response = client
            .get(format!("http://{}{}", addr, path))
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(response.status(), 200, "{} should return 200", path);
        let content_type = response.headers()["content-type"].to_str().unwrap().to_string();
        assert!(content_type.starts_with("text/html"), "{} content-type: {}", path, content_type);

        let body = response.text().await.unwrap();
        assert!(body.contains(r#"<header class="site-header">"#), "{} missing header", path);
        assert!(body.contains(r#"<footer class="site-footer">"#), "{} missing footer", path);
        assert!(body.contains(r#"href="/about""#), "{} missing About link", path);
        assert!(body.contains(r#"href="/style.css""#), "{} missing stylesheet link", path);
    }
}

/// The shared stylesheet is served as CSS
#[tokio::test]
async fn test_stylesheet_served() {
    let addr = common::spawn_test_server().await;
    let response = Client::new()
        .get(format!("http://{}/style.css", addr))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);
    let content_type = response.headers()["content-type"].to_str().unwrap().to_string();
    assert!(content_type.starts_with("text/css"), "content-type: {}", content_type);
    assert!(response.text().await.unwrap().contains("prefers-color-scheme"));
}
