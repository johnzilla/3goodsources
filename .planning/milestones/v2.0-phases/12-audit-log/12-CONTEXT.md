# Phase 12: Audit Log - Context

**Gathered:** 2026-03-07
**Status:** Ready for planning

<domain>
## Phase Boundary

Append-only audit log recording every registry change with cryptographic signatures and hash chain. New `src/audit/` module following the registry pattern, `audit_log.json` data file with 40 retroactive entries, GET /audit REST endpoint with query filtering, and get_audit_log MCP tool. Server is read-only — signing happens offline.

</domain>

<decisions>
## Implementation Decisions

### Signing format
- Field concatenation, NOT sorted-key JSON: `timestamp|action|category|sha256(data_json)|actor_pubkey`
- Signatures hex-encoded (consistent with PKARR pubkey display convention in project)
- SHA-256 hash of the data JSON portion (sorted-key compact) covers variable-schema data without breaking signing
- Server verifies all signatures at load time — invalid entries cause startup failure (fail-fast, consistent with registry loader)

### Retroactive entries
- 40 entries total: 10 category_added + 30 source_added
- All timestamped to v1.0 ship date: 2026-02-03T00:00:00Z
- Use standard action types (source_added, category_added) — not a separate backfill action type
- Ordering: all 10 category_added entries first, then 30 source_added grouped by category
- Hash chain starts from entry #1 (first category_added), previous_hash is null for genesis entry

### Endpoint design
- Path: GET /audit (matches /health, /registry convention)
- Returns full log by default — raw JSON array, no wrapper object
- Optional query params: since (ISO timestamp), category (slug), action (enum value)
- No pagination for v2.0 (40 entries, growth is manual curation — years before it matters)
- MCP tool get_audit_log accepts same filter params as REST endpoint

### Entry schema
- Future-proof: action enum includes identity and proposal types now (identity_registered, identity_updated, proposal_submitted, proposal_status_changed, vote_cast)
- Action type is strict Rust enum with serde rename (snake_case strings in JSON)
- category field is Option<String> — null for non-registry actions
- Entry ID: UUID v4
- Use #[serde(default)] not deny_unknown_fields — schema must evolve without breaking historical entries

### Claude's Discretion
- Exact uuid crate version and features (v4 generation)
- Audit log file format (single JSON array vs JSONL)
- Specific validation rules in loader beyond signature verification
- Test fixture design for invalid audit entries

</decisions>

<specifics>
## Specific Ideas

- Canonical signing format is a one-way door — must be documented clearly so third parties can verify signatures independently
- Hash chain provides tamper detection but chain validation is NOT required at runtime in v2.0 (field exists, validation deferred)
- ed25519-dalek / PKARR key interoperability needs verification with a unit test early in implementation

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/registry/` module: 4-file pattern (mod.rs, types.rs, loader.rs, error.rs) — replicate exactly for src/audit/
- `src/pubky/identity.rs`: PKARR keypair loading — same key used for signing audit entries offline
- `tests/common/mod.rs`: spawn_test_server pattern — extend with audit_log field in AppState

### Established Patterns
- AppState uses Arc<T> for shared immutable data — add `pub audit_log: Arc<Vec<AuditEntry>>`
- Registry loader: tokio::fs::read_to_string → serde_json::from_str → validate → Result<T, Error>
- MCP tool responses: `json!({"content": [{"type": "text", "text": "..."}], "isError": false})`
- Error types use thiserror derive with structured variants
- Integration tests use reqwest::Client against spawned test server on random port

### Integration Points
- `src/server.rs`: Add audit_log to AppState struct, add GET /audit route to Router
- `src/mcp/tools.rs`: Add get_audit_log match arm in handle_tool_call, update tools/list response
- `src/main.rs`: Load audit_log.json at startup alongside registry.json
- `tests/common/mod.rs`: Update spawn_test_server to load test audit log
- `Cargo.toml`: Add ed25519-dalek, chrono, sha2, uuid dependencies

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 12-audit-log*
*Context gathered: 2026-03-07*
