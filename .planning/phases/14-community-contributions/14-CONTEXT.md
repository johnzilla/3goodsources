# Phase 14: Community Contributions - Context

**Gathered:** 2026-03-08
**Status:** Ready for planning

<domain>
## Phase Boundary

Community proposal system for source changes with transparent vote tracking. New `src/contributions/` module following the registry pattern, `contributions.json` data file with one demo proposal, GET /proposals and GET /proposals/{id} REST endpoints with filtering, and list_proposals and get_proposal MCP tools. Server is read-only — contribution data is curator-managed JSON.

</domain>

<decisions>
## Implementation Decisions

### Proposal schema
- contributions.json structured as object keyed by UUID v4 (O(1) lookup, consistent with identities.json pattern)
- Action field as enum: add_source, update_source, remove_source, add_category, update_category
- Action-specific details stored in a flexible `data` JSON object (matches audit entry pattern)
- Status lifecycle with restricted transitions only: pending->approved, pending->rejected, pending->withdrawn
- Loader validates no illegal status transitions exist at load time (fail-fast, consistent with other loaders)
- Each proposal tracks: id, action, status, category, proposer (pubkey), created_at, data, votes

### Vote tracking
- Individual vote records stored as array per proposal: [{ voter: pubkey, vote: support/oppose, timestamp }]
- Binary voting only: support and oppose (no abstain — not voting is implicit abstention)
- Voter identity type (human/bot) determined by cross-referencing voter pubkey against identities.json at load time
- Unknown voters (pubkey not in identities.json) rejected at load time — all voters must be registered identities
- Loader takes identities as parameter for cross-reference validation

### Seed data
- Ship with one demo proposal (add_source action, pending status)
- Clearly marked as demo/example data, not a real proposal
- Include 1-2 sample votes from John Turner's pubkey to show full data structure
- Empty contributions.json ({}) loads successfully as valid empty state

### Endpoint design
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
- Error response format for invalid filter params (lenient vs strict — prior pattern is lenient)

</decisions>

<specifics>
## Specific Ideas

- Audit log already has ProposalSubmitted, ProposalStatusChanged, VoteCast action types defined — no new audit actions needed
- Contributions loader needs both the contributions file path AND the identities HashMap for vote validation
- The separation of summary (list) vs detail (single) endpoints mirrors how real APIs work — agents checking proposals don't need full vote details every time

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/audit/` and `src/identity/`: 4-file module pattern (mod.rs, types.rs, loader.rs, error.rs) — replicate for src/contributions/
- `src/audit/types.rs`: AuditAction enum already includes ProposalSubmitted, ProposalStatusChanged, VoteCast
- `src/identity/types.rs`: IdentityType enum (Human/Bot) — used for vote classification via cross-reference
- `src/mcp/tools.rs`: 6 existing tools with schemars JsonSchema derives — add list_proposals and get_proposal as tools 7-8

### Established Patterns
- AppState uses Arc<T> for shared immutable data — add proposals as Arc<HashMap<Uuid, Proposal>>
- Loader: tokio::fs::read_to_string -> serde_json::from_str -> validate -> Result<T, Error>
- MCP tool responses: json!({"content": [{"type": "text", "text": "..."}], "isError": false})
- Error types use thiserror derive with structured variants
- REST endpoints return (StatusCode, [(header, value)], String) tuple
- Optional query params via axum Query<T> extractor (see AuditFilterParams pattern)
- Lenient filter behavior from Phase 12: invalid params ignored, not errored

### Integration Points
- `src/server.rs`: Add proposals to AppState, add GET /proposals and GET /proposals/{id} routes
- `src/mcp/tools.rs`: Add list_proposals and get_proposal match arms, update tools/list (6 -> 8 tools)
- `src/mcp/handler.rs`: Pass proposals to McpHandler
- `src/main.rs`: Load contributions.json at startup, pass identities for vote validation
- `src/config.rs`: Add contributions_path field
- `tests/common/mod.rs`: Update spawn_test_server to load test contributions

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 14-community-contributions*
*Context gathered: 2026-03-08*
