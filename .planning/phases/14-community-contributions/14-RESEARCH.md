# Phase 14: Community Contributions - Research

**Researched:** 2026-03-08
**Domain:** Rust module for community proposals with vote tracking, REST endpoints, MCP tools
**Confidence:** HIGH

## Summary

Phase 14 adds a `src/contributions/` module following the exact same 4-file pattern (mod.rs, types.rs, loader.rs, error.rs) used by `src/audit/` and `src/identity/`. The module loads `contributions.json` as a `HashMap<Uuid, Proposal>` with vote validation cross-referencing identities at load time. Two REST endpoints (GET /proposals, GET /proposals/{id}) and two MCP tools (list_proposals, get_proposal) expose the data.

This is a pure replication of established patterns. The identity module (Phase 13) is the closest analog: UUID-keyed HashMap in AppState, single-item lookup by path param, list endpoint with filtering. The audit module adds the filtering pattern (query params via axum `Query<T>` extractor). The contribution module combines both patterns.

**Primary recommendation:** Follow the identity module structure exactly, adding filtering from the audit pattern. The loader takes both a file path AND an `Arc<HashMap<String, Identity>>` for vote validation.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- contributions.json structured as object keyed by UUID v4 (O(1) lookup, consistent with identities.json pattern)
- Action field as enum: add_source, update_source, remove_source, add_category, update_category
- Action-specific details stored in a flexible `data` JSON object (matches audit entry pattern)
- Status lifecycle with restricted transitions only: pending->approved, pending->rejected, pending->withdrawn
- Loader validates no illegal status transitions exist at load time (fail-fast, consistent with other loaders)
- Each proposal tracks: id, action, status, category, proposer (pubkey), created_at, data, votes
- Individual vote records stored as array per proposal: [{ voter: pubkey, vote: support/oppose, timestamp }]
- Binary voting only: support and oppose (no abstain -- not voting is implicit abstention)
- Voter identity type (human/bot) determined by cross-referencing voter pubkey against identities.json at load time
- Unknown voters (pubkey not in identities.json) rejected at load time -- all voters must be registered identities
- Loader takes identities as parameter for cross-reference validation
- Ship with one demo proposal (add_source action, pending status) with 1-2 sample votes from John Turner's pubkey
- Empty contributions.json ({}) loads successfully as valid empty state
- GET /proposals: summary list filtered by status and category query params
- Summary includes: id, action, status, category, proposer, created_at (no votes)
- GET /proposals/{id}: full detail including vote array with individual records
- Default sort: newest first (by created_at)
- MCP tools list_proposals and get_proposal mirror REST endpoint data and filtering

### Claude's Discretion
- Exact proposal data field shapes per action type
- Demo proposal content (which category, which source to propose)
- MCP tool response text formatting
- Test fixture design for invalid proposals/votes
- Whether to include an updated_at field alongside created_at
- Error response format for invalid filter params (lenient vs strict -- prior pattern is lenient)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CONTRIB-01 | contributions.json holds proposals with defined status lifecycle (pending, approved, rejected, withdrawn) | Types module defines ProposalStatus enum; loader validates no illegal transitions at load time |
| CONTRIB-02 | Proposals support actions: add_source, update_source, remove_source, add_category, update_category | ProposalAction enum in types.rs; flexible `data` field per action |
| CONTRIB-03 | Human and bot votes tracked separately per proposal, classified by voter's identity type | Vote struct with voter pubkey; loader cross-references identities HashMap to classify and validate |
| CONTRIB-04 | GET /proposals endpoint returns proposals filterable by status and category | REST endpoint with ProposalFilterParams query extractor, shared filter function |
| CONTRIB-05 | GET /proposals/{id} endpoint returns single proposal with vote details | Path extractor with UUID, full Proposal serialization including votes |
| CONTRIB-06 | list_proposals and get_proposal MCP tools expose proposal data to agents | Two new MCP tool params structs, tool functions, match arms in handle_tool_call, tools/list updated to 8 |
</phase_requirements>

## Standard Stack

### Core (already in Cargo.toml)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde + serde_json | 1.0 | Serialize/deserialize contributions.json | Project standard |
| uuid | 1 (v4, serde) | Proposal ID keys | Same as audit entries |
| chrono | 0.4 (serde) | created_at timestamps | Same as audit entries |
| thiserror | 2.0 | ContributionError type | Same as IdentityError, AuditError |
| axum | 0.8 | REST endpoints with Query/Path extractors | Project web framework |
| schemars | 1 (derive) | MCP tool input schemas | Same as existing tools |
| tokio | 1.49 | Async file I/O in loader | Project runtime |

### No New Dependencies
All libraries needed are already in Cargo.toml. No new crate additions required.

## Architecture Patterns

### Module Structure (replicates identity module)
```
src/contributions/
    mod.rs          # pub use re-exports
    types.rs        # Proposal, ProposalAction, ProposalStatus, Vote, ProposalFilterParams
    loader.rs       # load(path, &HashMap<String, Identity>) -> Result<HashMap<Uuid, Proposal>>
    error.rs        # ContributionError with thiserror
```

### Pattern 1: Types with Serde Enums
**What:** ProposalStatus and ProposalAction as `#[serde(rename_all = "snake_case")]` enums, matching AuditAction pattern.
**When to use:** All enum types in this project.
**Example:**
```rust
// Matches AuditAction pattern from src/audit/types.rs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    Pending,
    Approved,
    Rejected,
    Withdrawn,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalAction {
    AddSource,
    UpdateSource,
    RemoveSource,
    AddCategory,
    UpdateCategory,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoteChoice {
    Support,
    Oppose,
}
```

### Pattern 2: Proposal Struct with #[serde(default)]
**What:** Main Proposal struct using `#[serde(default)]` for forward compatibility, NOT `deny_unknown_fields`.
**When to use:** All top-level data structs (project decision from v2.0).
**Example:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Proposal {
    pub action: ProposalAction,
    pub status: ProposalStatus,
    pub category: String,
    pub proposer: String,           // pubkey
    pub created_at: DateTime<Utc>,
    pub data: serde_json::Value,    // flexible per-action data
    pub votes: Vec<Vote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter: String,              // pubkey
    pub vote: VoteChoice,
    pub timestamp: DateTime<Utc>,
}
```

Note: The `id` field is NOT in the struct -- it is the HashMap key (same as identities pattern where pubkey is the key).

### Pattern 3: Loader with Cross-Reference Validation
**What:** Loader takes identities HashMap as parameter for vote validation, matching CONTEXT.md decision.
**When to use:** When loaded data references data from another module.
**Example:**
```rust
// Similar to identity loader but with cross-module reference
pub async fn load(
    path: impl AsRef<Path>,
    identities: &HashMap<String, Identity>,
) -> Result<HashMap<Uuid, Proposal>, ContributionError> {
    // 1. Read file
    // 2. Parse JSON into HashMap<Uuid, Proposal>
    // 3. Validate: all voter pubkeys exist in identities
    // 4. Validate: no illegal status (only valid enum values -- serde handles this)
    // 5. Return
}
```

### Pattern 4: Summary vs Detail Serialization
**What:** GET /proposals returns summaries (no votes), GET /proposals/{id} returns full detail with votes.
**When to use:** List vs detail endpoints.
**Implementation:** Create a `ProposalSummary` struct that omits votes, or serialize with `#[serde(skip)]` for the list endpoint. Simplest approach: build a separate summary struct populated from Proposal.

```rust
#[derive(Debug, Serialize)]
pub struct ProposalSummary {
    pub id: Uuid,
    pub action: ProposalAction,
    pub status: ProposalStatus,
    pub category: String,
    pub proposer: String,
    pub created_at: DateTime<Utc>,
}
```

### Pattern 5: Filter Params with Lenient Behavior
**What:** Query params for status and category filtering, matching audit's lenient pattern.
**When to use:** List endpoints with optional filters.
**Example:**
```rust
// Matches AuditFilterParams pattern
#[derive(Debug, Deserialize)]
pub struct ProposalFilterParams {
    pub status: Option<String>,
    pub category: Option<String>,
}
```
Lenient: invalid status values return empty results (not errors), matching Phase 12 decision.

### Pattern 6: AppState Flat Field
**What:** Add `proposals: Arc<HashMap<Uuid, Proposal>>` as flat field on AppState.
**When to use:** All shared data in this project (v2.0 decision).

### Pattern 7: MCP Tool Params with deny_unknown_fields
**What:** MCP tool param structs use `#[serde(deny_unknown_fields)]` (strict), unlike data structs.
**When to use:** All MCP tool input schemas.
**Example:**
```rust
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListProposalsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetProposalParams {
    /// Proposal UUID
    pub id: String,
}
```

### Anti-Patterns to Avoid
- **Nested module in identity or audit:** Contributions is a peer module, not a child. Create `src/contributions/` at the same level.
- **Storing vote classification in JSON:** Human/bot classification is computed at load time from identities, not stored in contributions.json.
- **Using deny_unknown_fields on data structs:** Project uses `#[serde(default)]` for schema evolution.
- **Validating status transitions at load time:** The CONTEXT says "validates no illegal status transitions exist" -- this means checking that the status field contains a valid enum value, which serde handles automatically. There are no transition logs to check; proposals have a single current status.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| UUID parsing | Manual string validation | `uuid::Uuid` with serde | Already in Cargo.toml, handles all edge cases |
| Timestamp handling | Custom parsing | `chrono::DateTime<Utc>` with serde | Already in Cargo.toml, ISO 8601 standard |
| Enum serialization | Match arms for string conversion | `#[serde(rename_all = "snake_case")]` | Consistent with AuditAction, IdentityType |
| JSON Schema for MCP | Manual schema JSON | `schemars::schema_for!()` | Matches all existing MCP tools |
| Error types | String errors | `thiserror` derive | Matches IdentityError, project standard |

**Key insight:** Every building block needed is already in the project. This phase is pattern replication, not innovation.

## Common Pitfalls

### Pitfall 1: Forgetting to Sort Proposals by created_at (Newest First)
**What goes wrong:** List endpoint returns proposals in HashMap iteration order (random).
**Why it happens:** HashMap has no inherent ordering.
**How to avoid:** Collect into Vec, sort by `created_at` descending before returning.
**Warning signs:** Tests showing inconsistent ordering across runs.

### Pitfall 2: UUID Key Not in Serialized Output for List Endpoint
**What goes wrong:** The proposal ID (UUID) is the HashMap key, not a field in the Proposal struct. If you just serialize the Proposal, the ID is missing from the response.
**Why it happens:** Following the identities pattern where pubkey is the key.
**How to avoid:** ProposalSummary must explicitly include the `id: Uuid` field, populated from the HashMap key.
**Warning signs:** API response missing proposal IDs.

### Pitfall 3: Loader Must Accept Empty Object
**What goes wrong:** Empty `{}` causes deserialization to fail.
**Why it happens:** Some JSON parsers struggle with empty containers.
**How to avoid:** `HashMap<Uuid, Proposal>` deserializes `{}` correctly by default in serde_json. Test this explicitly.
**Warning signs:** Server fails to start with empty contributions.json.

### Pitfall 4: MCP Handler Signature Change
**What goes wrong:** Adding proposals to McpHandler requires updating `new()`, `handle_tool_call()`, and all callers.
**Why it happens:** Each phase adds a new data parameter.
**How to avoid:** Follow the exact pattern from Phase 13: add `proposals: Arc<HashMap<Uuid, Proposal>>` field, update constructor, pass through to tools.
**Warning signs:** Compilation errors in handler.rs and tests/common/mod.rs.

### Pitfall 5: Integration Test spawn_test_server Needs Update
**What goes wrong:** Integration tests fail because spawn_test_server doesn't load contributions.
**Why it happens:** AppState gains a new required field.
**How to avoid:** Add contributions loading to tests/common/mod.rs, either from a test fixture file or inline JSON.
**Warning signs:** All integration tests fail to compile.

### Pitfall 6: tools/list Count Must Update from 6 to 8
**What goes wrong:** MCP tools/list test asserts 6 tools, but now there are 8.
**Why it happens:** Existing test `test_tools_list_returns_six_tools` has hardcoded count.
**How to avoid:** Update test assertion from 6 to 8 and add the two new tool names to the checked list.
**Warning signs:** MCP handler test failure.

## Code Examples

### contributions.json Seed Data
```json
{
  "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d": {
    "action": "add_source",
    "status": "pending",
    "category": "rust-learning",
    "proposer": "197f6b23e16c8532c6abc838facd5ea789be0c76b2920334039bfa8b3d368d61",
    "created_at": "2026-03-08T12:00:00Z",
    "data": {
      "name": "Rust by Example",
      "url": "https://doc.rust-lang.org/rust-by-example/",
      "source_type": "documentation",
      "rank": 4,
      "why": "Interactive examples covering all Rust concepts from basics to advanced"
    },
    "votes": [
      {
        "voter": "197f6b23e16c8532c6abc838facd5ea789be0c76b2920334039bfa8b3d368d61",
        "vote": "support",
        "timestamp": "2026-03-08T12:30:00Z"
      }
    ]
  }
}
```

### REST Endpoint: GET /proposals with Filtering
```rust
// Matches audit_endpoint pattern from src/server.rs
async fn proposals_endpoint(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ProposalFilterParams>,
) -> (StatusCode, [(axum::http::HeaderName, &'static str); 1], String) {
    let mut summaries: Vec<ProposalSummary> = state.proposals.iter()
        .filter(|(_, p)| {
            // Lenient filter: invalid status returns empty, not error
            if let Some(ref status) = params.status {
                let proposal_status = serde_json::to_value(&p.status)
                    .ok().and_then(|v| v.as_str().map(|s| s.to_string()));
                match proposal_status {
                    Some(s) if s == *status => {},
                    _ => return false,
                }
            }
            if let Some(ref cat) = params.category {
                if p.category != *cat {
                    return false;
                }
            }
            true
        })
        .map(|(id, p)| ProposalSummary {
            id: *id,
            action: p.action.clone(),
            status: p.status.clone(),
            category: p.category.clone(),
            proposer: p.proposer.clone(),
            created_at: p.created_at,
        })
        .collect();

    // Sort newest first
    summaries.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    match serde_json::to_string(&summaries) {
        Ok(json) => (StatusCode::OK, [(header::CONTENT_TYPE, "application/json")], json),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, [(header::CONTENT_TYPE, "application/json")],
            format!(r#"{{"error":"Failed to serialize proposals: {}"}}"#, e)),
    }
}
```

### REST Endpoint: GET /proposals/{id}
```rust
// Matches identity_by_pubkey_endpoint pattern
async fn proposal_by_id_endpoint(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> (StatusCode, [(axum::http::HeaderName, &'static str); 1], String) {
    match state.proposals.get(&id) {
        Some(proposal) => {
            // Full detail including votes
            // Need to include the id in the response since it's not in the struct
            let mut response = serde_json::to_value(proposal).unwrap();
            response.as_object_mut().unwrap().insert("id".to_string(), serde_json::to_value(id).unwrap());
            match serde_json::to_string_pretty(&response) {
                Ok(json) => (StatusCode::OK, [(header::CONTENT_TYPE, "application/json")], json),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, ...),
            }
        }
        None => (StatusCode::NOT_FOUND, [(header::CONTENT_TYPE, "application/json")],
            r#"{"error":"Proposal not found"}"#.to_string()),
    }
}
```

### MCP Tool: list_proposals
```rust
// Matches tool_get_audit_log text formatting pattern
fn tool_list_proposals(
    arguments: Option<Value>,
    proposals: &HashMap<Uuid, Proposal>,
) -> Result<Value, ToolCallError> {
    let params: ListProposalsParams = /* parse or default */;

    let mut filtered: Vec<(&Uuid, &Proposal)> = proposals.iter()
        .filter(|(_, p)| /* status and category filtering */)
        .collect();

    filtered.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));

    let mut text = format!("Proposals ({}):\n", filtered.len());
    for (id, p) in &filtered {
        text.push_str(&format!(
            "\n- {} | {} | {} | {} | by: {} | {}",
            id, /* action */, /* status */, p.category,
            &p.proposer[..16], // truncated like audit actor
            p.created_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        ));
    }

    Ok(json!({"content": [{"type": "text", "text": text}], "isError": false}))
}
```

## Integration Points (Exact Files to Modify)

| File | Change |
|------|--------|
| `src/lib.rs` | Add `pub mod contributions;` |
| `src/config.rs` | Add `contributions_path: PathBuf` field |
| `src/main.rs` | Load contributions.json, pass identities for validation, add to AppState and McpHandler |
| `src/server.rs` | Add `proposals: Arc<HashMap<Uuid, Proposal>>` to AppState, add 2 route handlers, add 2 routes |
| `src/mcp/tools.rs` | Add ListProposalsParams, GetProposalParams, get_tools_list (6->8), handle_tool_call (add 2 arms), tool functions |
| `src/mcp/handler.rs` | Add proposals field, update new(), pass to handle_tool_call |
| `tests/common/mod.rs` | Load contributions in spawn_test_server, add to AppState and McpHandler |
| `.env` (or equivalent) | Add CONTRIBUTIONS_PATH env var |

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| axum `:param` path syntax | axum `{param}` path syntax | axum 0.7+ | Use `{id}` not `:id` in route definitions |
| `deny_unknown_fields` | `#[serde(default)]` | v2.0 project decision | Schema evolution without breaking changes |

## Open Questions

1. **Should GET /proposals/{id} include the UUID in the response body?**
   - What we know: UUID is the HashMap key, not a field in the Proposal struct. List endpoint includes it in ProposalSummary.
   - What's unclear: Whether to inject it into the detail response too.
   - Recommendation: Yes, include it. Inject `"id"` field into the JSON response. Consistent with ProposalSummary having the id.

2. **updated_at field -- include or not?**
   - What we know: CONTEXT.md lists this as Claude's discretion.
   - Recommendation: Include it. Low cost, useful for tracking when proposal status changed. Default to None, only set when status changes (curator manual process, not server logic).

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test + tokio::test 1.49 |
| Config file | Cargo.toml (dev-dependencies section) |
| Quick run command | `cargo test --lib contributions` |
| Full suite command | `cargo test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CONTRIB-01 | Status lifecycle loads/validates correctly | unit | `cargo test --lib contributions::loader` | No - Wave 0 |
| CONTRIB-02 | All 5 action types deserialize correctly | unit | `cargo test --lib contributions::types` | No - Wave 0 |
| CONTRIB-03 | Votes validated against identities, human/bot classified | unit | `cargo test --lib contributions::loader::tests::test_vote_validation` | No - Wave 0 |
| CONTRIB-04 | GET /proposals with status/category filters | integration | `cargo test --test integration_contributions test_proposals_filtered` | No - Wave 0 |
| CONTRIB-05 | GET /proposals/{id} returns full detail with votes | integration | `cargo test --test integration_contributions test_proposal_by_id` | No - Wave 0 |
| CONTRIB-06 | MCP list_proposals and get_proposal tools | integration | `cargo test --test integration_contributions test_mcp_list_proposals` | No - Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --lib contributions`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before /gsd:verify-work

### Wave 0 Gaps
- [ ] `src/contributions/types.rs` -- type definitions with unit tests for serde roundtrip
- [ ] `src/contributions/loader.rs` -- loader with unit tests for validation (valid, invalid votes, empty)
- [ ] `src/contributions/error.rs` -- error type
- [ ] `src/contributions/mod.rs` -- re-exports
- [ ] `tests/integration_contributions.rs` -- REST + MCP integration tests
- [ ] `contributions.json` -- seed data file with demo proposal
- [ ] Update `tests/common/mod.rs` -- add contributions to spawn_test_server

## Sources

### Primary (HIGH confidence)
- Source code inspection: `src/identity/` module (types.rs, loader.rs, error.rs, mod.rs) -- exact pattern to replicate
- Source code inspection: `src/audit/types.rs` -- AuditAction enum pattern, AuditFilterParams, filter_entries()
- Source code inspection: `src/server.rs` -- AppState structure, endpoint patterns, route definitions
- Source code inspection: `src/mcp/tools.rs` -- tool param structs, get_tools_list(), handle_tool_call()
- Source code inspection: `src/mcp/handler.rs` -- McpHandler constructor, field additions pattern
- Source code inspection: `src/main.rs` -- startup loading sequence, Arc wrapping
- Source code inspection: `src/config.rs` -- Config struct pattern for adding new paths
- Source code inspection: `tests/common/mod.rs` -- spawn_test_server pattern
- Source code inspection: `tests/integration_identity.rs` -- integration test patterns
- Source code inspection: `Cargo.toml` -- all dependencies already present

### Secondary (MEDIUM confidence)
- None needed -- all patterns are established in existing codebase

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all libraries already in Cargo.toml, no new dependencies
- Architecture: HIGH -- direct replication of identity + audit patterns from same codebase
- Pitfalls: HIGH -- identified from concrete code review of existing modules
- Code examples: HIGH -- adapted directly from existing src/server.rs and src/mcp/tools.rs

**Research date:** 2026-03-08
**Valid until:** 2026-04-08 (stable -- no external dependencies or moving targets)
