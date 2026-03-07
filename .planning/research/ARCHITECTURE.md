# Architecture Patterns: v2.0 Integration

**Project:** Three Good Sources (3GS) -- v2.0 Community Curation
**Researched:** 2026-03-07
**Confidence:** HIGH (based on direct analysis of all existing source files)

## Existing Architecture Summary

The current codebase (2,179 lines, 20 `.rs` files) follows a clean modular pattern:

```
main.rs          -- Config load, keypair init, registry load, state assembly, server start
server.rs        -- AppState struct, build_router(), route handlers (/, /mcp, /health, /registry)
lib.rs           -- Module declarations only (pub mod for each top-level module)
config.rs        -- Env-based config via envy (REGISTRY_PATH, PORT, LOG_FORMAT, PKARR_SECRET_KEY)
mcp/handler.rs   -- JSON-RPC dispatch (initialize, tools/list, tools/call)
mcp/tools.rs     -- Tool implementations (get_sources, list_categories, get_provenance, get_endorsements)
mcp/types.rs     -- JSON-RPC request/response types, CallToolParams, InitializeParams
registry/loader.rs -- Async file read + serde + validation
registry/types.rs  -- Registry, Category, Source, Curator, Endorsement structs
registry/mod.rs    -- Re-exports: RegistryError, load(), Registry
pubky/identity.rs  -- Keypair generation/loading from hex secret
pubky/mod.rs       -- Re-exports: error, identity
matcher/           -- Fuzzy matching engine (config, scorer, normalize)
```

**Established loading pattern:** JSON file -> `tokio::fs::read_to_string` -> `serde_json::from_str` -> `validate()` -> `Arc::new()` -> stored in `AppState`.

**Current AppState:**
```rust
pub struct AppState {
    pub mcp_handler: McpHandler,
    pub registry: Arc<Registry>,
    pub pubkey: PublicKey,  // PublicKey is Copy, no Arc needed
}
```

**Current McpHandler fields:**
```rust
pub struct McpHandler {
    initialized: Arc<AtomicBool>,
    registry: Arc<Registry>,
    match_config: MatchConfig,
    pubkey_z32: String,
}
```

**Current tool dispatch signature (tools.rs):**
```rust
pub fn handle_tool_call(
    name: &str,
    arguments: Option<Value>,
    registry: &Registry,
    match_config: &MatchConfig,
    pubkey_z32: &str,
) -> Result<Value, ToolCallError>
```

**Current tools/list count:** 4 tools. v2.0 adds 3 more (7 total).

## Recommended Architecture for v2.0

### Design Principle: Parallel Modules, Not Nested

Each new data domain gets its own top-level module following the exact pattern established by `src/registry/`. This keeps concerns separated and avoids bloating the registry module with unrelated data.

### New Module Structure

```
src/
  audit/                    # NEW MODULE
    mod.rs                  -- pub use re-exports
    types.rs                -- AuditLog, AuditEntry, AuditAction enum
    loader.rs               -- load() + validate()
    error.rs                -- AuditError
  identity/                 # NEW MODULE (note: distinct from pubky/identity.rs)
    mod.rs                  -- pub use re-exports
    types.rs                -- IdentityRegistry, Identity, IdentityClaim, Platform enum
    loader.rs               -- load() + validate()
    error.rs                -- IdentityError
  contributions/            # NEW MODULE
    mod.rs                  -- pub use re-exports
    types.rs                -- ContributionRegistry, Proposal, ProposalStatus, Vote, VoterType
    loader.rs               -- load() + validate()
    error.rs                -- ContributionError
```

**Naming note:** `src/identity/` is about identity *linking* (PKARR pubkey to X/Nostr/GitHub handles). `src/pubky/identity.rs` is about PKARR *keypair management*. These are different concerns. The existing `pubky/identity.rs` stays as-is.

### Component Boundaries

| Component | Responsibility | Communicates With |
|-----------|---------------|-------------------|
| `src/audit/` | Load, validate, serve audit log entries | AppState (read), server routes |
| `src/identity/` | Load, validate, serve identity claims | AppState (read), server routes, MCP tools |
| `src/contributions/` | Load, validate, serve contribution proposals | AppState (read), server routes, MCP tools |
| `src/server.rs` | Extended AppState, new routes, router | All new modules via AppState |
| `src/mcp/tools.rs` | New tool implementations | All new data via McpHandler |
| `src/mcp/handler.rs` | Extended McpHandler construction | tools.rs |
| `src/config.rs` | New file path configs | main.rs startup |
| `src/main.rs` | Load new data files at startup | All new loaders |

### Extended AppState

```rust
pub struct AppState {
    pub mcp_handler: McpHandler,
    pub registry: Arc<Registry>,
    pub audit_log: Arc<AuditLog>,              // NEW
    pub identities: Arc<IdentityRegistry>,      // NEW
    pub contributions: Arc<ContributionRegistry>, // NEW
    pub pubkey: PublicKey,
}
```

### Extended McpHandler

```rust
pub struct McpHandler {
    initialized: Arc<AtomicBool>,
    registry: Arc<Registry>,
    audit_log: Arc<AuditLog>,              // NEW
    identities: Arc<IdentityRegistry>,      // NEW
    contributions: Arc<ContributionRegistry>, // NEW
    match_config: MatchConfig,
    pubkey_z32: String,
}
```

### Tool Dispatch Refactor

The current `handle_tool_call` takes 5 parameters. Adding 3 more data sources would push it to 8 parameters. Two options:

**Option A: Just add the parameters.** Simple, matches existing style, 8 params is manageable for an internal function.

**Option B: Introduce a context struct.**
```rust
pub struct ToolContext<'a> {
    pub registry: &'a Registry,
    pub audit_log: &'a AuditLog,
    pub identities: &'a IdentityRegistry,
    pub contributions: &'a ContributionRegistry,
    pub match_config: &'a MatchConfig,
    pub pubkey_z32: &'a str,
}
```

**Recommendation: Option A for now.** The codebase avoids premature abstraction (no traits with single impls, no unnecessary wrappers). Adding a struct for 8 parameters is reasonable but not necessary. If a v3.0 adds more data domains, refactor then.

However, the individual `tool_*` functions should only receive what they need. For example, `tool_get_identity` only needs `&IdentityRegistry`, not the full parameter list. This is already the established pattern -- `tool_get_sources` takes `registry` and `match_config` but not `pubkey_z32`.

## Data Flow

### Startup (main.rs) -- Extended

```
1. Config::load()  -- now includes audit_log_path, identities_path, contributions_path
2. init_logging()  -- unchanged
3. MatchConfig::load() + validate()  -- unchanged
4. generate_or_load_keypair()  -- unchanged
5. registry::load(&config.registry_path)  -- existing
6. audit::load(&config.audit_log_path)  -- NEW
7. identity::load(&config.identities_path)  -- NEW
8. contributions::load(&config.contributions_path)  -- NEW
9. McpHandler::new(registry, audit_log, identities, contributions, match_config, pubkey_z32)
10. AppState { mcp_handler, registry, audit_log, identities, contributions, pubkey }
11. build_router(app_state)
```

All loads are independent and could theoretically run concurrently with `tokio::try_join!`, but sequential is fine for 4 small JSON files at startup.

### Request Handling -- New Endpoints

```
GET /audit?action=add_source&since=2026-01-01
  -> State(state) -> state.audit_log -> filter entries -> JSON response

GET /identities
  -> State(state) -> state.identities -> serialize all -> JSON response

GET /identities/{pubkey}
  -> State(state) -> state.identities -> lookup by pubkey -> JSON or 404

GET /proposals
  -> State(state) -> state.contributions -> filter by ?status -> JSON response

GET /proposals/{id}
  -> State(state) -> state.contributions -> lookup by id -> JSON or 404

POST /mcp (tools/call: get_identity)
  -> McpHandler -> handle_tool_call -> tool_get_identity(identities) -> MCP response

POST /mcp (tools/call: list_proposals)
  -> McpHandler -> handle_tool_call -> tool_list_proposals(contributions) -> MCP response

POST /mcp (tools/call: get_proposal)
  -> McpHandler -> handle_tool_call -> tool_get_proposal(contributions) -> MCP response
```

## New Data File Schemas

### audit_log.json

```json
{
  "version": "1.0.0",
  "entries": [
    {
      "id": "audit-001",
      "timestamp": "2026-03-07T12:00:00Z",
      "action": "add_source",
      "category": "rust-learning",
      "details": "Added 'The Rust Programming Language' as source #1",
      "curator_pubkey": "pk:...",
      "signature": "sig:..."
    }
  ]
}
```

**Type definitions:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditLog {
    pub version: String,
    pub entries: Vec<AuditEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: String,  // ISO 8601 string, parsed for filtering
    pub action: AuditAction,
    pub category: Option<String>,  // Some actions are category-specific
    pub details: String,
    pub curator_pubkey: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    AddSource,
    RemoveSource,
    UpdateSource,
    AddCategory,
    RemoveCategory,
    UpdateCategory,
}
```

**Validation rules:**
- Entry IDs must be unique
- Timestamps must parse as valid ISO 8601 (validate with simple regex, no chrono dependency needed)
- Action must be a known enum variant (serde handles this)

### identities.json

```json
{
  "version": "1.0.0",
  "identities": [
    {
      "pubkey": "pk:...",
      "display_name": "John Turner",
      "claims": [
        {
          "platform": "x",
          "handle": "@johnturner",
          "proof_url": "https://x.com/johnturner/status/...",
          "verified_at": "2026-03-07T12:00:00Z"
        },
        {
          "platform": "github",
          "handle": "jturner",
          "proof_url": "https://gist.github.com/jturner/...",
          "verified_at": "2026-03-07T12:00:00Z"
        },
        {
          "platform": "nostr",
          "handle": "npub1...",
          "proof_url": null,
          "verified_at": "2026-03-07T12:00:00Z"
        }
      ]
    }
  ]
}
```

**Type definitions:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdentityRegistry {
    pub version: String,
    pub identities: Vec<Identity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    pub pubkey: String,
    pub display_name: String,
    pub claims: Vec<IdentityClaim>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdentityClaim {
    pub platform: Platform,
    pub handle: String,
    pub proof_url: Option<String>,
    pub verified_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    X,
    Github,
    Nostr,
}
```

**Validation rules:**
- Each identity must have a unique pubkey
- At least one claim per identity
- Platform is enum-constrained (serde handles this)

### contributions.json

```json
{
  "version": "1.0.0",
  "proposals": [
    {
      "id": "prop-001",
      "submitted_at": "2026-03-07T12:00:00Z",
      "proposer": {
        "pubkey": "pk:...",
        "display_name": "Alice"
      },
      "type": "new_category",
      "title": "Add Docker Security category",
      "description": "Three sources for hardening Docker containers",
      "status": "open",
      "votes": [
        {
          "voter_pubkey": "pk:...",
          "voter_type": "human",
          "vote": "approve",
          "comment": "Great sources, well researched",
          "voted_at": "2026-03-07T14:00:00Z"
        }
      ],
      "resolution": null,
      "resolved_at": null
    }
  ]
}
```

**Type definitions:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContributionRegistry {
    pub version: String,
    pub proposals: Vec<Proposal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Proposal {
    pub id: String,
    pub submitted_at: String,
    pub proposer: Proposer,
    #[serde(rename = "type")]
    pub proposal_type: ProposalType,
    pub title: String,
    pub description: String,
    pub status: ProposalStatus,
    pub votes: Vec<Vote>,
    pub resolution: Option<String>,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Proposer {
    pub pubkey: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Vote {
    pub voter_pubkey: String,
    pub voter_type: VoterType,
    pub vote: VoteChoice,
    pub comment: Option<String>,
    pub voted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalType {
    NewCategory,
    NewSource,
    UpdateSource,
    RemoveSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProposalStatus {
    Open,
    Accepted,
    Rejected,
    Withdrawn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VoterType {
    Human,
    Bot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VoteChoice {
    Approve,
    Reject,
}
```

**Validation rules:**
- Each proposal must have a unique ID
- Status/type/vote enums are serde-constrained
- VoterType separation enables human vs bot signal analysis

## New Routes

### REST Endpoints

| Route | Method | Handler | Query Params | Returns |
|-------|--------|---------|-------------|---------|
| `/audit` | GET | `audit_endpoint` | `?action=X`, `?since=YYYY-MM-DD`, `?category=X` | Filtered audit entries |
| `/identities` | GET | `identities_endpoint` | none | All identities |
| `/identities/{pubkey}` | GET | `identity_by_pubkey_endpoint` | none | Single identity or 404 |
| `/proposals` | GET | `proposals_endpoint` | `?status=open` | Filtered proposals |
| `/proposals/{id}` | GET | `proposal_by_id_endpoint` | none | Single proposal or 404 |

**Router extension:**
```rust
Router::new()
    .route("/", get(landing_page_endpoint))
    .route("/mcp", post(mcp_endpoint))
    .route("/health", get(health_endpoint))
    .route("/registry", get(registry_endpoint))
    .route("/audit", get(audit_endpoint))                    // NEW
    .route("/identities", get(identities_endpoint))           // NEW
    .route("/identities/{pubkey}", get(identity_by_pubkey_endpoint))  // NEW
    .route("/proposals", get(proposals_endpoint))             // NEW
    .route("/proposals/{id}", get(proposal_by_id_endpoint))   // NEW
    .layer(cors)
    .with_state(state)
```

Note: axum 0.8 uses `{param}` syntax for path parameters (not `:param`).

### New MCP Tools (3 additions, 7 total)

| Tool | Parameters | Returns |
|------|-----------|---------|
| `get_identity` | `{ pubkey: string }` | Identity claims for a PKARR pubkey |
| `list_proposals` | `{ status?: string }` | All proposals, optionally filtered |
| `get_proposal` | `{ id: string }` | Single proposal details |

These follow the established pattern: `*Params` struct with `#[derive(Deserialize, JsonSchema)]` + `#[serde(deny_unknown_fields)]`, entry in `get_tools_list()` JSON, match arm in `handle_tool_call()`, `tool_*` implementation function.

## Patterns to Follow

### Pattern 1: Module Loader (replicate registry/loader.rs exactly)

Every new module must follow this established pattern:

```rust
// src/audit/loader.rs
use super::{AuditLog, AuditError};
use std::path::Path;
use tokio::fs;

pub async fn load(path: impl AsRef<Path>) -> Result<AuditLog, AuditError> {
    let path = path.as_ref();
    let path_str = path.display().to_string();

    tracing::info!(path = %path.display(), "Loading audit log");

    let contents = fs::read_to_string(path)
        .await
        .map_err(|e| AuditError::FileRead {
            path: path_str.clone(),
            error: e.to_string(),
        })?;

    let audit_log: AuditLog = serde_json::from_str(&contents)
        .map_err(|e| AuditError::JsonParse {
            path: path_str.clone(),
            error: e.to_string(),
            line: e.line(),
            column: e.column(),
        })?;

    validate(&audit_log)?;

    tracing::info!(
        version = %audit_log.version,
        entries = audit_log.entries.len(),
        "Audit log loaded successfully"
    );

    Ok(audit_log)
}
```

### Pattern 2: Error Enum (replicate registry/error.rs exactly)

```rust
// src/audit/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuditError {
    #[error("Failed to read audit log file at {path}: {error}")]
    FileRead { path: String, error: String },

    #[error("Failed to parse audit log JSON from {path} at line {line}, column {column}: {error}")]
    JsonParse {
        path: String,
        error: String,
        line: usize,
        column: usize,
    },

    #[error("Duplicate audit entry ID '{id}'")]
    DuplicateId { id: String },
}
```

### Pattern 3: Query Parameter Filtering (new pattern for project)

The project currently has no query parameter support. New endpoints introduce it via axum's `Query` extractor:

```rust
use axum::extract::Query;

#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    pub action: Option<String>,
    pub since: Option<String>,
    pub category: Option<String>,
}

async fn audit_endpoint(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AuditQuery>,
) -> Json<serde_json::Value> {
    let entries: Vec<_> = state.audit_log.entries.iter()
        .filter(|e| {
            params.action.as_ref().map_or(true, |a| {
                // Compare action string to enum variant
                format!("{:?}", e.action).to_lowercase().contains(&a.to_lowercase())
            })
        })
        .filter(|e| {
            params.since.as_ref().map_or(true, |s| e.timestamp.as_str() >= s.as_str())
        })
        .filter(|e| {
            params.category.as_ref().map_or(true, |c| {
                e.category.as_ref().map_or(false, |ec| ec == c)
            })
        })
        .collect();

    Json(json!({ "entries": entries, "count": entries.len() }))
}
```

### Pattern 4: Path Parameter Extraction (new pattern for project)

For `/identities/{pubkey}` and `/proposals/{id}`:

```rust
use axum::extract::Path;

async fn identity_by_pubkey_endpoint(
    State(state): State<Arc<AppState>>,
    Path(pubkey): Path<String>,
) -> (StatusCode, [(HeaderName, &'static str); 1], String) {
    match state.identities.identities.iter().find(|i| i.pubkey == pubkey) {
        Some(identity) => {
            let json = serde_json::to_string_pretty(identity).unwrap();
            (StatusCode::OK, [(header::CONTENT_TYPE, "application/json")], json)
        }
        None => (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "application/json")],
            format!(r#"{{"error":"Identity not found for pubkey: {}"}}"#, pubkey),
        ),
    }
}
```

### Pattern 5: Config Extension (env vars via envy)

The project uses `envy` to map env vars to struct fields. Field names map to env var names via SCREAMING_SNAKE_CASE automatically:

```rust
pub struct Config {
    pub registry_path: PathBuf,           // REGISTRY_PATH (existing)
    pub audit_log_path: PathBuf,          // AUDIT_LOG_PATH (new)
    pub identities_path: PathBuf,         // IDENTITIES_PATH (new)
    pub contributions_path: PathBuf,      // CONTRIBUTIONS_PATH (new)
    #[serde(default = "default_log_format")]
    pub log_format: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub pkarr_secret_key: Option<String>,
}
```

These are required fields (not Option), so the server will fail to start if they are missing, matching the existing behavior for `registry_path`.

## Anti-Patterns to Avoid

### Anti-Pattern 1: Nesting New Data Inside Registry

**What:** Adding audit_log, identities, contributions fields to `Registry` struct or `registry.json`.
**Why bad:** Violates single responsibility. Registry has `#[serde(deny_unknown_fields)]` which would reject unknown fields anyway. Makes independent loading/validation impossible. Breaks all existing tests.
**Instead:** Separate modules, separate files, separate types. They share `AppState` but nothing else.

### Anti-Pattern 2: Dynamic File Reloading

**What:** Adding file watchers or hot-reload for JSON files.
**Why bad:** Premature complexity. Existing pattern is load-once-on-startup. All data is curator-managed and committed to repo. Changes deploy via Docker rebuild. The read-only server philosophy is explicit in PROJECT.md.
**Instead:** Keep load-on-startup. Redeploy to update data.

### Anti-Pattern 3: Runtime Signing

**What:** Having the server sign audit entries at runtime.
**Why bad:** The server is read-only. No write API in v2.0. Signing at runtime means the server needs write access to data files.
**Instead:** Signatures are pre-computed offline by the curator and stored in the JSON files. The server loads and serves pre-signed data.

### Anti-Pattern 4: Adding chrono Dependency for Date Filtering

**What:** Adding the `chrono` crate just to parse ISO 8601 strings for the `?since=` filter.
**Why bad:** Heavy dependency for simple string comparison. ISO 8601 strings in `YYYY-MM-DDTHH:MM:SSZ` format sort lexicographically.
**Instead:** Use string comparison (`timestamp >= since_str`) for filtering. Validate format with a regex at load time. Only add chrono if actual date math is needed later.

## Signing Integration with Existing PKARR

The existing PKARR keypair serves two purposes today:
1. **Identity display** -- pubkey shown in `/health` and `get_provenance` tool
2. **Keypair storage** -- loaded from `PKARR_SECRET_KEY` env var or generated ephemerally

For v2.0, the `signature` field in audit entries is **pre-computed offline by the curator**. The server does not sign at runtime.

**Flow:**
1. Curator edits data files locally
2. Curator signs entries using their PKARR keypair (offline CLI tool, separate from server)
3. Signed JSON files committed to repo
4. Server loads and serves pre-signed data
5. Clients verify signatures using pubkey from `/health` or `get_provenance`

The identity module links the same PKARR pubkey to social platform handles. The curator's identity entry in `identities.json` will reference the same pubkey that appears in `registry.json`'s `curator.pubkey` field and in `/health`'s response.

## Build Order (Dependency-Aware)

The three new features have minimal inter-dependencies:

```
audit (standalone) ----\
                        +---> All share AppState, loaded independently at startup
identity (standalone) -/
                        \
contributions (references proposer pubkey, but no hard code dependency)
```

### Phase 1: Audit Log Module
- **Why first:** Simplest schema (flat list of entries). No new MCP tools (REST-only per project spec). Establishes the "new module" pattern that identity and contributions will copy.
- **Creates:** `src/audit/` (4 files), `audit_log.json`, test fixtures
- **Modifies:** `main.rs` (add load), `lib.rs` (add mod), `config.rs` (add path), `server.rs` (add AppState field + route + handler)
- **Does not modify:** `mcp/` (no new tools for audit)
- **Tests:** Loader unit tests, route integration test, filter tests

### Phase 2: Identity Module
- **Why second:** Introduces path parameter routes (`/identities/{pubkey}`) and the first new MCP tool (`get_identity`). This pattern is then reused by contributions.
- **Creates:** `src/identity/` (4 files), `identities.json`, test fixtures
- **Modifies:** `main.rs`, `lib.rs`, `config.rs`, `server.rs` (AppState + 2 routes), `mcp/handler.rs` (McpHandler fields), `mcp/tools.rs` (1 new tool + tools_list update)
- **Tests:** Loader tests, route tests, MCP tool integration test

### Phase 3: Contributions Module
- **Why third:** Most complex schema (proposals with votes, status, resolution). Adds two MCP tools. Reuses all patterns established in phases 1-2.
- **Creates:** `src/contributions/` (4 files), `contributions.json`, test fixtures
- **Modifies:** `main.rs`, `lib.rs`, `config.rs`, `server.rs` (AppState + 2 routes), `mcp/handler.rs`, `mcp/tools.rs` (2 new tools + tools_list update)
- **Tests:** Loader tests, route tests, vote type filtering tests, 2 MCP tool integration tests

### Phase 4: Integration and Polish
- Update `get_provenance` tool to mention identity linking
- Update landing page HTML to document new endpoints
- Update `.env.example` and Dockerfile with new env vars
- Update `test_tools_list_returns_four_tools` to assert 7 tools
- End-to-end integration test across all new endpoints

## Files Modified vs Created

### Modified (existing files)

| File | Changes |
|------|---------|
| `src/main.rs` | 3 new module declarations, 3 new load calls, extended AppState and McpHandler construction |
| `src/lib.rs` | Add `pub mod audit; pub mod identity; pub mod contributions;` |
| `src/config.rs` | Add 3 new `PathBuf` fields |
| `src/server.rs` | Extend AppState struct (3 new Arc fields), add 5 new routes to `build_router()`, add 5 new handler functions |
| `src/mcp/handler.rs` | Extend McpHandler struct (3 new Arc fields), extend `McpHandler::new()` signature, pass new data to `handle_tool_call()` |
| `src/mcp/tools.rs` | Add 3 new param structs, extend `get_tools_list()` (4 -> 7 tools), add 3 match arms in `handle_tool_call()`, add 3 `tool_*` functions |
| `.env` / `.env.example` | Add `AUDIT_LOG_PATH`, `IDENTITIES_PATH`, `CONTRIBUTIONS_PATH` |
| `Dockerfile` | COPY new JSON files, set new ENV vars |
| `tests/integration_mcp.rs` | Update tool count assertion from 4 to 7 |

### Created (new files)

| File | Purpose |
|------|---------|
| `src/audit/mod.rs` | Module re-exports |
| `src/audit/types.rs` | AuditLog, AuditEntry, AuditAction |
| `src/audit/loader.rs` | Load + validate audit_log.json |
| `src/audit/error.rs` | AuditError enum |
| `src/identity/mod.rs` | Module re-exports |
| `src/identity/types.rs` | IdentityRegistry, Identity, IdentityClaim, Platform |
| `src/identity/loader.rs` | Load + validate identities.json |
| `src/identity/error.rs` | IdentityError enum |
| `src/contributions/mod.rs` | Module re-exports |
| `src/contributions/types.rs` | ContributionRegistry, Proposal, Vote, ProposalStatus, VoterType |
| `src/contributions/loader.rs` | Load + validate contributions.json |
| `src/contributions/error.rs` | ContributionError enum |
| `audit_log.json` | Seed audit data (retroactive entries for v1.0 source additions) |
| `identities.json` | Seed identity data (curator's own identity with claims) |
| `contributions.json` | Empty contributions (`{"version":"1.0.0","proposals":[]}`) |
| `tests/fixtures/valid_audit_log.json` | Test fixture |
| `tests/fixtures/valid_identities.json` | Test fixture |
| `tests/fixtures/valid_contributions.json` | Test fixture |
| `tests/integration_audit.rs` | Integration tests for /audit |
| `tests/integration_identity.rs` | Integration tests for /identities |
| `tests/integration_contributions.rs` | Integration tests for /proposals + MCP tools |

## Estimated Size Impact

| Area | Current | After v2.0 | Notes |
|------|---------|------------|-------|
| Rust source files | 20 | 32 | +12 new files in 3 modules |
| Lines of Rust | ~2,179 | ~3,200-3,500 | ~1,000-1,300 new lines |
| JSON data files | 1 | 4 | +3 new data files |
| Test files | 4 integration | 7 integration | +3 new test files |
| MCP tools | 4 | 7 | +3 new tools |
| HTTP routes | 4 | 9 | +5 new routes |

## Scalability Considerations

| Concern | At current scale | At 1K entries | Notes |
|---------|-----------------|---------------|-------|
| Audit log size | <1KB (seed data) | ~100KB | Linear scan fine, string comparison for filtering |
| Identity count | 1 (curator) | ~50 | Linear scan fine, Vec::iter().find() adequate |
| Proposal count | 0 | ~100 | Linear scan fine |
| Startup time | ~10ms | ~15ms | Negligible additional load time |

Growth is bounded by human curation speed. In-memory with no pagination is appropriate for the foreseeable project lifetime.

## Sources

- Direct analysis of all 20 existing `.rs` source files (HIGH confidence)
- axum 0.8 `{param}` path syntax and `Query`/`Path` extractors verified from existing codebase usage of axum 0.8 (HIGH confidence)
- Project conventions: `deny_unknown_fields`, thiserror enums, async file loading, Arc wrapping, env-based config via envy (HIGH confidence, directly observed in code)
- PROJECT.md v2.0 feature requirements (HIGH confidence, explicit specification)
