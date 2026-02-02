# Architecture Patterns: Rust MCP Server with Pubky Integration

**Project:** Three Good Sources (3GS)
**Researched:** 2026-02-01
**Confidence:** MEDIUM (based on Rust ecosystem patterns, MCP spec, and axum 0.8 best practices)

## Executive Summary

A Rust MCP server with Pubky integration follows a layered architecture with clear separation between protocol handling (MCP JSON-RPC), business logic (registry matching), and data loading (Pubky/local). The architecture uses axum 0.8's state management for shared resources, trait-based abstraction for pluggable data sources, and a unidirectional data flow from HTTP request to JSON-RPC response.

**Key architectural decisions:**
1. **HTTP-first transport**: Single POST endpoint at `/mcp` handling JSON-RPC 2.0
2. **Immutable registry**: Load once at startup, use `Arc<Registry>` (no RwLock overhead)
3. **Trait-based loaders**: `RegistryLoader` trait with `PubkyLoader` and `LocalLoader` implementations
4. **Stateless handlers**: All state in `AppState`, handlers are pure functions
5. **Error mapping**: Domain errors (`thiserror`) map to JSON-RPC error codes

## Recommended Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         HTTP Layer                          │
│                    axum 0.8 Router                          │
│                  POST /mcp (JSON-RPC)                       │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│                      MCP Protocol Layer                     │
│  - JSON-RPC 2.0 parsing/validation                         │
│  - Method dispatch (initialize, tools/list, tools/call)    │
│  - Error mapping to JSON-RPC format                        │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│                     Business Logic Layer                    │
│  - Query matching (fuzzy search, scoring, ranking)         │
│  - Intent pattern matching                                 │
│  - Result filtering and limiting                           │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│                       Data Layer                            │
│  - Registry (in-memory, Arc-wrapped)                       │
│  - RegistryLoader trait (Pubky or Local)                   │
│  - Pubky SDK client wrapper                                │
└─────────────────────────────────────────────────────────────┘
```

## Component Boundaries

### 1. HTTP Layer (main.rs + axum router)

**Responsibility:**
- Accept HTTP POST requests at `/mcp`
- Extract JSON body
- Pass to MCP protocol handler
- Return HTTP 200 with JSON-RPC response

**Communicates with:**
- MCP Protocol Layer (calls `handle_mcp_request`)

**State:**
- Owns `AppState` (Arc-wrapped, cloned into handlers)

**Key pattern:**
```rust
// axum 0.8 pattern with State extractor
async fn mcp_handler(
    State(state): State<AppState>,
    Json(request): Json<JsonRpcRequest>,
) -> Json<JsonRpcResponse>
```

---

### 2. MCP Protocol Layer (src/mcp/)

**Modules:**
- `mod.rs` - Public interface
- `protocol.rs` - JSON-RPC types (Request, Response, Error)
- `handlers.rs` - Method dispatch logic

**Responsibility:**
- Parse JSON-RPC 2.0 requests
- Validate `jsonrpc: "2.0"` field
- Dispatch to method handlers based on `method` field
- Map domain errors to JSON-RPC error codes
- Construct JSON-RPC responses

**Communicates with:**
- HTTP Layer (called by axum handler)
- Business Logic Layer (calls registry matching)

**State:**
- Stateless (receives AppState via function parameters)

**JSON-RPC 2.0 Message Formats:**

#### Request Format
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "search_sources",
    "arguments": {
      "query": "rust async programming"
    }
  }
}
```

#### Success Response Format
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "..."
      }
    ]
  }
}
```

#### Error Response Format
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": {
      "details": "Query must be non-empty"
    }
  }
}
```

**Standard JSON-RPC error codes:**
- `-32700`: Parse error (invalid JSON)
- `-32600`: Invalid Request (missing required fields)
- `-32601`: Method not found
- `-32602`: Invalid params
- `-32603`: Internal error

**MCP-specific methods:**

1. **initialize**
   - Request params: `{ "protocolVersion": "2024-11-05", "capabilities": {} }`
   - Response result: `{ "protocolVersion": "2024-11-05", "capabilities": { "tools": {} }, "serverInfo": { "name": "3gs", "version": "0.1.0" } }`

2. **tools/list**
   - Request params: `{}`
   - Response result: `{ "tools": [{ "name": "search_sources", "description": "...", "inputSchema": {...} }] }`

3. **tools/call**
   - Request params: `{ "name": "search_sources", "arguments": { "query": "..." } }`
   - Response result: `{ "content": [{ "type": "text", "text": "..." }] }`

---

### 3. Business Logic Layer (src/registry/)

**Modules:**
- `mod.rs` - Public interface, Registry struct
- `loader.rs` - RegistryLoader trait and implementations
- `matcher.rs` - Query matching pipeline

**Responsibility:**
- Store registry data in memory (Arc-wrapped)
- Load registry from Pubky or local JSON at startup
- Match queries against intent patterns
- Score, rank, and filter results
- Return top 3 sources

**Communicates with:**
- MCP Protocol Layer (called by tools/call handler)
- Data Layer (uses RegistryLoader at startup)

**State:**
- Registry data (Vec of Source entries)
- Wrapped in `Arc<Registry>` (immutable after load)

**Data structures:**
```rust
pub struct Registry {
    sources: Vec<Source>,
}

pub struct Source {
    pub id: String,
    pub name: String,
    pub url: String,
    pub description: String,
    pub categories: Vec<String>,
    pub keywords: Vec<String>,
    pub trust_score: f32,
}

pub struct MatchResult {
    pub source: Source,
    pub score: f32,
    pub match_reasons: Vec<String>,
}
```

**Query matching pipeline:**
1. **Preprocess query**: Lowercase, strip punctuation, tokenize
2. **Fuzzy matching**: Levenshtein distance on keywords/categories
3. **Keyword boosting**: Exact keyword matches get higher scores
4. **Threshold filtering**: Only keep matches above 0.3 score
5. **Ranking**: Sort by score descending, then trust_score
6. **Limiting**: Return top 3 results

---

### 4. Data Layer (src/pubky/)

**Modules:**
- `mod.rs` - Public interface
- `client.rs` - Pubky SDK wrapper
- `identity.rs` - Identity/keypair management

**Responsibility:**
- Abstract data loading behind `RegistryLoader` trait
- Implement PubkyLoader (fetch from homeserver)
- Implement LocalLoader (read from JSON file)
- Handle Pubky SDK initialization and client management

**Communicates with:**
- Business Logic Layer (called by Registry::load_from)

**State:**
- Pubky client (if using PubkyLoader)
- File path (if using LocalLoader)

**Trait abstraction:**
```rust
#[async_trait]
pub trait RegistryLoader: Send + Sync {
    async fn load(&self) -> Result<Vec<Source>, LoadError>;
}

pub struct PubkyLoader {
    client: PubkyClient,
    homeserver_url: String,
    identity_key: String,
}

pub struct LocalLoader {
    file_path: PathBuf,
}

#[async_trait]
impl RegistryLoader for PubkyLoader {
    async fn load(&self) -> Result<Vec<Source>, LoadError> {
        // Fetch from Pubky homeserver
    }
}

#[async_trait]
impl RegistryLoader for LocalLoader {
    async fn load(&self) -> Result<Vec<Source>, LoadError> {
        // Read from local JSON file
    }
}
```

---

### 5. Error Layer (src/error.rs)

**Responsibility:**
- Define domain-specific errors using `thiserror`
- Provide conversions to JSON-RPC error codes
- Centralize error handling logic

**Error types:**
```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Registry load failed: {0}")]
    RegistryLoad(String),

    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    #[error("Pubky client error: {0}")]
    PubkyClient(String),

    #[error("JSON parse error: {0}")]
    JsonParse(String),
}

impl AppError {
    pub fn to_jsonrpc_error(&self) -> (i32, String) {
        match self {
            AppError::InvalidQuery(msg) => (-32602, msg.clone()),
            AppError::RegistryLoad(msg) => (-32603, msg.clone()),
            AppError::PubkyClient(msg) => (-32603, msg.clone()),
            AppError::JsonParse(_) => (-32700, "Parse error".into()),
        }
    }
}
```

---

## Data Flow

### Startup Flow

```
1. main.rs reads config (CLI args or env vars)
   ↓
2. Determine loader type (Pubky vs Local)
   ↓
3. Create PubkyLoader or LocalLoader
   ↓
4. Registry::load_from(loader).await
   ↓
5. Wrap Registry in Arc<Registry>
   ↓
6. Create AppState { registry: Arc<Registry>, config }
   ↓
7. Build axum router with State(app_state)
   ↓
8. Start HTTP server on configured port
```

### Request Flow

```
1. HTTP POST /mcp arrives at axum router
   ↓
2. axum extracts State(AppState) and Json(JsonRpcRequest)
   ↓
3. mcp_handler calls dispatch_method(request, state)
   ↓
4. dispatch_method matches on request.method:
   - "initialize" → handle_initialize()
   - "tools/list" → handle_tools_list()
   - "tools/call" → handle_tools_call(params, registry)
   ↓
5. handle_tools_call extracts query from params.arguments
   ↓
6. Calls registry.search(query)
   ↓
7. registry.search runs matching pipeline:
   - Preprocess query
   - Fuzzy match against all sources
   - Score and rank
   - Return top 3
   ↓
8. Format results as MCP tool response content
   ↓
9. Wrap in JsonRpcResponse with id from request
   ↓
10. axum serializes to JSON and returns HTTP 200
```

---

## State Management (axum 0.8 AppState pattern)

### AppState Structure

```rust
#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<Registry>,
    pub config: Arc<Config>,
}

pub struct Config {
    pub pubky_enabled: bool,
    pub homeserver_url: Option<String>,
    pub fallback_path: PathBuf,
    pub match_threshold: f32,
}
```

### Why Arc<Registry> (not Arc<RwLock<Registry>>)

**Decision: Use `Arc<Registry>` (immutable after load)**

**Rationale:**
- Registry is loaded once at startup
- No runtime updates needed (hot-reload not required for MVP)
- Avoids RwLock overhead on every query
- Simpler reasoning (no lock contention)
- Better performance (read-only Arc is lock-free)

**Future consideration:**
- If hot-reload needed later, add reload endpoint that swaps Arc (atomic pointer swap)
- Pattern: `Arc<ArcSwap<Registry>>` from arc-swap crate

### State Access Pattern

```rust
// axum 0.8 State extractor (auto-clones Arc)
async fn mcp_handler(
    State(state): State<AppState>,
    Json(request): Json<JsonRpcRequest>,
) -> Result<Json<JsonRpcResponse>, JsonRpcError> {
    // state.registry is Arc<Registry> (cheap clone)
    let results = state.registry.search(&query).await?;
    // ...
}
```

**Key points:**
- `State(state)` in axum 0.8 auto-clones the Arc (cheap)
- No explicit `.clone()` needed
- Each handler invocation gets its own Arc pointer (reference count increment)
- Registry data itself is never copied

---

## Module Layout Analysis

### Proposed Structure

```
src/
  main.rs              # Entry point, server setup
  mcp/
    mod.rs             # Public interface
    handlers.rs        # Method dispatch, handler functions
    protocol.rs        # JSON-RPC types
  registry/
    mod.rs             # Registry struct, public interface
    loader.rs          # RegistryLoader trait + impls
    matcher.rs         # Query matching pipeline
  pubky/
    mod.rs             # Public interface
    client.rs          # Pubky SDK wrapper
    identity.rs        # Identity/keypair management
  error.rs             # Error types and conversions
```

### Idiomatic Assessment: GOOD

**Strengths:**
- Clear separation of concerns (protocol vs logic vs data)
- Each module has single responsibility
- Trait-based abstraction in correct module (loader.rs)
- Error handling centralized

**Suggested improvements:**

1. **Add types.rs in registry module** for shared types:
   ```
   src/registry/
     mod.rs
     types.rs         # Source, MatchResult, QueryMatch
     loader.rs        # RegistryLoader trait
     matcher.rs       # Matching pipeline
   ```

2. **Consider config.rs at root** if configuration grows:
   ```
   src/
     config.rs        # Config struct, CLI parsing
   ```

3. **Add tests/ subdirs** for integration tests:
   ```
   src/mcp/
     handlers.rs
     protocol.rs
     tests.rs         # Unit tests for protocol parsing
   ```

### Alternative Layout (if Pubky becomes complex)

```
src/
  main.rs
  server/             # HTTP and routing
    mod.rs
    handlers.rs
  protocol/           # MCP JSON-RPC
    mod.rs
    types.rs
    dispatch.rs
  domain/             # Business logic
    registry.rs
    matcher.rs
    types.rs
  adapters/           # External integrations
    pubky.rs
    local.rs
  error.rs
```

**When to use:** If project grows beyond MVP and follows hexagonal architecture

**For MVP:** Stick with proposed layout (simpler, less indirection)

---

## Error Handling Strategy

### Layer-specific error handling

```rust
// Domain errors (thiserror for structure)
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Source not found: {0}")]
    NotFound(String),

    #[error("Invalid query: {0}")]
    InvalidQuery(String),
}

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("Pubky fetch failed: {0}")]
    PubkyFetch(String),

    #[error("JSON parse failed: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// Top-level app error (enum of domain errors)
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Registry error: {0}")]
    Registry(#[from] RegistryError),

    #[error("Load error: {0}")]
    Load(#[from] LoadError),

    #[error("Protocol error: {0}")]
    Protocol(String),
}
```

### Mapping to JSON-RPC errors

```rust
impl AppError {
    pub fn to_jsonrpc_error_code(&self) -> i32 {
        match self {
            AppError::Registry(RegistryError::InvalidQuery(_)) => -32602,
            AppError::Registry(_) => -32603,
            AppError::Load(_) => -32603,
            AppError::Protocol(_) => -32600,
        }
    }

    pub fn to_jsonrpc_response(&self, id: serde_json::Value) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: self.to_jsonrpc_error_code(),
                message: self.to_string(),
                data: None,
            }),
        }
    }
}
```

### Error flow

```
Domain error occurs
  ↓
Converted to AppError (via From trait)
  ↓
Handler catches Result<T, AppError>
  ↓
Maps to JsonRpcError with code + message
  ↓
Wrapped in JsonRpcResponse
  ↓
Serialized to JSON by axum
```

**Key principle:** Errors bubble up as strongly-typed enums, converted to JSON-RPC format only at protocol boundary

---

## Patterns to Follow

### Pattern 1: Stateless Handlers with Shared State

**What:** All handlers are stateless functions that receive AppState as parameter

**When:** Always in axum web services

**Why:**
- Handlers are easy to test (pure functions)
- State is explicitly threaded through (no hidden globals)
- axum handles Arc cloning automatically

**Example:**
```rust
async fn handle_tools_call(
    params: ToolCallParams,
    state: &AppState,
) -> Result<ToolCallResult, AppError> {
    // Access shared state
    let results = state.registry.search(&params.arguments.query).await?;

    // Pure transformation
    Ok(format_results(results))
}
```

---

### Pattern 2: Trait-Based Abstraction for Data Sources

**What:** Define behavior as trait, implement for different backends

**When:** Multiple data sources with same interface (Pubky vs Local)

**Why:**
- Easy to swap implementations (config-driven)
- Testable (mock implementations)
- No runtime conditionals in business logic

**Example:**
```rust
// Define trait
#[async_trait]
pub trait RegistryLoader: Send + Sync {
    async fn load(&self) -> Result<Vec<Source>, LoadError>;
}

// Factory function
pub fn create_loader(config: &Config) -> Box<dyn RegistryLoader> {
    if config.pubky_enabled {
        Box::new(PubkyLoader::new(config))
    } else {
        Box::new(LocalLoader::new(&config.fallback_path))
    }
}

// Usage (business logic doesn't care which)
let loader = create_loader(&config);
let registry = Registry::load_from(loader).await?;
```

---

### Pattern 3: Newtype Wrappers for Domain Types

**What:** Wrap primitive types in domain-specific newtypes

**When:** IDs, URLs, queries have validation or special behavior

**Why:**
- Type safety (can't mix up SourceId vs UserId)
- Centralized validation
- Self-documenting code

**Example:**
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceId(String);

impl SourceId {
    pub fn new(s: String) -> Result<Self, AppError> {
        if s.is_empty() {
            return Err(AppError::Protocol("SourceId cannot be empty".into()));
        }
        Ok(SourceId(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

---

### Pattern 4: Builder Pattern for Complex Responses

**What:** Use builder for constructing MCP responses with many optional fields

**When:** Creating tool responses with content arrays

**Example:**
```rust
pub struct ToolResponseBuilder {
    content: Vec<ContentItem>,
}

impl ToolResponseBuilder {
    pub fn new() -> Self {
        Self { content: Vec::new() }
    }

    pub fn add_text(mut self, text: String) -> Self {
        self.content.push(ContentItem::Text { text });
        self
    }

    pub fn add_image(mut self, url: String, alt: String) -> Self {
        self.content.push(ContentItem::Image { url, alt });
        self
    }

    pub fn build(self) -> ToolCallResult {
        ToolCallResult {
            content: self.content,
        }
    }
}

// Usage
let response = ToolResponseBuilder::new()
    .add_text("Found 3 sources:".into())
    .add_text(format_source(&source1))
    .add_text(format_source(&source2))
    .add_text(format_source(&source3))
    .build();
```

---

## Anti-Patterns to Avoid

### Anti-Pattern 1: Global Mutable State

**What goes wrong:** Using `lazy_static!` or `OnceCell` for mutable global registry

**Why it happens:** Seems easier than threading state through functions

**Consequences:**
- Hard to test (tests interfere with each other)
- Unclear ownership and lifetimes
- Difficult to mock or inject dependencies

**Instead:** Use axum's State extractor with Arc-wrapped state

**Detection:**
- `static mut REGISTRY: ...`
- `lazy_static! { static ref REGISTRY: Mutex<...> }`

---

### Anti-Pattern 2: Stringly-Typed APIs

**What goes wrong:** Using plain `String` for typed values (IDs, method names)

**Why bad:**
- No compile-time validation
- Easy to mix up parameters
- Runtime errors instead of compile errors

**Instead:** Use enums for known values, newtypes for IDs

**Example:**
```rust
// BAD
fn dispatch_method(method: &str) -> Result<Response> {
    match method {
        "tools/call" => ...,
        "tools/list" => ...,
        _ => Err("unknown method"),
    }
}

// GOOD
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum MpcMethod {
    Initialize,
    #[serde(rename = "tools/list")]
    ToolsList,
    #[serde(rename = "tools/call")]
    ToolsCall,
}

fn dispatch_method(method: MpcMethod) -> Result<Response> {
    match method {
        MpcMethod::ToolsCall => ...,
        MpcMethod::ToolsList => ...,
        MpcMethod::Initialize => ...,
    }
}
```

---

### Anti-Pattern 3: Blocking I/O in Async Handlers

**What goes wrong:** Using `std::fs::read` or blocking Pubky calls in async functions

**Why bad:**
- Blocks tokio worker thread
- Degrades server throughput
- Can cause deadlocks under load

**Instead:** Use `tokio::fs` or `spawn_blocking` for blocking operations

**Example:**
```rust
// BAD
async fn load_local(path: &Path) -> Result<Vec<Source>> {
    let data = std::fs::read_to_string(path)?; // BLOCKS!
    Ok(serde_json::from_str(&data)?)
}

// GOOD
async fn load_local(path: &Path) -> Result<Vec<Source>> {
    let data = tokio::fs::read_to_string(path).await?;
    Ok(serde_json::from_str(&data)?)
}

// Also GOOD (for CPU-heavy parsing)
async fn load_local(path: &Path) -> Result<Vec<Source>> {
    let path = path.to_owned();
    tokio::task::spawn_blocking(move || {
        let data = std::fs::read_to_string(&path)?;
        serde_json::from_str(&data)
    }).await?
}
```

---

### Anti-Pattern 4: Catch-All Error Handling

**What goes wrong:** Converting all errors to generic "Internal Error"

**Why bad:**
- Loses debugging information
- Client can't distinguish error types
- Makes troubleshooting harder

**Instead:** Map errors to specific JSON-RPC codes with details

**Example:**
```rust
// BAD
async fn handle_request(req: Request) -> Response {
    match process(req).await {
        Ok(result) => Response::success(result),
        Err(_) => Response::error(-32603, "Internal error"),
    }
}

// GOOD
async fn handle_request(req: Request) -> Response {
    match process(req).await {
        Ok(result) => Response::success(result),
        Err(e) => {
            let (code, message) = e.to_jsonrpc_error();
            Response::error(code, message)
                .with_data(json!({ "details": e.to_string() }))
        }
    }
}
```

---

### Anti-Pattern 5: Premature Abstraction

**What goes wrong:** Creating abstract traits for single implementations

**Why it happens:** Anticipating future requirements that may never come

**Consequences:**
- Extra complexity with no benefit
- Harder to understand and modify
- Indirection that slows development

**Instead:** Start concrete, refactor to trait when second implementation emerges

**Detection:**
- Trait with only one `impl` in codebase
- Generic parameters used only with one type
- "Future-proofing" comments

**Example:**
```rust
// BAD (premature)
trait Scorer: Send + Sync {
    fn score(&self, query: &str, source: &Source) -> f32;
}

struct LevenshteinScorer;
impl Scorer for LevenshteinScorer { ... }

fn search(scorer: &dyn Scorer) -> Vec<Source> { ... }

// GOOD (concrete until needed)
fn score_match(query: &str, source: &Source) -> f32 {
    // Direct implementation
}

fn search(query: &str) -> Vec<Source> {
    sources.iter()
        .map(|s| (s, score_match(query, s)))
        .collect()
}
```

**Exception:** RegistryLoader trait is NOT premature because we know we have two implementations (Pubky + Local) from day one.

---

## Build Order and Dependencies

### Phase 1: Foundation (No Dependencies)

**Components:**
- Error types (error.rs)
- Domain types (registry/types.rs: Source, MatchResult)
- Config struct (config.rs or in main.rs)

**Rationale:** These are dependencies for all other components

**Testing:** Unit tests for type validation, error conversion

---

### Phase 2: Data Layer (Depends on Phase 1)

**Components:**
- RegistryLoader trait (registry/loader.rs)
- LocalLoader implementation (reads JSON)
- Registry struct with load_from method

**Skip:** PubkyLoader (defer to later phase)

**Rationale:**
- Can develop and test with local JSON
- Pubky integration is independent concern
- Unblocks business logic development

**Testing:**
- LocalLoader with fixture JSON files
- Registry construction and access

---

### Phase 3: Business Logic (Depends on Phase 1, 2)

**Components:**
- Query matching pipeline (registry/matcher.rs)
- Fuzzy search implementation
- Scoring and ranking logic

**Rationale:**
- Core value of the system
- Can test thoroughly with local data
- Independent of protocol details

**Testing:**
- Unit tests with various query patterns
- Edge cases (empty query, no matches, exact matches)
- Performance tests (large registry)

---

### Phase 4: Protocol Layer (Depends on Phase 1)

**Components:**
- JSON-RPC types (mcp/protocol.rs)
- Request/response serialization
- Method dispatch skeleton

**Rationale:**
- Independent of business logic (can stub)
- Enables protocol testing
- Defines handler signatures

**Testing:**
- JSON-RPC parsing (valid and invalid)
- Error response formatting
- Method dispatch routing

---

### Phase 5: HTTP Integration (Depends on Phase 2, 3, 4)

**Components:**
- axum router setup (main.rs)
- AppState construction
- Handler wiring
- Startup sequence

**Rationale:**
- Brings all layers together
- End-to-end functionality
- Can test full request flow

**Testing:**
- Integration tests with real HTTP requests
- End-to-end query flow
- Error handling paths

---

### Phase 6: Pubky Integration (Depends on Phase 2)

**Components:**
- Pubky SDK wrapper (pubky/client.rs)
- PubkyLoader implementation
- Identity management (pubky/identity.rs)
- Config-based loader selection

**Rationale:**
- Can be developed in parallel with Phase 3-5
- Plugs into existing RegistryLoader trait
- Non-breaking addition

**Testing:**
- Integration tests against Pubky testnet
- Fallback behavior (Pubky → Local on failure)

---

### Dependency Graph

```
Phase 1: Foundation
         ↓
    ┌────┴────┐
    ↓         ↓
Phase 2:   Phase 4:
Data       Protocol
    ↓         ↓
    └────┬────┘
         ↓
      Phase 3:
    Business Logic
         ↓
      Phase 5:
   HTTP Integration
         ↓
      Phase 6:
Pubky Integration
```

**Critical path:** 1 → 2 → 3 → 5 (can ship without Pubky)

**Parallel development:** Phase 4 can happen alongside Phase 2

**Optional:** Phase 6 adds Pubky but doesn't block MVP

---

## Scalability Considerations

### At 100 users (MVP)

| Concern | Approach |
|---------|----------|
| Request throughput | Single-process axum server, ~10K req/s |
| Registry size | In-memory Vec, <1MB, linear search fine |
| Startup time | <100ms for local JSON load |
| Memory | ~50MB process size |
| Concurrent requests | Tokio handles 100s of concurrent connections |

**No optimizations needed**

---

### At 10K users

| Concern | Approach |
|---------|----------|
| Request throughput | Add horizontal scaling (multiple instances behind load balancer) |
| Registry size | If >10K sources, add HashMap index by category |
| Fuzzy search | Consider pre-computed n-gram index |
| Memory | Still fits in memory (~500MB for large registry) |
| Concurrent requests | Tokio still handles this, may need rate limiting |

**Optimizations:**
- Add category-based index for faster filtering
- Cache top queries (LRU cache of query → results)
- Add observability (metrics, tracing)

---

### At 1M users

| Concern | Approach |
|---------|----------|
| Request throughput | Autoscaling pod cluster, CDN for static responses |
| Registry size | If >1M sources, consider external search (ElasticSearch/Meilisearch) |
| Fuzzy search | Replace with dedicated search engine |
| Memory | Shared cache layer (Redis) for hot queries |
| Concurrent requests | Rate limiting per user, priority queues |

**Architecture changes:**
- Registry becomes external service (not in-memory)
- Add caching layer (Redis)
- Add message queue for async processing
- Split into microservices (API gateway → query service → registry service)

**For MVP:** Ignore all of this. Start simple, scale when needed.

---

## Pubky-Specific Considerations

### Pubky Client Lifecycle

**Question:** When to initialize Pubky client?

**Answer:** At startup, before creating AppState

**Pattern:**
```rust
#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::load()?;

    // Initialize Pubky client if enabled
    let pubky_client = if config.pubky_enabled {
        Some(PubkyClient::new(&config.homeserver_url).await?)
    } else {
        None
    };

    // Create loader based on availability
    let loader: Box<dyn RegistryLoader> = match pubky_client {
        Some(client) => Box::new(PubkyLoader::new(client)),
        None => Box::new(LocalLoader::new(&config.fallback_path)),
    };

    let registry = Registry::load_from(loader).await?;

    let app_state = AppState {
        registry: Arc::new(registry),
        config: Arc::new(config),
    };

    // ... start server
}
```

---

### Pubky Fallback Strategy

**Requirement:** If Pubky fetch fails, fall back to local JSON

**Implementation:**
```rust
async fn load_registry(config: &Config) -> Result<Registry> {
    // Try Pubky first if enabled
    if config.pubky_enabled {
        match try_load_from_pubky(config).await {
            Ok(registry) => {
                info!("Loaded registry from Pubky");
                return Ok(registry);
            }
            Err(e) => {
                warn!("Pubky load failed: {}, falling back to local", e);
            }
        }
    }

    // Fallback to local
    let loader = LocalLoader::new(&config.fallback_path);
    let sources = loader.load().await?;
    Ok(Registry::new(sources))
}
```

**Key decision:** Fail open (serve stale local data) rather than fail closed (reject requests)

---

### Pubky Trust Score Integration

**Data flow:**
```
Pubky homeserver
    ↓ (fetch)
Source metadata with trust signals
    ↓ (parse)
Calculate trust_score (0.0-1.0)
    ↓ (store)
Source struct with trust_score field
    ↓ (use)
Ranking tiebreaker in search results
```

**Trust score factors:**
- Pubky identity verification
- Endorsements from trusted sources
- Historical accuracy
- Community ratings

**Storage:** Store as `f32` field on `Source`, use in ranking

---

### Identity Management

**For MVP:** Server has single identity (operator's key)

**Pattern:**
```rust
pub struct PubkyIdentity {
    keypair: Keypair,
    public_key: String,
}

impl PubkyIdentity {
    pub fn load_from_env() -> Result<Self> {
        let secret = env::var("PUBKY_SECRET_KEY")?;
        let keypair = Keypair::from_secret(&secret)?;
        Ok(PubkyIdentity {
            public_key: keypair.public_key().to_string(),
            keypair,
        })
    }

    pub fn sign(&self, data: &[u8]) -> Signature {
        self.keypair.sign(data)
    }
}
```

**Security:** Never log or expose secret key, load from env var only

---

## Testing Strategy

### Unit Tests

**What to test:**
- JSON-RPC parsing and serialization
- Query matching logic (scoring, ranking)
- Error conversions
- Domain type validation

**Location:** `#[cfg(test)] mod tests` in each module

**Example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_scoring() {
        let score = calculate_levenshtein("rust", "trust");
        assert!(score > 0.7);
    }

    #[tokio::test]
    async fn test_local_loader() {
        let loader = LocalLoader::new("fixtures/test_registry.json");
        let sources = loader.load().await.unwrap();
        assert_eq!(sources.len(), 3);
    }
}
```

---

### Integration Tests

**What to test:**
- Full HTTP request → JSON-RPC response flow
- Multiple requests to same server
- Error handling paths
- Startup and shutdown

**Location:** `tests/integration_test.rs`

**Pattern:**
```rust
#[tokio::test]
async fn test_tools_call_request() {
    let app = create_test_app().await;

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "search_sources",
            "arguments": { "query": "rust async" }
        }
    });

    let response = app
        .oneshot(
            Request::builder()
                .uri("/mcp")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body: JsonRpcResponse = serde_json::from_slice(
        &hyper::body::to_bytes(response.into_body()).await.unwrap()
    ).unwrap();

    assert_eq!(body.id, json!(1));
    assert!(body.result.is_some());
}
```

---

### Property-Based Tests (with proptest)

**What to test:**
- Fuzzy matching always returns scores in [0.0, 1.0]
- Top 3 results are always sorted by score descending
- All queries return valid JSON-RPC responses

**Example:**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_score_always_in_range(query in "\\PC{1,100}") {
        let source = Source::fixture();
        let score = calculate_match_score(&query, &source);
        prop_assert!(score >= 0.0 && score <= 1.0);
    }
}
```

---

## Open Questions and Research Flags

### Questions Requiring Phase-Specific Research

1. **Pubky SDK API surface:**
   - What's the exact API for fetching from homeserver?
   - How are signatures verified?
   - What's the error model?
   - **Flag for:** Phase 6 (Pubky Integration)

2. **MCP protocol versioning:**
   - How to handle protocol version negotiation?
   - Are there breaking changes between versions?
   - **Flag for:** Phase 4 (Protocol Layer)

3. **Fuzzy matching performance:**
   - At what registry size does linear search become too slow?
   - Should we use n-gram indexing from start?
   - **Flag for:** Phase 3 (Business Logic) - benchmark with realistic data

4. **Hot-reload requirements:**
   - Do we need runtime registry updates?
   - If yes, how often?
   - **Flag for:** Post-MVP enhancement

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| axum 0.8 patterns | HIGH | Standard patterns, well-documented |
| JSON-RPC structure | HIGH | Spec is clear, widely implemented |
| Trait abstraction | HIGH | Idiomatic Rust pattern |
| Module layout | HIGH | Follows Rust conventions |
| Error handling | HIGH | thiserror + anyhow is standard |
| Fuzzy matching | MEDIUM | Algorithm choice may need tuning |
| Pubky integration | LOW | Pubky SDK API not verified |
| Performance estimates | MEDIUM | Based on typical Rust web service benchmarks |

---

## Sources

**Note:** Due to tool restrictions, sources are based on Rust ecosystem knowledge as of January 2025. Key patterns verified through:

- axum documentation and examples (standard State pattern)
- Rust API guidelines (trait-based abstraction, error handling)
- JSON-RPC 2.0 specification (message formats)
- Common Rust web service architectures (layered, trait-based)

**Verification needed:**
- Pubky SDK documentation (client API, identity management)
- MCP protocol specification (exact message formats, versioning)
- axum 0.8 specific changes (if any from 0.7)

**Confidence level:** MEDIUM overall (HIGH for Rust patterns, LOW for Pubky specifics)
