# Phase 13: Identity Linking - Research

**Researched:** 2026-03-08
**Domain:** Identity module (Rust/serde types, JSON data file, REST endpoints, MCP tool)
**Confidence:** HIGH

## Summary

Phase 13 introduces a new `src/identity/` module that follows the exact same 4-file pattern as `src/audit/` (mod.rs, types.rs, loader.rs, error.rs). The identity module maps PKARR Ed25519 public keys (hex-encoded, 64 chars) to cross-platform handles with proof URLs. The data lives in a curator-managed `identities.json` file loaded at startup.

This phase is structurally identical to what Phase 12 did for audit logs. The server remains read-only. No new Cargo dependencies are needed -- all required crates (serde, schemars, thiserror, tokio, axum) are already in Cargo.toml. The implementation adds two REST endpoints (GET /identities, GET /identities/{pubkey}), one MCP tool (get_identity), and the data types/loader.

**Primary recommendation:** Replicate the audit module pattern exactly -- same file structure, same loader style, same error handling, same integration points. The codebase is highly consistent and this phase should maintain that consistency.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- identities.json structured as object keyed by pubkey (O(1) lookup, matches /identities/{pubkey} endpoint)
- Each identity has: name (display name), type ("human" or "bot"), platforms (array of claim objects)
- Platform claims as array of objects: { platform, handle, proof_url }
- Platform enum: x, nostr, github -- strict Rust enum, add more later as new variants
- Use #[serde(default)] for the identity struct (schema evolution, consistent with audit types)
- Bot identities include operator_pubkey field pointing to a human identity's pubkey in the same file
- operator_pubkey is Option<String> with #[serde(skip_serializing_if = "Option::is_none")] -- absent for humans, present for bots
- Loader validates at load time: bot's operator_pubkey must reference an existing human identity (fail-fast, consistent with audit loader)
- No bot identities in v2.0 seed data -- only John Turner's human identity. Bots added when actual bots exist
- X proof: tweet URL containing PKARR pubkey
- GitHub proof: Gist URL containing PKARR pubkey
- Nostr proof: NIP-05 verification URL
- Server stores proof_url as string only -- no URL validation at load time
- Use the server's existing PKARR pubkey (same key from registry.json curator field / PKARR_SECRET_KEY)
- Placeholder proof URLs and handles for now -- clearly marked as TODO, replace with real ones later
- John Turner identity only -- no bot identities in seed data
- Identity pubkey in identities.json should match the curator pubkey in registry.json for John Turner

### Claude's Discretion
- Exact loader implementation details beyond fail-fast validation
- Filter parameters for GET /identities endpoint (if any beyond returning all)
- MCP tool response text formatting
- Test fixture design for invalid identities
- Whether to add a created_at/registered_at timestamp field to identity objects

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| IDENT-01 | identities.json maps PKARR pubkeys to platform handles with human/bot type | Identity struct with type enum and platforms array; identities.json keyed by hex pubkey |
| IDENT-02 | Every platform claim includes a proof URL for independent verification | PlatformClaim struct with proof_url field; proof URLs stored as strings |
| IDENT-03 | Bot identities link to a human operator's pubkey | operator_pubkey: Option<String> field; loader validates reference exists |
| IDENT-04 | GET /identities endpoint returns all registered identities | New route in server.rs, returns serialized HashMap |
| IDENT-05 | GET /identities/{pubkey} returns single identity | Axum Path extractor, O(1) HashMap lookup, 404 on miss |
| IDENT-06 | get_identity MCP tool returns identity info for a given pubkey | New tool in tools.rs with GetIdentityParams, pubkey input param |
| IDENT-07 | Curator's own identity registered with real platform proofs | Seed identities.json with test key pubkey, placeholder URLs marked TODO |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde + serde_json | 1.0 | Serialize/deserialize identity types | Already in Cargo.toml |
| schemars | 1 | JSON Schema for MCP tool params | Already used for all MCP tools |
| thiserror | 2.0 | Structured error types | Same pattern as AuditError |
| tokio | 1.49 | Async file I/O for loader | Already used for audit loader |
| axum | 0.8 | REST endpoint routing + extractors | Already used for /audit, /registry |

### Supporting
No new dependencies needed. All required crates are already in Cargo.toml.

### Alternatives Considered
None -- the decisions are locked to follow existing codebase patterns.

## Architecture Patterns

### Recommended Project Structure
```
src/
├── identity/
│   ├── mod.rs          # pub mod + re-exports (same as audit/mod.rs)
│   ├── types.rs        # Identity, IdentityType, Platform, PlatformClaim structs
│   ├── loader.rs       # load() fn: read file, deserialize, validate bot references
│   └── error.rs        # IdentityError enum with thiserror
├── server.rs           # Add identities to AppState, add /identities routes
├── mcp/
│   ├── tools.rs        # Add get_identity tool (6th tool)
│   └── handler.rs      # Pass identities to McpHandler
├── main.rs             # Load identities.json at startup
├── config.rs           # Add identities_path config field
└── lib.rs              # Add pub mod identity
```

### Pattern 1: Module Structure (replicate from audit/)
**What:** 4-file module with mod.rs, types.rs, loader.rs, error.rs
**When to use:** Every new data domain in this project
**Example:**
```rust
// src/identity/mod.rs
pub mod error;
pub mod loader;
pub mod types;

pub use error::IdentityError;
pub use loader::load;
pub use types::{Identity, IdentityType, Platform, PlatformClaim};
```

### Pattern 2: Serde Types with Schema Evolution
**What:** #[serde(default)] on structs, #[serde(rename_all = "snake_case")] on enums
**When to use:** All data types loaded from curator-managed JSON
**Example:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Identity {
    pub name: String,
    #[serde(rename = "type")]
    pub identity_type: IdentityType,
    pub platforms: Vec<PlatformClaim>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_pubkey: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum IdentityType {
    Human,
    Bot,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    X,
    Nostr,
    Github,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformClaim {
    pub platform: Platform,
    pub handle: String,
    pub proof_url: String,
}
```

### Pattern 3: Fail-Fast Loader with Validation
**What:** Load JSON file, deserialize, validate invariants, return Result
**When to use:** All JSON data loaded at startup
**Example:**
```rust
pub async fn load(path: impl AsRef<Path>) -> Result<HashMap<String, Identity>, IdentityError> {
    let contents = fs::read_to_string(path).await.map_err(|e| IdentityError::FileRead { .. })?;
    let identities: HashMap<String, Identity> = serde_json::from_str(&contents).map_err(|e| IdentityError::JsonParse { .. })?;

    // Validate bot operator references
    for (pubkey, identity) in &identities {
        if identity.identity_type == IdentityType::Bot {
            if let Some(ref op_key) = identity.operator_pubkey {
                match identities.get(op_key) {
                    Some(op) if op.identity_type == IdentityType::Human => {},
                    _ => return Err(IdentityError::InvalidOperator { .. }),
                }
            } else {
                return Err(IdentityError::MissingOperator { .. });
            }
        }
    }

    Ok(identities)
}
```

### Pattern 4: AppState Flat Fields with Arc
**What:** Add identities as `Arc<HashMap<String, Identity>>` field to AppState
**When to use:** All shared immutable data loaded at startup
**Example:**
```rust
pub struct AppState {
    pub mcp_handler: McpHandler,
    pub registry: Arc<Registry>,
    pub pubkey: PublicKey,
    pub audit_log: Arc<Vec<AuditEntry>>,
    pub identities: Arc<HashMap<String, Identity>>,  // NEW
}
```

### Pattern 5: REST Endpoint with Path Extractor
**What:** GET /identities/{pubkey} uses axum Path<String> extractor
**When to use:** Single-resource lookup endpoints
**Example:**
```rust
async fn identity_by_pubkey(
    State(state): State<Arc<AppState>>,
    Path(pubkey): Path<String>,
) -> (StatusCode, [(axum::http::HeaderName, &'static str); 1], String) {
    match state.identities.get(&pubkey) {
        Some(identity) => { /* serialize and return */ },
        None => (StatusCode::NOT_FOUND, ..., r#"{"error":"Identity not found"}"#.to_string()),
    }
}
```

### Pattern 6: MCP Tool with Params
**What:** GetIdentityParams struct with pubkey field, schemars derive, deny_unknown_fields
**When to use:** All MCP tool parameter types
**Example:**
```rust
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetIdentityParams {
    /// PKARR public key (64-character hex string) to look up
    pub pubkey: String,
}
```

### Anti-Patterns to Avoid
- **Don't use `type` as a Rust field name:** It is a reserved keyword. Use `identity_type` in Rust with `#[serde(rename = "type")]` for JSON compatibility.
- **Don't validate proof URLs at load time:** Decision explicitly defers URL validation to future VERIFY-01 requirement.
- **Don't add bot identities to seed data:** No actual bots exist yet. Only John Turner's human identity.
- **Don't add new dependencies:** Everything needed is already in Cargo.toml.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON schema for MCP | Manual schema JSON | schemars derive | All 5 existing tools use this pattern |
| Error types | Manual Display/Error impl | thiserror derive | Consistent with AuditError |
| Deserialization | Manual JSON parsing | serde derives | Standard Rust ecosystem tool |
| Path extraction | Manual URL parsing | axum Path<T> extractor | Built-in, type-safe |

**Key insight:** This entire phase is a mechanical replication of established patterns in the codebase. No novel engineering is required.

## Common Pitfalls

### Pitfall 1: Rust `type` Keyword Conflict
**What goes wrong:** Using `type` as a struct field name causes a compile error
**Why it happens:** `type` is a reserved keyword in Rust
**How to avoid:** Name the field `identity_type` and use `#[serde(rename = "type")]`
**Warning signs:** Compile error "expected identifier, found keyword `type`"

### Pitfall 2: HashMap Key vs JSON Object Key Mismatch
**What goes wrong:** identities.json uses pubkey as object key, but the GET /identities response might lose the key
**Why it happens:** Serializing HashMap<String, Identity> includes the key, but the Identity struct itself does not contain the pubkey
**How to avoid:** Either include the pubkey in the serialized response explicitly, or serialize the whole HashMap (which naturally includes keys). For /identities/{pubkey}, consider returning a wrapper that includes the pubkey field.
**Warning signs:** API response missing the pubkey identifier

### Pitfall 3: Empty identities.json
**What goes wrong:** Server crashes on startup if identities.json is `{}`
**Why it happens:** Not handling empty HashMap as valid state
**How to avoid:** Empty `{}` should deserialize to an empty HashMap<String, Identity> successfully. Test this case explicitly.
**Warning signs:** CONTEXT.md explicitly calls out this case

### Pitfall 4: Forgetting to Update Tool Count
**What goes wrong:** Tests assert 5 tools but there are now 6
**Why it happens:** Existing test `test_tools_list_returns_five_tools` hardcodes count
**How to avoid:** Update the test assertion from 5 to 6, and update the tools list comment "5 tools" -> "6 tools"
**Warning signs:** Integration test failures on tool count

### Pitfall 5: Forgetting Integration Points
**What goes wrong:** Module compiles but server does not serve identity data
**Why it happens:** Missing wiring in main.rs, server.rs, handler.rs, config.rs, lib.rs
**How to avoid:** Checklist of all files that need modification (see Architecture Patterns)
**Warning signs:** Endpoints return 404, MCP tool not found

### Pitfall 6: Config Missing identities_path
**What goes wrong:** Server cannot find identities.json at startup
**Why it happens:** Config struct does not have the new path field, or .env file lacks the variable
**How to avoid:** Add `identities_path: PathBuf` to Config struct, add IDENTITIES_PATH to .env
**Warning signs:** "Failed to load configuration" error

## Code Examples

### identities.json Seed Data Structure
```json
{
  "197f6b23e16c8532c6abc838facd5ea789be0c76b2920334039bfa8b3d368d61": {
    "name": "John Turner",
    "type": "human",
    "platforms": [
      {
        "platform": "x",
        "handle": "TODO_x_handle",
        "proof_url": "https://x.com/TODO_handle/status/TODO_tweet_id"
      },
      {
        "platform": "github",
        "handle": "TODO_github_handle",
        "proof_url": "https://gist.github.com/TODO_handle/TODO_gist_id"
      },
      {
        "platform": "nostr",
        "handle": "TODO_nostr_name",
        "proof_url": "https://TODO_domain/.well-known/nostr.json?name=TODO"
      }
    ]
  }
}
```

Note: The pubkey `197f6b23...` is the actual test key actor from audit_log.json. The real PKARR_SECRET_KEY pubkey should be used. Check if the signing utility from Phase 12 outputs the pubkey or derive it from the secret key.

### Error Type
```rust
#[derive(Debug, Error)]
pub enum IdentityError {
    #[error("Failed to read identities file at {path}: {error}")]
    FileRead { path: String, error: String },

    #[error("Failed to parse identities JSON from {path}: {error}")]
    JsonParse { path: String, error: String },

    #[error("Bot identity {pubkey} references non-existent operator {operator_pubkey}")]
    InvalidOperator { pubkey: String, operator_pubkey: String },

    #[error("Bot identity {pubkey} missing required operator_pubkey field")]
    MissingOperator { pubkey: String },
}
```

### GET /identities Endpoint
```rust
async fn identities_endpoint(
    State(state): State<Arc<AppState>>,
) -> (StatusCode, [(axum::http::HeaderName, &'static str); 1], String) {
    match serde_json::to_string(&*state.identities) {
        Ok(json) => (StatusCode::OK, [(header::CONTENT_TYPE, "application/json")], json),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, [(header::CONTENT_TYPE, "application/json")],
            format!(r#"{{"error":"Failed to serialize identities: {}"}}"#, e)),
    }
}
```

### GET /identities/{pubkey} Endpoint
```rust
async fn identity_by_pubkey(
    State(state): State<Arc<AppState>>,
    Path(pubkey): Path<String>,
) -> (StatusCode, [(axum::http::HeaderName, &'static str); 1], String) {
    match state.identities.get(&pubkey) {
        Some(identity) => {
            match serde_json::to_string(identity) {
                Ok(json) => (StatusCode::OK, [(header::CONTENT_TYPE, "application/json")], json),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, ...),
            }
        }
        None => (StatusCode::NOT_FOUND, [(header::CONTENT_TYPE, "application/json")],
            r#"{"error":"Identity not found"}"#.to_string()),
    }
}
```

### MCP Tool Handler
```rust
fn tool_get_identity(
    arguments: Option<Value>,
    identities: &HashMap<String, Identity>,
) -> Result<Value, ToolCallError> {
    let params: GetIdentityParams = if let Some(args) = arguments {
        serde_json::from_value(args).map_err(|_| ToolCallError::InvalidParams)?
    } else {
        return Err(ToolCallError::InvalidParams);
    };

    match identities.get(&params.pubkey) {
        Some(identity) => {
            let mut text = format!("Identity: {}\nType: {:?}\nPubkey: {}\n\nPlatforms:\n",
                identity.name, identity.identity_type, params.pubkey);
            for claim in &identity.platforms {
                text.push_str(&format!("- {:?}: {} (proof: {})\n",
                    claim.platform, claim.handle, claim.proof_url));
            }
            if let Some(ref op) = identity.operator_pubkey {
                text.push_str(&format!("\nOperator: {}\n", op));
            }
            Ok(json!({"content": [{"type": "text", "text": text}], "isError": false}))
        }
        None => {
            Ok(json!({"content": [{"type": "text", "text": format!("No identity found for pubkey: {}", params.pubkey)}], "isError": true}))
        }
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Inline pubkey in registry.json | Separate identity module with cross-platform claims | Phase 13 (v2.0) | Enables community curation with verified identities |

No deprecated patterns to worry about -- this is new functionality following established project conventions.

## Open Questions

1. **Pubkey for seed identity**
   - What we know: The audit log uses `197f6b23e16c8532c6abc838facd5ea789be0c76b2920334039bfa8b3d368d61` as the actor key (derived from test secret `[42u8; 32]`). The registry.json has `pk:placeholder` for curator.
   - What's unclear: Should the seed identity use the test key (matching audit log) or the real PKARR_SECRET_KEY?
   - Recommendation: Use the test key for now (matching audit_log.json), with a TODO comment. Same approach as audit log entries which were "generated with test key; re-sign with PKARR_SECRET_KEY for production."

2. **GET /identities response format**
   - What we know: identities.json is keyed by pubkey as a JSON object
   - What's unclear: Should GET /identities return the raw object format (keyed by pubkey) or an array with pubkey included in each item?
   - Recommendation: Return the raw HashMap serialization (object keyed by pubkey) for consistency with the file format. The GET /identities/{pubkey} endpoint returns the single Identity object.

3. **Whether to add created_at/registered_at**
   - What we know: This is Claude's discretion per CONTEXT.md
   - Recommendation: Skip for now. No existing requirement needs it, and #[serde(default)] ensures it can be added later without breaking existing data.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test + tokio::test (async) |
| Config file | Cargo.toml [dev-dependencies] |
| Quick run command | `cargo test identity` |
| Full suite command | `cargo test` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| IDENT-01 | Identity struct deserializes with type + platforms | unit | `cargo test identity::types` | No - Wave 0 |
| IDENT-02 | PlatformClaim includes proof_url | unit | `cargo test identity::types::test_claim` | No - Wave 0 |
| IDENT-03 | Bot operator_pubkey validated at load time | unit | `cargo test identity::loader::test_bot` | No - Wave 0 |
| IDENT-04 | GET /identities returns all identities | integration | `cargo test integration_identity::test_get_identities` | No - Wave 0 |
| IDENT-05 | GET /identities/{pubkey} returns single identity | integration | `cargo test integration_identity::test_get_identity_by_pubkey` | No - Wave 0 |
| IDENT-06 | get_identity MCP tool returns identity data | unit | `cargo test mcp::handler::tests::test_get_identity` | No - Wave 0 |
| IDENT-07 | Seed identity matches curator pubkey | unit | `cargo test identity::loader::test_seed` | No - Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test identity`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before /gsd:verify-work

### Wave 0 Gaps
- [ ] `src/identity/types.rs` -- unit tests for Identity, IdentityType, Platform, PlatformClaim deserialization
- [ ] `src/identity/loader.rs` -- unit tests for load(), bot validation, empty file handling
- [ ] `tests/integration_identity.rs` -- integration tests for GET /identities and GET /identities/{pubkey}
- [ ] `src/mcp/handler.rs` -- add test for get_identity MCP tool call
- [ ] `identities.json` -- seed data file (test fixture and production)
- [ ] `tests/common/mod.rs` -- update spawn_test_server to load identities

## Sources

### Primary (HIGH confidence)
- Project source code: src/audit/ (types.rs, loader.rs, error.rs, mod.rs) -- exact pattern to replicate
- Project source code: src/server.rs -- AppState struct and route registration pattern
- Project source code: src/mcp/tools.rs -- tool params, tool list, handle_tool_call dispatch
- Project source code: src/mcp/handler.rs -- McpHandler constructor and tool dispatch wiring
- Project source code: src/main.rs -- startup sequence for loading JSON and building state
- Project source code: src/config.rs -- Config struct for environment-based path config
- Project source code: tests/common/mod.rs -- test server setup pattern

### Secondary (MEDIUM confidence)
- audit_log.json actor field -- confirms hex-encoded Ed25519 pubkey format (64 chars)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all libraries already in Cargo.toml, no new dependencies
- Architecture: HIGH -- exact replication of audit module pattern with verified code
- Pitfalls: HIGH -- identified from reading actual codebase (reserved keyword, wiring points, test counts)

**Research date:** 2026-03-08
**Valid until:** 2026-04-08 (stable -- internal project patterns, no external API dependencies)
