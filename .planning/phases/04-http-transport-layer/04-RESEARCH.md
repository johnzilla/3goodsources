# Phase 4: HTTP Transport Layer - Research

**Researched:** 2026-02-02
**Domain:** Rust web servers with axum 0.8, tower-http middleware, JSON-RPC over HTTP
**Confidence:** HIGH

## Summary

Phase 4 implements an HTTP server to expose the MCP protocol over POST /mcp with supporting endpoints for health checks and registry transparency. The standard approach in Rust is axum 0.8 + tokio + tower-http middleware, which was already decided in Phase 1.

Research confirms axum 0.8 was released January 2025 with breaking changes to path syntax and extractors. The framework follows a macro-free design using Tower's middleware ecosystem, particularly tower-http for CORS, compression, and tracing. State management uses Arc<T> for shared handler state, and custom error types implement IntoResponse for clean error handling.

The existing McpHandler::handle_json(&str) -> Option<String> pattern integrates cleanly with axum handlers by extracting the raw request body as String and returning the serialized response. CORS uses CorsLayer::permissive() from tower-http 0.6.6, and health endpoints follow simple async handler patterns.

**Primary recommendation:** Use axum 0.8 with tokio::net::TcpListener, tower-http CorsLayer, Arc-wrapped McpHandler state, and String extractor for raw JSON-RPC body handling.

## Standard Stack

The established libraries/tools for Rust HTTP servers in 2026:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| axum | 0.8.x | HTTP routing & handlers | Official Tokio project, macro-free API, ergonomic extractors, production-ready |
| tokio | 1.49+ | Async runtime | Already in project, industry standard for async Rust, full feature set available |
| tower-http | 0.6.6 | HTTP middleware (CORS, tracing, compression) | Tower ecosystem integration, battle-tested middleware, axum's recommended approach |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tower | 0.5+ | Service trait & middleware composition | Implicit dependency via axum, may need explicit for custom middleware |
| hyper | 1.x | HTTP protocol implementation | Implicit via axum, no direct dependency needed |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| axum | actix-web | Actix has more features but uses actor model; axum is simpler, more ergonomic for this use case |
| axum | rocket | Rocket uses macros heavily; axum's macro-free API gives better type errors and IDE support |
| tower-http CORS | custom middleware | Custom CORS is error-prone (preflight handling, header combinations); tower-http is spec-compliant |

**Installation:**
```toml
[dependencies]
axum = "0.8"
tower-http = { version = "0.6", features = ["cors"] }
# tokio already present with "full" features
```

## Architecture Patterns

### Recommended Project Structure
```
src/
├── main.rs           # Server setup, listener binding, route configuration
├── routes/           # Route handlers (mcp_handler, health_handler, registry_handler)
└── [existing modules]
```

For this phase, integrating routes directly in main.rs or a new routes module is acceptable given the small number of endpoints (3 total).

### Pattern 1: State Management with Arc
**What:** Share application state (McpHandler, Registry, Config) across handlers using Arc wrappers and axum's State extractor
**When to use:** When multiple handlers need read access to shared data (our use case)
**Example:**
```rust
// Source: https://docs.rs/axum/latest/axum/
use axum::{Router, extract::State, routing::post};
use std::sync::Arc;

struct AppState {
    mcp_handler: Arc<McpHandler>,
    config: Arc<Config>,
}

async fn mcp_endpoint(
    State(state): State<Arc<AppState>>,
    body: String,
) -> impl IntoResponse {
    // Use state.mcp_handler
}

let app_state = Arc::new(AppState { ... });
let app = Router::new()
    .route("/mcp", post(mcp_endpoint))
    .with_state(app_state);
```

### Pattern 2: Raw Body Extraction with String
**What:** Extract entire request body as String for JSON-RPC handling
**When to use:** When you need the raw request body to pass to an existing JSON handler
**Example:**
```rust
// Source: https://docs.rs/axum/latest/axum/extract/index.html
async fn mcp_handler(body: String) -> impl IntoResponse {
    // body is the entire request as UTF-8 string
    let response_json = handler.handle_json(&body);
    // Return response
}
```
**Note:** String extractor must be the last parameter (consumes body stream)

### Pattern 3: Custom Error Type with IntoResponse
**What:** Implement IntoResponse on custom error enum to convert errors to HTTP responses
**When to use:** Centralized error handling with consistent status codes (our use case)
**Example:**
```rust
// Source: https://github.com/tokio-rs/axum/blob/main/examples/error-handling/src/main.rs
use axum::{response::{IntoResponse, Response}, http::StatusCode};

enum AppError {
    InvalidJson,
    InternalError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::InvalidJson => (StatusCode::BAD_REQUEST, "Invalid JSON"),
            AppError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error"),
        };
        (status, message).into_response()
    }
}
```

### Pattern 4: CORS with CorsLayer
**What:** Apply permissive CORS middleware using tower-http CorsLayer
**When to use:** When API needs cross-origin access (required for browser-based MCP clients)
**Example:**
```rust
// Source: https://docs.rs/tower-http/latest/tower_http/cors/index.html
use tower_http::cors::CorsLayer;
use axum::Router;

let app = Router::new()
    .route("/mcp", post(handler))
    .layer(CorsLayer::permissive());
```
**Note:** `CorsLayer::permissive()` is the simplest approach for MVP; production may want to restrict origins

### Pattern 5: Server Startup with TcpListener
**What:** Bind tokio::net::TcpListener and pass to axum::serve
**When to use:** Always (standard pattern in axum 0.8)
**Example:**
```rust
// Source: https://docs.rs/axum/latest/axum/fn.serve.html
use tokio::net::TcpListener;

let listener = TcpListener::bind("0.0.0.0:3000").await?;
axum::serve(listener, app).await?;
```

### Pattern 6: Environment-based Port Configuration
**What:** Read PORT from environment with fallback default
**When to use:** Deployment to platforms like Render that set PORT dynamically
**Example:**
```rust
// Source: https://oneuptime.com/blog/post/2026-01-07-rust-axum-rest-api/view
let port = std::env::var("PORT")
    .ok()
    .and_then(|p| p.parse().ok())
    .unwrap_or(3000);
let addr = format!("0.0.0.0:{}", port);
let listener = TcpListener::bind(&addr).await?;
```

### Pattern 7: Returning Raw JSON Strings
**What:** Return pre-serialized JSON string from handler with application/json content-type
**When to use:** When handler returns JSON string (our McpHandler case)
**Example:**
```rust
// Source: https://docs.rs/axum/latest/axum/response/index.html
use axum::{response::IntoResponse, http::{StatusCode, header}};

async fn handler(body: String) -> impl IntoResponse {
    let json_response = some_handler(&body);
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        json_response
    )
}
```

### Pattern 8: Health Check Endpoint
**What:** Simple GET handler returning JSON with version info
**When to use:** Production deployments needing liveness/readiness checks
**Example:**
```rust
// Source: https://oneuptime.com/blog/post/2026-01-07-rust-axum-rest-api/view
use axum::Json;
use serde_json::json;

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
```

### Anti-Patterns to Avoid
- **Using Json<T> extractor for raw JSON-RPC:** Json<T> deserializes into Rust types, but McpHandler needs raw string for custom parsing
- **Forgetting String extractor position:** String must be last parameter since it consumes request body
- **Custom CORS implementation:** tower-http handles preflight, Vary headers, and edge cases correctly
- **Blocking operations in handlers:** All I/O must be async; use tokio::spawn_blocking for CPU-heavy work
- **Missing content-type on JSON responses:** Always set application/json header when returning JSON strings

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| CORS headers | Manual header setting | tower-http CorsLayer | Handles preflight OPTIONS, Vary header, origin validation, spec compliance |
| Graceful shutdown | Manual signal handling | axum::serve().with_graceful_shutdown() | Proper connection draining, task cancellation, timeout handling |
| Request logging | println! or custom logs | tower-http::trace::TraceLayer | Structured logs, request IDs, latency tracking, error context |
| JSON response serialization | String concatenation | serde_json::to_string() | Handles escaping, UTF-8, nested structures correctly |
| Port binding retry | Loop with sleep | TcpListener::bind() once | Let process supervisor handle restarts; failing fast is better than hiding issues |

**Key insight:** HTTP middleware is complex (CORS preflight, header ordering, caching). Tower ecosystem has production-tested solutions with edge case handling that would take weeks to replicate correctly.

## Common Pitfalls

### Pitfall 1: Axum 0.8 Path Syntax Change
**What goes wrong:** Routes defined with old syntax `/:param` fail to compile or behave incorrectly
**Why it happens:** Breaking change in axum 0.8 from `:param` to `{param}` syntax
**How to avoid:** Use new syntax `/{param}` and `/{*catchall}` for paths; escape literal braces with `{{` or `}}`
**Warning signs:** Compilation errors about route syntax; routes not matching expected paths

### Pitfall 2: Extractor Order Violations
**What goes wrong:** Compilation errors like "body extractor must be last"
**Why it happens:** Body-consuming extractors (String, Bytes, Json<T>) can only run once per request
**How to avoid:** Always put body extractors (String) as the last handler parameter; State<T> comes before body
**Warning signs:** Confusing compiler errors about extractor traits; IntoResponse not implemented

### Pitfall 3: Missing Content-Type on Raw JSON
**What goes wrong:** Client receives JSON but without application/json header, causing parsing failures
**Why it happens:** Returning String doesn't automatically set content-type (unlike Json<T>)
**How to avoid:** Always return tuple with header: `(StatusCode::OK, [(header::CONTENT_TYPE, "application/json")], body)`
**Warning signs:** Curl shows correct data but missing content-type; browser/client fails to parse

### Pitfall 4: Blocking McpHandler Calls
**What goes wrong:** Server becomes unresponsive under load; timeouts on concurrent requests
**Why it happens:** McpHandler::handle_json is synchronous but called from async context, blocks tokio thread
**How to avoid:** Ensure McpHandler operations are non-blocking; if heavy CPU work, use tokio::spawn_blocking
**Warning signs:** Single request works fine, multiple concurrent requests cause delays; high CPU on tokio worker threads

### Pitfall 5: Option<String> None Response Handling
**What goes wrong:** Server returns empty response or hangs when McpHandler returns None (notifications)
**Why it happens:** None from handle_json means "no response" but handler must still return HTTP 200 or error
**How to avoid:** Map None to HTTP 204 No Content or 200 with empty JSON object, depending on spec requirements
**Warning signs:** Client timeouts on notification requests; server logs show handler completed but no response sent

### Pitfall 6: CORS Preflight Not Handled
**What goes wrong:** Browser sends OPTIONS request, gets 404 or 405, blocks actual POST
**Why it happens:** CORS preflight OPTIONS isn't automatically handled without middleware
**How to avoid:** Use CorsLayer which automatically responds to OPTIONS with correct headers
**Warning signs:** POST works in curl but fails in browser; OPTIONS requests show 404/405 errors

### Pitfall 7: Port Already in Use
**What goes wrong:** Server crashes on startup with "address already in use"
**Why it happens:** Previous process still bound to port; testing without cleanup
**How to avoid:** Use SO_REUSEADDR (automatic with tokio), or let TcpListener error and surface to user
**Warning signs:** Server works once then fails on restart; port conflict errors in logs

### Pitfall 8: Malformed JSON Panics
**What goes wrong:** Server crashes when receiving invalid UTF-8 or extremely large bodies
**Why it happens:** String extractor expects valid UTF-8; no size limits by default
**How to avoid:** String extractor rejects invalid UTF-8 automatically; add tower-http::limit::RequestBodyLimitLayer for size protection
**Warning signs:** Crashes on fuzzing; memory usage spikes on large requests

## Code Examples

Verified patterns from official sources:

### Complete Main Function with Server Setup
```rust
// Source: https://docs.rs/axum/latest/axum/ and https://tokio.rs/blog/2025-01-01-announcing-axum-0-8-0
use axum::{Router, routing::{get, post}, extract::State};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging (already in project)
    tracing_subscriber::fmt::init();

    // Load config and create state
    let config = Config::load()?;
    let registry = registry::load(&config.registry_path).await?;
    let match_config = MatchConfig::load()?;
    let mcp_handler = McpHandler::new(Arc::clone(&registry), match_config);

    let app_state = Arc::new(AppState {
        mcp_handler,
        registry,
        config,
    });

    // Build router with routes
    let app = Router::new()
        .route("/mcp", post(mcp_endpoint))
        .route("/health", get(health_endpoint))
        .route("/registry", get(registry_endpoint))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    // Bind and serve
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    let addr = format!("0.0.0.0:{}", port);

    tracing::info!("Server starting on {}", addr);
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

### MCP Endpoint Handler
```rust
// Source: https://docs.rs/axum/latest/axum/extract/index.html
use axum::{
    extract::State,
    response::IntoResponse,
    http::{StatusCode, header},
};
use std::sync::Arc;

struct AppState {
    mcp_handler: McpHandler,
}

async fn mcp_endpoint(
    State(state): State<Arc<AppState>>,
    body: String,
) -> impl IntoResponse {
    match state.mcp_handler.handle_json(&body) {
        Some(json_response) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            json_response
        ),
        None => (
            StatusCode::NO_CONTENT,
            [(header::CONTENT_TYPE, "application/json")],
            String::new()
        ),
    }
}
```

### Health Check Endpoint
```rust
// Source: https://oneuptime.com/blog/post/2026-01-07-rust-axum-rest-api/view
use axum::Json;
use serde_json::{json, Value};

async fn health_endpoint() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
```

### Registry Transparency Endpoint
```rust
// Source: https://docs.rs/axum/latest/axum/response/index.html
use axum::{
    extract::State,
    response::IntoResponse,
    http::{StatusCode, header},
};
use std::sync::Arc;

async fn registry_endpoint(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Return raw registry JSON for transparency
    let json_str = serde_json::to_string_pretty(&*state.registry)
        .unwrap_or_else(|_| "{}".to_string());

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        json_str
    )
}
```

### CORS Configuration
```rust
// Source: https://docs.rs/tower-http/latest/tower_http/cors/index.html
use tower_http::cors::CorsLayer;

// Permissive (MVP - allow all origins)
let cors = CorsLayer::permissive();

// OR very permissive (mirrors request headers/methods)
let cors = CorsLayer::very_permissive();

// Applied as layer
let app = Router::new()
    .route("/mcp", post(handler))
    .layer(cors);
```

### Port Configuration from Environment
```rust
// Source: https://oneuptime.com/blog/post/2026-01-07-rust-axum-rest-api/view
let port: u16 = std::env::var("PORT")
    .ok()
    .and_then(|p| p.parse().ok())
    .unwrap_or(3000);

let addr = format!("0.0.0.0:{}", port);
let listener = TcpListener::bind(&addr).await?;
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| hyper Server directly | axum::serve(listener, router) | axum 0.6+ | Simpler API, no manual service setup |
| /:param route syntax | /{param} route syntax | axum 0.8 (Jan 2025) | Must update all route definitions |
| #[async_trait] macro | Native async fn in traits | Rust 1.75 / axum 0.8 | Remove macro, cleaner code |
| Option<T> auto-unwraps | OptionalFromRequestParts trait | axum 0.8 | More explicit optional extraction |
| tower-http 0.5.x | tower-http 0.6.x | 2025 | Feature flag changes, minor API updates |

**Deprecated/outdated:**
- `hyper::Server::bind()`: Use `axum::serve()` instead
- Old route syntax `/:param`: Use `/{param}` in axum 0.8+
- `#[async_trait]` on extractors: Native async works in axum 0.8
- Manually handling OPTIONS for CORS: Use CorsLayer which auto-handles preflight

## Open Questions

Things that couldn't be fully resolved:

1. **McpHandler thread safety**
   - What we know: McpHandler is currently not Sync (uses MatchConfig value, not Arc)
   - What's unclear: Whether McpHandler needs interior mutability or can be immutable
   - Recommendation: Review if McpHandler should hold MatchConfig by value or Arc; if immutable, no Send/Sync issues

2. **Response streaming for large registries**
   - What we know: Registry JSON can be returned as String, works for small registries
   - What's unclear: Whether streaming is needed for registries > 1MB (current limit is 10MB per STATE.md)
   - Recommendation: Start with String response, add streaming if registry grows beyond 1MB

3. **Health check pubkey field**
   - What we know: ENDP-02 mentions "without pubkey for now"
   - What's unclear: Whether health endpoint should have placeholder null or omit field entirely
   - Recommendation: Omit pubkey field in Phase 4, add in Phase 5 when PKARR integration happens

## Sources

### Primary (HIGH confidence)
- [Axum 0.8.0 Official Announcement](https://tokio.rs/blog/2025-01-01-announcing-axum-0-8-0) - Breaking changes and migration guide
- [Axum 0.8.8 Documentation](https://docs.rs/axum/latest/axum/) - Official API docs, extractors, response types
- [Tower-HTTP Documentation](https://docs.rs/tower-http/latest/tower_http/) - CORS middleware configuration
- [Tower-HTTP CORS Module](https://docs.rs/tower-http/latest/tower_http/cors/index.html) - CorsLayer API and examples
- [Axum Error Handling Example](https://github.com/tokio-rs/axum/blob/main/examples/error-handling/src/main.rs) - Official IntoResponse pattern

### Secondary (MEDIUM confidence)
- [How to Build Production-Ready REST APIs in Rust with Axum](https://oneuptime.com/blog/post/2026-01-07-rust-axum-rest-api/view) - January 2026 production patterns
- [The Ultimate Guide to Axum (2025)](https://www.shuttle.dev/blog/2023/12/06/using-axum-rust) - Updated for axum 0.8
- [Elegant Error Handling in Axum/Actix Web](https://leapcell.io/blog/elegant-error-handling-in-axum-actix-web-with-intoresponse) - IntoResponse patterns
- [API Development in Rust: CORS, Tower Middleware, and Axum](https://dev.to/amaendeepm/api-development-in-rust-cors-tower-middleware-and-the-power-of-axum-397k) - CORS integration examples

### Tertiary (LOW confidence)
- [GitHub Issues/Discussions on axum](https://github.com/tokio-rs/axum) - Community patterns and edge cases
- [Tower-HTTP Releases](https://github.com/tower-rs/tower-http/releases) - Version history for 0.6.x

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Official Tokio documentation, recent release announcements, established ecosystem
- Architecture: HIGH - Multiple official examples, production guides from 2026, verified patterns
- Pitfalls: MEDIUM - Mix of official docs and community experience; some pitfalls inferred from discussions

**Research date:** 2026-02-02
**Valid until:** ~30 days (stable ecosystem, axum 0.8 recently released, patterns unlikely to change rapidly)
