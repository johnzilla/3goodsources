# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-01)

**Core value:** Agents get curated, high-quality sources instead of SEO-gamed search results — three good sources per topic, human-vetted, cryptographically signed, served via open protocol.
**Current focus:** Phase 4 - HTTP Transport Layer

## Current Position

Phase: 4 of 7 (HTTP Transport Layer)
Plan: 1 of 2 in current phase
Status: In progress
Last activity: 2026-02-02 — Completed 04-01-PLAN.md (HTTP server foundation)

Progress: [████████████░] 80% (8/10 plans completed)

## Performance Metrics

**Velocity:**
- Total plans completed: 8
- Average duration: 2.4 min
- Total execution time: 0.6 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 1. Foundation | 3 | 9 min | 3 min |
| 2. Query Matching | 2 | 6 min | 3 min |
| 3. MCP Protocol | 2 | 7 min | 3.5 min |
| 4. HTTP Transport | 1 | 1 min | 1 min |

**Recent Trend:**
- Last plan: 04-01 (1 min)
- Previous: 03-02 (5 min)
- Trend: Fast execution on infrastructure setup

*Updated after each plan completion*

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- **Local-first architecture**: Build without Pubky SDK dependency for core phases. PKARR keypair is just crypto — can implement without full SDK. Pubky homeserver/trust graph deferred to v2.
- **Phase structure follows dependency order**: Foundation (data layer) → Business logic (matching) → Protocol (MCP) → Transport (HTTP) → Identity (PKARR) → Infrastructure (Docker/Render) → Documentation/Testing.
- **HashMap for category storage** (01-01): Use HashMap<String, Category> keyed by slug for direct access instead of Vec.
- **Strict serde validation** (01-01): Apply #[serde(deny_unknown_fields)] to ALL registry structs to catch schema violations early.
- **Per-module error enums** (01-01): Each module (registry, mcp, pubky) has its own thiserror-based error enum.
- **Environment config with envy** (01-02): Use envy for type-safe environment variable deserialization with dotenvy for .env support.
- **Structured logging with format switching** (01-02): Support LOG_FORMAT env var to switch between pretty (dev) and json (prod) logging.
- **Fail-fast validation** (01-02): Load registry on startup and crash with descriptive errors if invalid, rather than serving bad data.
- **Source curation standards** (01-03): Prioritize official documentation and primary sources over blog posts, include practical tools, use natural language query patterns.
- **Separate MatchConfig** (02-01): Keep matching configuration separate from Config struct — distinct concerns with clear boundaries.
- **Text normalization order** (02-01): lowercase -> strip punctuation -> remove stop words -> normalize whitespace is the canonical pipeline order.
- **Fuzzy scoring surfaces** (02-02): Match query against query_patterns (normalized), slug with hyphens replaced by spaces, and category name lowercased. Do NOT match against description (too noisy).
- **Weighted sum not multiplicative** (02-02): Use `(fuzzy_weight * fuzzy) + (keyword_weight * keyword)` for score combination. Weighted sum allows independent signal tuning and prevents one zero from killing score.
- **TDD with atomic commits** (02-02): RED-GREEN-REFACTOR cycle with separate commits (test/feat/refactor) ensures clear history and revertable changes.
- **Initialization handshake with AtomicBool** (03-01): Use Arc<AtomicBool> to track MCP initialized state. Simple, thread-safe, minimal overhead for binary state tracking.
- **Batch rejection at raw JSON level** (03-01): Check if parsed JSON is array BEFORE deserializing into JsonRpcRequest. Early rejection is cleaner and more efficient.
- **Notification handling with Option return** (03-01): Return Option<String> from handle_json - None for notifications, Some for requests. Makes distinction explicit in type system.
- **Forward-compatible InitializeParams** (03-01): Do NOT use deny_unknown_fields on MCP spec-owned types. Server should accept new spec fields without breaking.
- **JSON Schema via schemars** (03-02): Use schemars::schema_for! macro for MCP inputSchema generation. Automatic type-to-schema conversion keeps schemas in sync with Rust types.
- **Plain text tool responses** (03-02): Return plain text (not markdown) in tool responses for better agent parsing. Use labels like "Category:", "URL:" for structure.
- **Match errors as MCP success** (03-02): No-match scenarios return MCP success with isError:true and helpful messages, distinguishing business errors from protocol errors.
- **Threshold parameter for sensitivity** (03-02): get_sources accepts optional threshold (0.0-1.0) to override config default, enabling per-query match tuning.
- **Permissive CORS for MVP** (04-01): Apply CorsLayer::permissive() to allow cross-origin requests from any domain for MVP simplicity. Can be tightened later for production.
- **HTTP 204 for MCP notifications** (04-01): When McpHandler::handle_json returns None (notification), respond with HTTP 204 No Content - semantically correct, matches JSON-RPC spec.
- **PORT defaults to 3000** (04-01): Config field defaults to 3000 if PORT env var not set, required by Render deployment platform.
- **String extractor for request body** (04-01): POST /mcp handler uses String extractor for body instead of Json<Value> - McpHandler needs raw JSON for its own parsing logic.

### Pending Todos

None yet.

### Blockers/Concerns

**Architecture decisions validated:**
- Local-first approach confirmed by research — no blocking Pubky SDK dependency
- axum 0.8 + tokio stack is standard Rust pattern
- MCP protocol implementation will be manual (no mature Rust MCP library)

**For future phases:**
- Phase 5 (Identity): PKARR keypair generation — verify ed25519-dalek crate availability during Phase 4 planning
- Phase 6 (Infrastructure): Render free tier 512MB RAM limit — enforce 10MB max registry size in Phase 1

## Session Continuity

Last session: 2026-02-02T14:41:27Z — Completed 04-01-PLAN.md execution
Stopped at: Phase 4 plan 1 complete, HTTP server module ready
Resume file: None

**Phase 1 Status:** Complete ✓
- 01-01: Types and schema ✓
- 01-02: Registry loader ✓
- 01-03: Seed registry data ✓

**Phase 2 Status:** Complete ✓
- 02-01: Matcher scaffolding ✓
- 02-02: Scoring engine ✓

**Phase 3 Status:** Complete ✓
- 03-01: MCP protocol foundation ✓
- 03-02: Tool implementations ✓

**Phase 4 Status:** In progress (1/2 complete)
- 04-01: HTTP server foundation ✓
- 04-02: Main integration and server startup (next)

**Next Plan:** 04-02-main-integration (wire AppState, start axum server)
