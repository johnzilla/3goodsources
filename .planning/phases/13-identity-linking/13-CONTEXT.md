# Phase 13: Identity Linking - Context

**Gathered:** 2026-03-08
**Status:** Ready for planning

<domain>
## Phase Boundary

Cross-platform identity mapping linking PKARR pubkeys to platform handles (X, Nostr, GitHub) with proof URLs for independent verification. New `src/identity/` module following the registry pattern, `identities.json` data file with John Turner's seed identity, GET /identities and GET /identities/{pubkey} REST endpoints, and get_identity MCP tool. Server is read-only — identity data is curator-managed JSON.

</domain>

<decisions>
## Implementation Decisions

### Identity schema
- identities.json structured as object keyed by pubkey (O(1) lookup, matches /identities/{pubkey} endpoint)
- Each identity has: name (display name), type ("human" or "bot"), platforms (array of claim objects)
- Platform claims as array of objects: { platform, handle, proof_url }
- Platform enum: x, nostr, github — strict Rust enum, add more later as new variants
- Use #[serde(default)] for the identity struct (schema evolution, consistent with audit types)

### Bot accountability chain
- Bot identities include operator_pubkey field pointing to a human identity's pubkey in the same file
- operator_pubkey is Option<String> with #[serde(skip_serializing_if = "Option::is_none")] — absent for humans, present for bots
- Loader validates at load time: bot's operator_pubkey must reference an existing human identity (fail-fast, consistent with audit loader)
- No bot identities in v2.0 seed data — only John Turner's human identity. Bots added when actual bots exist

### Proof URL strategy
- X proof: tweet URL containing PKARR pubkey (e.g., https://x.com/user/status/12345)
- GitHub proof: Gist URL containing PKARR pubkey (e.g., https://gist.github.com/user/abc123)
- Nostr proof: NIP-05 verification URL (e.g., https://domain/.well-known/nostr.json?name=user)
- Server stores proof_url as string only — no URL validation at load time (automated verification is future VERIFY-01 requirement)
- Third parties verify proofs independently by visiting the URL

### Seed identity data
- Use the server's existing PKARR pubkey (same key from registry.json curator field / PKARR_SECRET_KEY)
- Placeholder proof URLs and handles for now — clearly marked as TODO, replace with real ones later
- John Turner identity only — no bot identities in seed data

### Claude's Discretion
- Exact loader implementation details beyond fail-fast validation
- Filter parameters for GET /identities endpoint (if any beyond returning all)
- MCP tool response text formatting
- Test fixture design for invalid identities
- Whether to add a created_at/registered_at timestamp field to identity objects

</decisions>

<specifics>
## Specific Ideas

- Identity pubkey in identities.json should match the curator pubkey in registry.json for John Turner — this is the same person
- Proof URL convention should be documented so future contributors know what format to use per platform
- The loader should handle the case where identities.json is empty ({}) gracefully — valid state for a fresh deployment

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/audit/` module: 4-file pattern (mod.rs, types.rs, loader.rs, error.rs) — replicate for src/identity/
- `src/audit/types.rs`: AuditAction enum with serde rename_all — same pattern for Platform enum
- `src/audit/loader.rs`: File loading + validation at startup — same pattern for identity loader
- `src/pubky/identity.rs`: PKARR keypair loading — same key referenced by identity data
- `src/mcp/tools.rs`: 5 existing tools with schemars JsonSchema derives — add get_identity as 6th
- `tests/common/mod.rs`: spawn_test_server pattern — extend with identities field in AppState

### Established Patterns
- AppState uses Arc<T> for shared immutable data — add identities as Arc<HashMap<String, Identity>>
- Registry loader: tokio::fs::read_to_string -> serde_json::from_str -> validate -> Result<T, Error>
- MCP tool responses: json!({"content": [{"type": "text", "text": "..."}], "isError": false})
- Error types use thiserror derive with structured variants
- REST endpoints return (StatusCode, [(header, value)], String) tuple
- Optional query params via axum Query<T> extractor (see AuditFilterParams)

### Integration Points
- `src/server.rs`: Add identities to AppState struct, add GET /identities and GET /identities/{pubkey} routes
- `src/mcp/tools.rs`: Add get_identity match arm in handle_tool_call, update tools/list (5 -> 6 tools)
- `src/mcp/handler.rs`: Pass identities to McpHandler
- `src/main.rs`: Load identities.json at startup alongside registry.json and audit_log.json
- `tests/common/mod.rs`: Update spawn_test_server to load test identities
- `Cargo.toml`: May not need new dependencies (serde, chrono, etc. already present)

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 13-identity-linking*
*Context gathered: 2026-03-08*
