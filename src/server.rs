use crate::mcp::McpHandler;
use crate::registry::Registry;
use axum::{
    extract::State,
    http::{header, StatusCode},
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

/// Application state shared across all route handlers
pub struct AppState {
    pub mcp_handler: McpHandler,
    pub registry: Arc<Registry>,
}

/// Build the axum router with all routes and middleware
pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/mcp", post(mcp_endpoint))
        .route("/health", get(health_endpoint))
        .route("/registry", get(registry_endpoint))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// POST /mcp - MCP JSON-RPC endpoint
async fn mcp_endpoint(
    State(state): State<Arc<AppState>>,
    body: String,
) -> (StatusCode, [(axum::http::HeaderName, &'static str); 1], String) {
    match state.mcp_handler.handle_json(&body) {
        Some(json) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            json,
        ),
        None => (
            StatusCode::NO_CONTENT,
            [(header::CONTENT_TYPE, "application/json")],
            String::new(),
        ),
    }
}

/// GET /health - Health check endpoint
async fn health_endpoint() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// GET /registry - Registry JSON endpoint
async fn registry_endpoint(
    State(state): State<Arc<AppState>>,
) -> (StatusCode, [(axum::http::HeaderName, &'static str); 1], String) {
    match serde_json::to_string_pretty(&*state.registry) {
        Ok(json) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            json,
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(header::CONTENT_TYPE, "application/json")],
            format!(r#"{{"error":"Failed to serialize registry: {}"}}"#, e),
        ),
    }
}
