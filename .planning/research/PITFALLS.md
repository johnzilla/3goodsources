# Domain Pitfalls: Rust MCP Server with Pubky Integration

**Domain:** MCP server (Rust) with decentralized identity (Pubky/PKARR)
**Researched:** 2026-02-01
**Confidence:** MEDIUM (based on MCP specification knowledge, Rust deployment patterns, and decentralized identity system design principles)

## Critical Pitfalls

Mistakes that cause complete failure, rewrites, or security breaches.

### Pitfall 1: JSON-RPC Protocol Violations Breaking MCP Clients

**What goes wrong:** MCP clients (Claude Desktop, IDEs) silently fail or reject the server because JSON-RPC responses don't match the spec.

**Why it happens:**
- Missing required `jsonrpc: "2.0"` field in every response
- Returning `result` AND `error` in the same response (spec requires exactly one)
- Using custom error codes instead of JSON-RPC standard codes (-32700 to -32603)
- Not handling `id: null` for notifications properly
- Missing required fields in capability negotiation (`protocolVersion`, `capabilities`, `serverInfo`)

**Consequences:**
- Server appears "broken" in Claude Desktop but works in curl tests
- No error messages (client just doesn't load the server)
- Impossible to debug without packet capture
- MVP completely fails to integrate with target clients

**Prevention:**
```rust
// WRONG: Missing jsonrpc field
json!({ "result": data, "id": id })

// RIGHT: Always include all required fields
json!({
    "jsonrpc": "2.0",
    "result": data,
    "id": id
})

// WRONG: Returning both result and error
json!({ "jsonrpc": "2.0", "result": null, "error": {...}, "id": id })

// RIGHT: Only one of result or error
json!({ "jsonrpc": "2.0", "error": {...}, "id": id })
```

**Detection:**
- Claude Desktop doesn't show server in list
- MCP inspector tools show "invalid response"
- Wireshark/tcpdump shows responses missing fields
- Integration tests with real MCP clients fail

**Phase mapping:** Must be addressed in Phase 1 (Core MCP). Create integration tests with real MCP client libraries, not just curl.

### Pitfall 2: Private Key Leakage in Version Control

**What goes wrong:** PKARR private keys (seed phrases or raw keys) get committed to git and pushed to GitHub.

**Why it happens:**
- Storing keys in `.env` files without proper `.gitignore`
- Hardcoding test keys in source code
- Logging key material during debugging
- Accidentally committing `.env.example` with real keys instead of placeholders
- Docker build copying all files including local key storage

**Consequences:**
- Anyone can impersonate the server's Pubky identity
- Registry data can be modified by attackers
- Complete loss of trust/reputation
- No way to revoke (PKARR keys can't be rotated without new identity)

**Prevention:**
```bash
# .gitignore - Add BEFORE creating any key files
.env
.env.local
*.key
*.pem
keys/
secrets/

# Use environment variables, never hardcode
# WRONG:
const PRIVATE_KEY: &str = "a1b2c3...";

# RIGHT:
let private_key = env::var("PKARR_PRIVATE_KEY")
    .expect("PKARR_PRIVATE_KEY must be set");
```

**Additional safeguards:**
- Use git-secrets or gitleaks pre-commit hooks
- Store keys in Render environment variables, not in code
- Document key generation separately from deployment
- Use different keys for development and production
- Never log full keys (only last 4 chars for debugging)

**Detection:**
- Run `git log -p | grep -i "private\|key\|seed"` to scan history
- Use GitHub secret scanning (will alert if keys pushed)
- Check `.gitignore` before generating any keys

**Phase mapping:** Address in Phase 0 (project setup). Create `.gitignore` and key management docs before writing any Pubky code.

### Pitfall 3: Unbounded Registry Data Causing Memory Exhaustion

**What goes wrong:** Server loads entire registry.json into memory, then crashes when file grows beyond available RAM (512MB on Render free tier).

**Why it happens:**
- Deserializing full JSON into heap-allocated structures
- Not implementing streaming/pagination
- Keeping all category data in memory for fuzzy matching
- Render free tier has strict 512MB limit
- Cold starts already consume 100-200MB for Rust binary

**Consequences:**
- Server crashes under load
- OOM killer terminates process
- No graceful degradation
- MVP fails at modest scale (100+ categories, 1000+ sources)

**Prevention:**
```rust
// WRONG: Load everything
let registry: Registry = serde_json::from_str(&file_contents)?;

// RIGHT: Implement size limits and streaming
const MAX_REGISTRY_SIZE: usize = 10 * 1024 * 1024; // 10MB
if file_contents.len() > MAX_REGISTRY_SIZE {
    return Err("Registry too large");
}

// Consider lazy loading or pagination for large datasets
// Keep only indexes in memory, load full data on demand
```

**Memory budget planning:**
- Rust binary: ~100-150MB
- Registry data: limit to 50MB parsed
- Request buffers: 20MB
- Remaining: 292MB for heap/stack
- **Total: Stay under 512MB**

**Detection:**
- Monitor `mem_usage` in Render metrics
- Load test with large registry files locally
- Set up alerts for >400MB usage
- Test with `ulimit -v 512000` locally

**Phase mapping:** Address in Phase 1 (Core MCP). Implement size limits and streaming before deploying to Render.

### Pitfall 4: Pubky SDK Breaking Changes Without Versioning

**What goes wrong:** Pubky SDK releases breaking API changes, server code stops compiling or behaves incorrectly.

**Why it happens:**
- Pubky is pre-1.0, no semver guarantees
- Homeserver protocol changes break clients
- Rust SDK might reorganize modules or change function signatures
- No LTS releases or compatibility guarantees

**Consequences:**
- `cargo update` breaks builds
- Deployed server can't communicate with homeservers
- No clear migration path
- Feature development blocked by SDK issues

**Prevention:**

**Pin exact versions in Cargo.toml:**
```toml
# WRONG: Allow patch updates
pubky = "0.1"

# RIGHT: Pin exact version
pubky = "=0.1.3"
```

**Create abstraction layer:**
```rust
// Isolate Pubky SDK behind a trait
pub trait IdentityProvider {
    fn verify_identity(&self, id: &str) -> Result<bool>;
    fn publish_data(&self, data: &[u8]) -> Result<()>;
}

// Implement for Pubky
pub struct PubkyIdentity { /* ... */ }
impl IdentityProvider for PubkyIdentity { /* ... */ }

// Implement local fallback
pub struct LocalIdentity { /* ... */ }
impl IdentityProvider for LocalIdentity { /* ... */ }
```

**Benefits:**
- SDK changes only affect one module
- Easy to swap implementations
- Can test without Pubky infrastructure
- Graceful degradation if Pubky unavailable

**Detection:**
- Pin versions and only update intentionally
- Subscribe to Pubky SDK changelog/releases
- Test against SDK updates in separate branch first
- Monitor homeserver status pages

**Phase mapping:** Address in Phase 2 (Pubky Integration). Create abstraction layer from the start, don't tightly couple to SDK.

### Pitfall 5: CORS Misconfiguration Blocking Browser Clients

**What goes wrong:** Browser-based MCP clients can't connect because CORS headers are missing or wrong.

**Why it happens:**
- Assuming MCP is server-to-server only (but some clients are browser-based)
- Setting `Access-Control-Allow-Origin: *` but missing other required headers
- Not handling OPTIONS preflight requests
- Blocking `Content-Type: application/json` in preflight

**Consequences:**
- Browser clients show "CORS error" in console
- Server logs show 200 OK but client never receives data
- curl works fine but browsers fail
- Impossible to use web-based MCP tools

**Prevention:**
```rust
// Required CORS headers for MCP
response.headers_mut().insert(
    "Access-Control-Allow-Origin",
    "*".parse().unwrap() // Or specific origins if needed
);
response.headers_mut().insert(
    "Access-Control-Allow-Methods",
    "POST, OPTIONS".parse().unwrap()
);
response.headers_mut().insert(
    "Access-Control-Allow-Headers",
    "Content-Type".parse().unwrap()
);

// Handle OPTIONS preflight
if request.method() == Method::OPTIONS {
    return Ok(Response::builder()
        .status(204)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "POST, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type")
        .body(Body::empty())?);
}
```

**Detection:**
- Test with browser fetch() API, not just curl
- Check browser DevTools console for CORS errors
- Verify OPTIONS requests return 204 with correct headers
- Use online CORS checkers

**Phase mapping:** Address in Phase 1 (Core MCP). Add CORS from the start, easier than retrofitting.

## Moderate Pitfalls

Mistakes that cause delays, technical debt, or degraded UX.

### Pitfall 6: Fuzzy Matching Too Aggressive (False Positives)

**What goes wrong:** Query "cat" matches "authentication" and "communication" because both contain "cat".

**Why it happens:**
- Using simple substring matching instead of word boundaries
- Levenshtein distance threshold too high
- Not weighting match position (prefix vs suffix vs middle)
- Not considering query intent

**Consequences:**
- Users get irrelevant categories
- Trust in recommendations decreases
- "It's faster to browse than search" (search becomes useless)

**Prevention:**
```rust
// WRONG: Simple substring
category.name.contains(&query)

// BETTER: Word boundary aware
let words: Vec<&str> = category.name.split_whitespace().collect();
words.iter().any(|w| w.starts_with(&query))

// BEST: Weighted scoring
fn match_score(query: &str, category: &str) -> f32 {
    if category.starts_with(query) { return 1.0; }
    if category.contains(&format!(" {}", query)) { return 0.8; }
    if category.contains(query) { return 0.5; }
    // Levenshtein distance for fuzzy matching
    let distance = levenshtein(query, category);
    if distance <= 2 { return 0.3; }
    0.0
}
```

**Detection:**
- Test with common queries (cat, car, test, data)
- Ask users "Was this relevant?" feedback
- Monitor match score distribution
- A/B test different thresholds

**Phase mapping:** Address in Phase 3 (Fuzzy Matching). Start with simple prefix matching, iterate based on usage.

### Pitfall 7: Cold Start Times Exceeding 30 Seconds on Render

**What goes wrong:** First request after idle takes 30+ seconds, users think server is down.

**Why it happens:**
- Render free tier spins down after 15 minutes idle
- Rust binary size is large (20-50MB)
- Docker image pull is slow
- Registry loading happens synchronously on startup

**Consequences:**
- Poor UX for first user of the day
- Timeout errors in MCP clients
- Appears unreliable/broken

**Prevention:**

**Optimize Docker image size:**
```dockerfile
# Use multi-stage build
FROM rust:1.75 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Use minimal runtime
FROM debian:bookworm-slim
COPY --from=builder /app/target/release/server /usr/local/bin/
CMD ["server"]
```

**Lazy load registry:**
```rust
// WRONG: Load on startup
let registry = load_registry().await?;
server.start(registry).await?;

// RIGHT: Load on first request
lazy_static! {
    static ref REGISTRY: Mutex<Option<Registry>> = Mutex::new(None);
}

async fn get_registry() -> &'static Registry {
    let mut reg = REGISTRY.lock().unwrap();
    if reg.is_none() {
        *reg = Some(load_registry().await);
    }
    reg.as_ref().unwrap()
}
```

**External keep-alive:**
- Set up UptimeRobot or similar to ping every 5 minutes
- Prevents Render from spinning down
- Free tier allows this

**Detection:**
- Measure time from container start to first successful request
- Monitor Render deployment logs for startup time
- Test after 20+ minutes of no traffic

**Phase mapping:** Address in Phase 4 (Render Deployment). Optimize only after core functionality works.

### Pitfall 8: Missing Error Context in JSON-RPC Errors

**What goes wrong:** Client receives `{"error": {"code": -32603, "message": "Internal error"}}` with no actionable information.

**Why it happens:**
- Catching all errors and returning generic message
- Not including `error.data` field with details
- Afraid to leak internal details (overly defensive)

**Consequences:**
- Debugging is impossible without server logs
- Users can't fix client-side issues
- Support burden increases

**Prevention:**
```rust
// WRONG: Generic error
Err(Error {
    code: -32603,
    message: "Internal error".into(),
    data: None,
})

// RIGHT: Specific error with context
Err(Error {
    code: -32602, // Invalid params
    message: "Query parameter too long".into(),
    data: Some(json!({
        "max_length": 200,
        "actual_length": query.len(),
        "field": "query"
    })),
})
```

**Balance security and debuggability:**
- Include constraints (max length, allowed values)
- Don't include stack traces or file paths
- Log full error server-side, return sanitized version to client

**Detection:**
- Trigger each error condition and check response
- Verify `error.data` provides actionable information
- Test error messages with non-developer users

**Phase mapping:** Address in Phase 1 (Core MCP). Define error types early, easier than retrofitting.

### Pitfall 9: Registry Malformed JSON Not Validated on Load

**What goes wrong:** Server loads registry.json with missing fields or invalid data, then crashes on first query.

**Why it happens:**
- Trusting `serde_json` to validate everything
- Not checking business rules (unique slugs, valid URLs)
- Assuming human-edited JSON is always correct

**Consequences:**
- Server starts successfully but crashes on queries
- Difficult to debug (error happens far from root cause)
- No clear error message about which entry is malformed

**Prevention:**
```rust
#[derive(Deserialize)]
struct Category {
    slug: String,
    name: String,
    sources: Vec<Source>,
}

fn validate_registry(registry: &Registry) -> Result<(), ValidationError> {
    // Check for duplicate slugs
    let mut seen_slugs = HashSet::new();
    for category in &registry.categories {
        if !seen_slugs.insert(&category.slug) {
            return Err(ValidationError::DuplicateSlug(category.slug.clone()));
        }

        // Validate slug format
        if !category.slug.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err(ValidationError::InvalidSlug(category.slug.clone()));
        }

        // Validate source URLs
        for source in &category.sources {
            Url::parse(&source.url)
                .map_err(|_| ValidationError::InvalidUrl(source.url.clone()))?;
        }
    }
    Ok(())
}

// Load with validation
let registry: Registry = serde_json::from_str(&contents)?;
validate_registry(&registry)?; // Fail fast if invalid
```

**Detection:**
- Create test registry with known bad data
- Verify each validation rule triggers correct error
- Test with empty arrays, null fields, extra fields

**Phase mapping:** Address in Phase 1 (Core MCP). Validation prevents entire class of runtime errors.

### Pitfall 10: Not Handling Batch JSON-RPC Requests

**What goes wrong:** Client sends batch request `[{...}, {...}]`, server returns 400 Bad Request.

**Why it happens:**
- Only implementing single-request handler
- Not checking if JSON is array vs object
- Assuming MCP clients never batch

**Consequences:**
- Some MCP clients fail to use server
- Server appears non-compliant with JSON-RPC 2.0
- Confusing errors for batch-capable clients

**Prevention:**
```rust
// Check if request is array (batch) or object (single)
let json_value: serde_json::Value = serde_json::from_slice(&body)?;

match json_value {
    Value::Array(requests) => {
        // Handle batch
        let responses: Vec<_> = requests
            .into_iter()
            .map(|req| handle_request(req))
            .collect();
        Ok(json_response(responses))
    }
    Value::Object(_) => {
        // Handle single request
        let response = handle_request(json_value)?;
        Ok(json_response(response))
    }
    _ => Err(Error::InvalidRequest),
}
```

**Detection:**
- Test with batch request: `[{"jsonrpc":"2.0","method":"list","id":1},{"jsonrpc":"2.0","method":"search","params":{"query":"test"},"id":2}]`
- Verify response is array with matching IDs
- Check that partial failures in batch are handled correctly

**Phase mapping:** Can defer to post-MVP if no clients use batching. Add if MCP spec requires or clients request it.

## Minor Pitfalls

Mistakes that cause annoyance but are easily fixed.

### Pitfall 11: Not Trimming/Normalizing User Queries

**What goes wrong:** Query " Test " doesn't match "test" due to leading/trailing whitespace and case sensitivity.

**Prevention:**
```rust
let normalized_query = query.trim().to_lowercase();
```

**Phase mapping:** Phase 3 (Fuzzy Matching).

### Pitfall 12: Missing Health Check Endpoint

**What goes wrong:** Render can't determine if server is ready, marks it unhealthy prematurely.

**Prevention:**
```rust
// Add GET /health endpoint
if request.uri().path() == "/health" {
    return Ok(Response::new(Body::from("OK")));
}
```

**Phase mapping:** Phase 4 (Render Deployment).

### Pitfall 13: Hardcoded PORT Instead of Reading Environment

**What goes wrong:** Server listens on 3000, Render assigns 10000, server unreachable.

**Prevention:**
```rust
let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
let addr = format!("0.0.0.0:{}", port).parse()?;
```

**Phase mapping:** Phase 4 (Render Deployment).

### Pitfall 14: Not Logging Request IDs for Debugging

**What goes wrong:** Can't correlate errors in logs with specific requests.

**Prevention:**
```rust
info!("Handling request id={} method={}", id, method);
```

**Phase mapping:** Phase 1 (Core MCP).

### Pitfall 15: Case-Sensitive Category Slug Matching

**What goes wrong:** Query references "Machine-Learning" but registry has "machine-learning", no match.

**Prevention:**
```rust
let slug_lower = slug.to_lowercase();
categories.iter().find(|c| c.slug.to_lowercase() == slug_lower)
```

**Phase mapping:** Phase 3 (Fuzzy Matching).

## Phase-Specific Warnings

| Phase | Likely Pitfall | Mitigation |
|-------|---------------|------------|
| Phase 0: Project Setup | Key leakage (Pitfall 2) | Create .gitignore FIRST, document key management |
| Phase 1: Core MCP | JSON-RPC violations (Pitfall 1) | Integration test with real MCP client, validate all responses |
| Phase 1: Core MCP | Missing error context (Pitfall 8) | Define error types early, include validation details |
| Phase 1: Core MCP | Registry validation (Pitfall 9) | Validate on load, fail fast with clear errors |
| Phase 2: Pubky Integration | SDK breaking changes (Pitfall 4) | Pin versions, create abstraction layer |
| Phase 2: Pubky Integration | CORS blocking browsers (Pitfall 5) | Add CORS headers from start, test with browser |
| Phase 3: Fuzzy Matching | Too aggressive matching (Pitfall 6) | Start simple (prefix), iterate with user feedback |
| Phase 4: Render Deployment | Cold start times (Pitfall 7) | Multi-stage Docker, lazy loading, external keepalive |
| Phase 4: Render Deployment | Memory exhaustion (Pitfall 3) | Test with 512MB limit locally, implement size limits |
| Phase 4: Render Deployment | Port binding (Pitfall 13) | Read PORT env var, test with different ports |

## Domain-Specific Testing Checklist

Before deploying to production, verify:

**MCP Protocol Compliance:**
- [ ] Every response includes `jsonrpc: "2.0"`
- [ ] Responses have exactly one of `result` or `error`
- [ ] Error codes match JSON-RPC spec (-32700 to -32603)
- [ ] Capability negotiation includes all required fields
- [ ] OPTIONS requests return 204 with CORS headers
- [ ] Integration test with Claude Desktop succeeds

**Pubky Integration:**
- [ ] Private keys never committed to git
- [ ] Keys stored in environment variables only
- [ ] Abstraction layer isolates SDK dependencies
- [ ] Local fallback works when Pubky unavailable
- [ ] SDK version pinned in Cargo.toml

**Resource Limits:**
- [ ] Registry file size under 10MB
- [ ] Memory usage under 400MB under load
- [ ] Cold start time under 15 seconds
- [ ] Health check responds within 1 second

**Data Validation:**
- [ ] Registry JSON validated on load
- [ ] Duplicate slugs detected and rejected
- [ ] Invalid URLs rejected with clear error
- [ ] Query length limits enforced
- [ ] Empty/null queries handled gracefully

## Confidence Assessment

**Overall confidence:** MEDIUM

**High confidence areas:**
- JSON-RPC protocol requirements (standard spec)
- Rust deployment patterns (well-documented)
- Memory limits on Render free tier (documented)
- CORS configuration (web standard)

**Medium confidence areas:**
- Pubky SDK maturity and breaking changes (pre-1.0, limited documentation)
- Specific MCP client validation behavior (Claude Desktop internals not public)
- Registry data scale limits (depends on schema complexity)

**Low confidence areas:**
- Exact Pubky homeserver reliability characteristics
- Future MCP protocol changes or extensions
- Render free tier performance characteristics under real load

**Recommendation:** Start with high-confidence preventions (JSON-RPC compliance, key management, resource limits). Validate medium-confidence areas through testing. Monitor low-confidence areas and adapt as issues arise.

## Sources

**Methodology:** Due to tool access limitations, this research is based on:
- JSON-RPC 2.0 specification (standard protocol)
- Model Context Protocol design principles (from training data)
- Rust deployment best practices (production experience patterns)
- Decentralized identity system design (PKARR/Pubky architecture)
- Render platform documentation (deployment constraints)

**Validation needed:**
- MCP specification for current required fields and error codes
- Pubky SDK documentation for API stability and versioning
- Render free tier limits (memory, cold start, network)
- Claude Desktop MCP integration requirements

**Recommended verification:**
- Review official MCP spec at spec.modelcontextprotocol.io
- Check Pubky SDK changelog for breaking changes
- Test deployed server with real MCP clients
- Monitor Render metrics during beta testing
