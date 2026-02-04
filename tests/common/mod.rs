use three_good_sources::matcher::MatchConfig;
use three_good_sources::mcp::McpHandler;
use three_good_sources::pubky::identity::generate_or_load_keypair;
use three_good_sources::registry::Registry;
use three_good_sources::server::{AppState, build_router};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

/// Spawn a real HTTP server on a random port for integration testing.
/// Returns the socket address for making requests.
pub async fn spawn_test_server() -> SocketAddr {
    // Load real registry from project root (relative to tests/)
    let registry_json = include_str!("../../registry.json");
    let registry: Registry = serde_json::from_str(registry_json)
        .expect("Failed to parse registry.json");
    let registry = Arc::new(registry);

    // Default match config
    let match_config = MatchConfig {
        match_threshold: 0.4,
        match_fuzzy_weight: 0.7,
        match_keyword_weight: 0.3,
    };

    // Generate ephemeral keypair for testing
    let keypair = generate_or_load_keypair(None)
        .expect("Failed to generate test keypair");
    let pubkey = keypair.public_key();
    let pubkey_z32 = pubkey.to_z32();

    // Build MCP handler and app state
    let mcp_handler = McpHandler::new(Arc::clone(&registry), match_config, pubkey_z32);
    let app_state = Arc::new(AppState {
        mcp_handler,
        registry,
        pubkey,
    });

    let app = build_router(app_state);

    // Bind to port 0 - OS assigns random available port
    let listener = TcpListener::bind("127.0.0.1:0").await
        .expect("Failed to bind to port 0");
    let addr = listener.local_addr().unwrap();

    // Spawn server in background task
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Brief pause to ensure server is listening
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    addr
}
