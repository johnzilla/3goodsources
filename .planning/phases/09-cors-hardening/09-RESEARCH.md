# Phase 9: CORS Hardening - Research

**Researched:** 2026-02-08
**Domain:** CORS configuration with tower-http and axum for MCP server
**Confidence:** HIGH

## Summary

CORS hardening for this Rust/axum MCP server requires migrating from `CorsLayer::permissive()` to a specific origin allowlist configuration. The project currently uses tower-http 0.6 with axum 0.8, which provides the `CorsLayer` middleware for CORS header management.

The primary challenge is configuring CORS for MCP (Model Context Protocol) servers, which have specific requirements beyond standard REST APIs. MCP servers need to handle cross-origin POST requests to the `/mcp` endpoint, support browser preflight OPTIONS requests, and expose MCP-specific headers like `Mcp-Session-Id` to browser clients.

The migration path is straightforward: replace `.layer(CorsLayer::permissive())` with `.layer(CorsLayer::new().allow_origin([...]).allow_methods([...]).allow_headers([...]).expose_headers([...]))` using the specific domains from requirements (3gs.ai, api.3gs.ai). This phase validates the hardened CORS on Render before the Phase 10 migration to DigitalOcean.

**Primary recommendation:** Use `CorsLayer::new().allow_origin()` with an explicit list of allowed origins, configure required methods (GET, POST, OPTIONS), allow standard headers (content-type, authorization), and expose MCP-specific headers for browser client access.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tower-http | 0.6 | HTTP middleware including CORS | Official tower ecosystem library, maintained by tokio-rs, de facto standard for axum middleware |
| axum | 0.8 | Web framework | Already in use, tower-http designed to integrate with axum |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| reqwest | 0.12 | HTTP client for testing | Integration tests validating CORS behavior (already in dev-dependencies) |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| tower-http CorsLayer | Custom middleware | Custom middleware requires reimplementing preflight handling, header validation, Vary header management - high complexity for no benefit |
| tower-http CorsLayer | axum-cors (third-party) | axum-cors is experimental/unmaintained; tower-http is official and well-tested |

**Installation:**
Already installed. No changes to `Cargo.toml` required.

## Architecture Patterns

### Recommended Project Structure
```
src/
├── server.rs           # CORS configuration lives here with router setup
├── config.rs           # Could add ALLOWED_ORIGINS constant here
└── ...
```

### Pattern 1: Explicit Origin Allowlist
**What:** Configure `CorsLayer` with specific allowed origins using `allow_origin()` with an array of parsed origins.

**When to use:** Production deployments requiring security (always, except development/prototyping).

**Example:**
```rust
// Source: https://github.com/tower-rs/tower-http/blob/main/tower-http/src/cors/allow_origin.rs
use axum::http::{header, HeaderValue, Method};
use tower_http::cors::CorsLayer;

let cors = CorsLayer::new()
    .allow_origin([
        "https://3gs.ai".parse::<HeaderValue>().unwrap(),
        "https://api.3gs.ai".parse::<HeaderValue>().unwrap(),
    ])
    .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
    .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
    .max_age(Duration::from_secs(3600));

let app = Router::new()
    .route("/mcp", post(mcp_endpoint))
    .route("/health", get(health_endpoint))
    .route("/registry", get(registry_endpoint))
    .layer(cors)
    .with_state(state);
```

### Pattern 2: MCP-Specific Header Exposure
**What:** Use `expose_headers()` to make MCP protocol headers accessible to browser clients.

**When to use:** When building MCP servers that will be accessed by browser-based clients.

**Example:**
```rust
// Source: https://mcpcat.io/guides/implementing-cors-policies-web-based-mcp-servers/
use axum::http::HeaderName;

let cors = CorsLayer::new()
    .allow_origin([...])
    .allow_methods([...])
    .allow_headers([...])
    .expose_headers([
        HeaderName::from_static("mcp-session-id"),
        HeaderName::from_static("x-request-id"),
    ])
    .max_age(Duration::from_secs(3600));
```

### Pattern 3: Environment-Based Configuration
**What:** Use environment variables or feature flags to switch between development (permissive) and production (restricted) CORS.

**When to use:** When you need different CORS policies for local development vs production.

**Example:**
```rust
let cors = if cfg!(debug_assertions) {
    // Development: permissive
    CorsLayer::permissive()
} else {
    // Production: restricted
    CorsLayer::new()
        .allow_origin([...])
        .allow_methods([...])
};
```

**Note:** For this phase, we're removing permissive mode entirely (success criterion 3), so this pattern is informational only.

### Anti-Patterns to Avoid
- **Using `CorsLayer::permissive()` in production:** Allows any origin with wildcard `*`, defeats the purpose of CORS security.
- **Multiple `allow_origin()` calls:** Each call overrides the previous one. Use a single call with an array/vec of origins.
- **Combining `allow_credentials(true)` with wildcard origin:** Browsers will reject this combination for security reasons. Never use `allow_credentials(true)` with `Any` origin.
- **Forgetting to configure `expose_headers()` for custom headers:** MCP protocol headers like `Mcp-Session-Id` won't be accessible to JavaScript without explicit exposure.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| CORS header validation | Custom middleware checking Origin header | `tower_http::cors::CorsLayer` | Preflight handling is complex (OPTIONS method routing, caching headers, Vary header management, spec compliance) |
| Origin matching | String comparison or regex matching | `CorsLayer::allow_origin()` with list or predicate | Built-in handles HeaderValue parsing, error cases, and spec-compliant matching |
| Preflight response caching | Custom Cache-Control logic | `CorsLayer::max_age()` | Access-Control-Max-Age header has specific semantics; built-in handles Duration conversion and header formatting |
| Vary header management | Manual Vary header setting | `CorsLayer` (automatic) or `.vary()` | CorsLayer automatically sets Vary: Origin for non-wildcard configs; manual management risks cache poisoning |

**Key insight:** CORS is deceptively complex. The spec has edge cases around preflight requests, credential handling, wildcard restrictions, and caching semantics. tower-http's `CorsLayer` implements the full spec correctly, handles all these edge cases, and is battle-tested in production Rust services.

## Common Pitfalls

### Pitfall 1: Wildcard Origin with Credentials
**What goes wrong:** Setting `allow_credentials(true)` with `any()` origin causes browsers to reject the CORS response.

**Why it happens:** CORS spec forbids combining `Access-Control-Allow-Credentials: true` with `Access-Control-Allow-Origin: *` for security reasons (would expose authenticated content to any origin).

**How to avoid:** Never use `allow_credentials(true)` with `CorsLayer::permissive()` or `AllowOrigin::any()`. Always use specific origins when credentials are needed.

**Warning signs:** Browser console error: "Credential is not supported if the CORS header 'Access-Control-Allow-Origin' is '*'"

**Source:** https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/CORS/Errors/CORSNotSupportingCredentials

### Pitfall 2: Missing Preflight Method Configuration
**What goes wrong:** Browser sends OPTIONS preflight request, but server doesn't include POST in `Access-Control-Allow-Methods`, causing the actual POST request to fail.

**Why it happens:** `CorsLayer` defaults to limited methods. If you don't explicitly allow POST, preflight validation fails.

**How to avoid:** Always explicitly configure all methods your API uses: `.allow_methods([Method::GET, Method::POST, Method::OPTIONS])`.

**Warning signs:** Preflight succeeds (200 OK), but browser blocks the actual POST request with CORS error.

**Source:** https://developer.mozilla.org/en-US/docs/Glossary/Preflight_request

### Pitfall 3: Forgetting to Expose Custom Headers
**What goes wrong:** Server sends custom headers (like `Mcp-Session-Id`), but JavaScript can't read them via `response.headers.get()`.

**Why it happens:** By default, only CORS-safe response headers (Cache-Control, Content-Type, etc.) are exposed to JavaScript. Custom headers require explicit `Access-Control-Expose-Headers`.

**How to avoid:** Use `.expose_headers([...])` for all custom protocol headers that browser clients need to read.

**Warning signs:** Network tab shows header in response, but `response.headers.get('Mcp-Session-Id')` returns null in JavaScript.

**Source:** https://mcpcat.io/guides/implementing-cors-policies-web-based-mcp-servers/

### Pitfall 4: Protocol Mismatch (http vs https)
**What goes wrong:** Browser origin is `https://3gs.ai`, but CORS config specifies `http://3gs.ai` - browser rejects as different origin.

**Why it happens:** CORS origin matching is exact: protocol, domain, and port must all match.

**How to avoid:** Use HTTPS URLs in production CORS configuration. For development, ensure localhost URLs match the dev server's actual protocol and port.

**Warning signs:** CORS error despite origin appearing to match; double-check protocol in browser DevTools Network tab.

### Pitfall 5: Missing Content-Type in Allowed Headers
**What goes wrong:** POST requests with `Content-Type: application/json` trigger preflight, but preflight fails because `Content-Type` isn't in `Access-Control-Allow-Headers`.

**Why it happens:** Requests with `Content-Type: application/json` require preflight (not a "simple request"). The preflight checks allowed headers.

**How to avoid:** Always include `header::CONTENT_TYPE` in `.allow_headers()` for APIs that accept JSON.

**Warning signs:** GET requests work, POST requests fail with preflight CORS error.

**Source:** https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/CORS

### Pitfall 6: Excessive Vary Header Setting
**What goes wrong:** tower-http's `CorsLayer` sets `Vary: origin, access-control-request-method, access-control-request-headers` unconditionally, even for permissive configurations, potentially causing caching issues.

**Why it happens:** This is a known issue in tower-http (Issue #539). The middleware sets Vary headers regardless of whether the response is actually dynamic.

**How to avoid:** For production with specific origins (non-wildcard), this is correct behavior. If using permissive mode, be aware of potential CDN/cache implications.

**Warning signs:** Unexpected cache MISS rates on CDNs or proxies.

**Source:** https://github.com/tower-rs/tower-http/issues/539

## Code Examples

Verified patterns from official sources:

### Complete Production CORS Setup for MCP Server
```rust
// Sources:
// - https://docs.rs/tower-http/latest/tower_http/cors/index.html
// - https://mcpcat.io/guides/implementing-cors-policies-web-based-mcp-servers/
// - https://github.com/tower-rs/tower-http/blob/main/tower-http/src/cors/allow_origin.rs

use axum::{
    http::{header, HeaderValue, Method},
    routing::{get, post},
    Router,
};
use std::time::Duration;
use tower_http::cors::CorsLayer;

pub fn build_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        // Specific origins only (requirement: 3gs.ai, api.3gs.ai)
        .allow_origin([
            "https://3gs.ai".parse::<HeaderValue>().unwrap(),
            "https://api.3gs.ai".parse::<HeaderValue>().unwrap(),
        ])
        // Methods required for MCP: GET for SSE, POST for requests, OPTIONS for preflight
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        // Standard headers for JSON API + Authorization for future auth
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
        // Expose MCP protocol headers to browser clients
        .expose_headers([
            HeaderValue::from_static("mcp-session-id"),
            HeaderValue::from_static("x-request-id"),
        ])
        // Cache preflight responses for 1 hour to reduce overhead
        .max_age(Duration::from_secs(3600));

    Router::new()
        .route("/mcp", post(mcp_endpoint))
        .route("/health", get(health_endpoint))
        .route("/registry", get(registry_endpoint))
        .layer(cors)  // Apply CORS middleware to all routes
        .with_state(state)
}
```

### Testing CORS Preflight with curl
```bash
# Source: https://gist.github.com/madis/4650014
# Test OPTIONS preflight for POST /mcp request
curl \
  --verbose \
  --request OPTIONS \
  https://api.3gs.ai/mcp \
  --header 'Origin: https://3gs.ai' \
  --header 'Access-Control-Request-Method: POST' \
  --header 'Access-Control-Request-Headers: Content-Type'

# Expected response headers:
# Access-Control-Allow-Origin: https://3gs.ai
# Access-Control-Allow-Methods: GET, POST, OPTIONS
# Access-Control-Allow-Headers: content-type, authorization
# Access-Control-Max-Age: 3600
# Vary: origin, access-control-request-method, access-control-request-headers
```

### Integration Test Pattern for CORS
```rust
// Source: Pattern derived from https://www.ruststepbystep.com/how-to-test-axum-apis-unit-and-integration-testing-guide/
#[tokio::test]
async fn test_cors_preflight_mcp_endpoint() {
    let app = build_test_router();

    let response = reqwest::Client::new()
        .request(Method::OPTIONS, "http://localhost:3000/mcp")
        .header("Origin", "https://3gs.ai")
        .header("Access-Control-Request-Method", "POST")
        .header("Access-Control-Request-Headers", "content-type")
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("access-control-allow-origin").unwrap(),
        "https://3gs.ai"
    );
    assert!(response.headers()
        .get("access-control-allow-methods")
        .unwrap()
        .to_str()
        .unwrap()
        .contains("POST"));
}

#[tokio::test]
async fn test_cors_actual_post_request() {
    let app = build_test_router();

    let response = reqwest::Client::new()
        .post("http://localhost:3000/mcp")
        .header("Origin", "https://3gs.ai")
        .header("Content-Type", "application/json")
        .body(r#"{"jsonrpc":"2.0","method":"tools/list","id":1}"#)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("access-control-allow-origin").unwrap(),
        "https://3gs.ai"
    );
}

#[tokio::test]
async fn test_cors_rejects_unlisted_origin() {
    let app = build_test_router();

    let response = reqwest::Client::new()
        .post("http://localhost:3000/mcp")
        .header("Origin", "https://evil.com")
        .header("Content-Type", "application/json")
        .body(r#"{"jsonrpc":"2.0","method":"tools/list","id":1}"#)
        .send()
        .await
        .unwrap();

    // Server should NOT include Access-Control-Allow-Origin for unlisted origins
    assert!(response.headers().get("access-control-allow-origin").is_none());
}
```

### AllowOrigin API Usage Patterns
```rust
// Source: https://github.com/tower-rs/tower-http/blob/main/tower-http/src/cors/allow_origin.rs

use tower_http::cors::AllowOrigin;
use axum::http::HeaderValue;

// Pattern 1: Explicit list (RECOMMENDED for production)
let origins = AllowOrigin::list([
    "https://3gs.ai".parse::<HeaderValue>().unwrap(),
    "https://api.3gs.ai".parse::<HeaderValue>().unwrap(),
]);

// Pattern 2: Single origin
let origin = AllowOrigin::exact("https://3gs.ai".parse().unwrap());

// Pattern 3: Predicate for dynamic matching (e.g., wildcard subdomain)
let origins = AllowOrigin::predicate(|origin: &HeaderValue, _parts| {
    origin.to_str()
        .map(|s| s.ends_with(".3gs.ai") || s == "https://3gs.ai")
        .unwrap_or(false)
});

// Pattern 4: Any origin (AVOID in production)
let origins = AllowOrigin::any(); // Results in Access-Control-Allow-Origin: *

// Pattern 5: Mirror request (accepts any origin dynamically)
let origins = AllowOrigin::mirror_request(); // Reflects request origin back
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Custom CORS middleware | tower-http CorsLayer | tower-http 0.1+ (2021) | Standardized CORS for Tower ecosystem; official middleware recommended for all axum apps |
| Permissive CORS by default | Explicit configuration required | Always | Security best practice: no wildcard origins in production unless intentional |
| Manual Vary header management | Automatic Vary header | tower-http 0.2+ | Prevents cache poisoning; middleware handles it correctly |

**Deprecated/outdated:**
- **axum-cors crate:** Experimental third-party crate, unmaintained. Use tower-http instead.
- **`CorsLayer::very_permissive()`:** Deprecated in tower-http 0.5+. Use `permissive()` or explicit configuration.

## Open Questions

1. **Should we support localhost origins for development/testing?**
   - What we know: Requirements specify only 3gs.ai and api.3gs.ai
   - What's unclear: Whether developers or CI will need to test CORS behavior with localhost origins
   - Recommendation: Add localhost variants ONLY to dev environment configuration if needed (via cfg! or env var), not to production. For this phase, stick strictly to requirements.

2. **Do we need `allow_credentials(true)`?**
   - What we know: No current authentication mechanism in the codebase (no cookie-based auth, no OAuth)
   - What's unclear: Future authentication plans
   - Recommendation: Do NOT add `allow_credentials(true)` in this phase. Add it when authentication is implemented (likely Phase 12+). Credentials requirement would constrain CORS further (can't use wildcard, must include credentials in requests).

3. **Should we expose additional response headers beyond MCP-specific ones?**
   - What we know: MCP spec mentions `Mcp-Session-Id` as critical for browser clients
   - What's unclear: Whether other custom headers will be added in future
   - Recommendation: Start with minimal exposure (`mcp-session-id`, `x-request-id`). Add more via `.expose_headers()` if future features require it.

4. **Do we need to support SSE (Server-Sent Events) endpoints?**
   - What we know: MCP protocol mentions SSE for bidirectional communication; current implementation is HTTP JSON-RPC only
   - What's unclear: Roadmap for SSE support
   - Recommendation: Current CORS config (GET method allowed) will work for future SSE endpoints. No special configuration needed now.

## Sources

### Primary (HIGH confidence)
- https://docs.rs/tower-http/latest/tower_http/cors/index.html - tower-http official documentation (current version)
- https://github.com/tower-rs/tower-http/blob/main/tower-http/src/cors/allow_origin.rs - AllowOrigin source code and API
- https://tidelabs.github.io/tidechain/tower_http/cors/struct.CorsLayer.html - CorsLayer method documentation
- https://mcpcat.io/guides/implementing-cors-policies-web-based-mcp-servers/ - MCP-specific CORS requirements (2026)
- https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/CORS - MDN CORS specification reference

### Secondary (MEDIUM confidence)
- https://www.ruststepbystep.com/how-to-handle-cors-in-rust-with-axum-a-step-by-step-guide/ - Axum CORS tutorial with code examples (2025)
- https://www.ruststepbystep.com/how-to-test-axum-apis-unit-and-integration-testing-guide/ - Axum testing patterns (2025)
- https://gist.github.com/madis/4650014 - curl commands for CORS preflight testing
- https://corsfix.com/cors-headers/access-control-max-age - Access-Control-Max-Age header explanation
- https://developer.mozilla.org/en-US/docs/Glossary/Preflight_request - Preflight request specification

### Tertiary (LOW confidence)
- https://github.com/tower-rs/tower-http/issues/539 - Vary header issue (known bug, low priority)
- https://github.com/tower-rs/tower-http/issues/397 - Vary header blocking issue (resolved in newer versions)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - tower-http is official, well-documented, current version in use (0.6)
- Architecture: HIGH - Patterns verified from official docs and source code; MCP requirements from official guide
- Pitfalls: MEDIUM-HIGH - Most pitfalls verified from MDN/official sources; tower-http specific issues from GitHub issues (lower confidence)

**Research date:** 2026-02-08
**Valid until:** 2026-03-08 (30 days - stable domain, tower-http is mature library with infrequent breaking changes)
