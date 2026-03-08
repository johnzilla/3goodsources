use crate::audit::{AuditEntry, AuditFilterParams, filter_entries};
use crate::contributions::{Proposal, ProposalFilterParams, ProposalSummary};
use crate::identity::Identity;
use crate::mcp::McpHandler;
use crate::registry::Registry;
use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderName, HeaderValue, Method, StatusCode},
    routing::{get, post},
    Json, Router,
};
use pkarr::PublicKey;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

const LANDING_HTML: &str = include_str!("../docs/index.html");

/// Application state shared across all route handlers
pub struct AppState {
    pub mcp_handler: McpHandler,
    pub registry: Arc<Registry>,
    pub pubkey: PublicKey,  // PublicKey is Copy, no Arc needed
    pub audit_log: Arc<Vec<AuditEntry>>,
    pub identities: Arc<HashMap<String, Identity>>,
    pub proposals: Arc<HashMap<Uuid, Proposal>>,
}

/// Build the axum router with all routes and middleware
pub fn build_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin([
            "https://3gs.ai".parse::<HeaderValue>().unwrap(),
            "https://api.3gs.ai".parse::<HeaderValue>().unwrap(),
        ])
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
        .expose_headers([
            HeaderName::from_static("mcp-session-id"),
            HeaderName::from_static("x-request-id"),
        ])
        .max_age(Duration::from_secs(3600));

    Router::new()
        .route("/", get(landing_page_endpoint))
        .route("/mcp", post(mcp_endpoint))
        .route("/health", get(health_endpoint))
        .route("/registry", get(registry_endpoint))
        .route("/audit", get(audit_endpoint))
        .route("/identities", get(identities_endpoint))
        .route("/identities/{pubkey}", get(identity_by_pubkey_endpoint))
        .route("/proposals", get(proposals_endpoint))
        .route("/proposals/{id}", get(proposal_by_id_endpoint))
        .layer(cors)
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
async fn health_endpoint(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "pubkey": state.pubkey.to_z32()
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

/// GET /audit - Audit log endpoint with optional query filters
async fn audit_endpoint(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AuditFilterParams>,
) -> (StatusCode, [(axum::http::HeaderName, &'static str); 1], String) {
    let filtered = filter_entries(&state.audit_log, &params);
    let entries: Vec<&AuditEntry> = filtered;

    match serde_json::to_string(&entries) {
        Ok(json) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            json,
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(header::CONTENT_TYPE, "application/json")],
            format!(r#"{{"error":"Failed to serialize audit log: {}"}}"#, e),
        ),
    }
}

/// GET /identities - Returns all identities as JSON object keyed by pubkey
async fn identities_endpoint(
    State(state): State<Arc<AppState>>,
) -> (StatusCode, [(axum::http::HeaderName, &'static str); 1], String) {
    match serde_json::to_string_pretty(&*state.identities) {
        Ok(json) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            json,
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(header::CONTENT_TYPE, "application/json")],
            format!(r#"{{"error":"Failed to serialize identities: {}"}}"#, e),
        ),
    }
}

/// GET /identities/:pubkey - Returns a single identity by pubkey or 404
async fn identity_by_pubkey_endpoint(
    State(state): State<Arc<AppState>>,
    Path(pubkey): Path<String>,
) -> (StatusCode, [(axum::http::HeaderName, &'static str); 1], String) {
    match state.identities.get(&pubkey) {
        Some(identity) => match serde_json::to_string_pretty(identity) {
            Ok(json) => (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/json")],
                json,
            ),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(header::CONTENT_TYPE, "application/json")],
                format!(r#"{{"error":"Failed to serialize identity: {}"}}"#, e),
            ),
        },
        None => (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "application/json")],
            r#"{"error":"Identity not found"}"#.to_string(),
        ),
    }
}

/// GET /proposals - List proposals with optional status and category filters
async fn proposals_endpoint(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ProposalFilterParams>,
) -> (StatusCode, [(axum::http::HeaderName, &'static str); 1], String) {
    let mut summaries: Vec<ProposalSummary> = state
        .proposals
        .iter()
        .filter(|(_, proposal)| {
            // Filter by status (lenient: serialize status to string and compare)
            if let Some(ref status_filter) = params.status {
                let status_str = serde_json::to_value(&proposal.status)
                    .ok()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .unwrap_or_default();
                if status_str != *status_filter {
                    return false;
                }
            }
            // Filter by category
            if let Some(ref cat_filter) = params.category {
                if proposal.category != *cat_filter {
                    return false;
                }
            }
            true
        })
        .map(|(id, proposal)| ProposalSummary {
            id: *id,
            action: proposal.action.clone(),
            status: proposal.status.clone(),
            category: proposal.category.clone(),
            proposer: proposal.proposer.clone(),
            created_at: proposal.created_at,
        })
        .collect();

    // Sort by created_at descending
    summaries.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    match serde_json::to_string(&summaries) {
        Ok(json) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            json,
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(header::CONTENT_TYPE, "application/json")],
            format!(r#"{{"error":"Failed to serialize proposals: {}"}}"#, e),
        ),
    }
}

/// GET /proposals/{id} - Get full proposal detail by UUID
async fn proposal_by_id_endpoint(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> (StatusCode, [(axum::http::HeaderName, &'static str); 1], String) {
    match state.proposals.get(&id) {
        Some(proposal) => {
            let mut value = serde_json::to_value(proposal).unwrap_or_default();
            if let Some(obj) = value.as_object_mut() {
                obj.insert("id".to_string(), serde_json::Value::String(id.to_string()));
            }
            match serde_json::to_string_pretty(&value) {
                Ok(json) => (
                    StatusCode::OK,
                    [(header::CONTENT_TYPE, "application/json")],
                    json,
                ),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    [(header::CONTENT_TYPE, "application/json")],
                    format!(r#"{{"error":"Failed to serialize proposal: {}"}}"#, e),
                ),
            }
        }
        None => (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "application/json")],
            r#"{"error":"Proposal not found"}"#.to_string(),
        ),
    }
}

/// GET / - Landing page endpoint
async fn landing_page_endpoint() -> (StatusCode, [(axum::http::HeaderName, &'static str); 1], &'static str) {
    (StatusCode::OK, [(header::CONTENT_TYPE, "text/html; charset=utf-8")], LANDING_HTML)
}
