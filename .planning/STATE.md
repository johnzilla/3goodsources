# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-01)

**Core value:** Agents get curated, high-quality sources instead of SEO-gamed search results — three good sources per topic, human-vetted, cryptographically signed, served via open protocol.
**Current focus:** Phase 7 - Documentation & Testing

## Current Position

Phase: 7 of 7 (Documentation & Testing)
Plan: 3 of 4 in current phase
Status: In progress
Last activity: 2026-02-04 — Completed 07-02-PLAN.md (Deep-Dive Documentation)

Progress: [███████████████████▓] 98% (15/15 plans completed)

## Performance Metrics

**Velocity:**
- Total plans completed: 15
- Average duration: 2.6 min
- Total execution time: 1.1 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 1. Foundation | 3 | 9 min | 3 min |
| 2. Query Matching | 2 | 6 min | 3 min |
| 3. MCP Protocol | 2 | 7 min | 3.5 min |
| 4. HTTP Transport | 2 | 3 min | 1.5 min |
| 5. Identity Layer | 2 | 9 min | 4.5 min |
| 6. Infrastructure | 2 | 7 min | 3.5 min |
| 7. Documentation | 3 | 11 min | 3.7 min |

**Recent Trend:**
- Last plan: 07-02 (5 min)
- Previous: 07-03 (2 min)
- Trend: Phase 7 deep-dive documentation complete, comprehensive technical references written

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
- **Arc wrapping for shared state** (04-02): Wrap registry in Arc immediately after loading, before constructing McpHandler. Enables shared ownership between AppState and handler without data cloning.
- **Early logging for startup feedback** (04-02): Log "Server listening on {addr}" before TcpListener::bind for immediate user feedback. Port conflicts clearer when user sees intended port in logs.
- **Use pkarr with curve25519-dalek patch** (05-01): Use pkarr crate with [patch.crates-io] to override curve25519-dalek with git main branch. Fixes pre-release dependency compilation issues while maintaining PKARR compatibility.
- **Hex encoding for secret keys** (05-01): Use 64-character hex encoding for PKARR_SECRET_KEY environment variable. Human-readable, widely supported, easy to validate (exactly 2 chars per byte).
- **Optional Config field with no validation** (05-01): Make pkarr_secret_key Option<String> with validation in identity module, not config module. Separation of concerns - Config loads env vars, identity validates and uses them.
- **PublicKey storage in AppState** (05-02): Store PublicKey directly in AppState (Copy type, no Arc needed). Simple and efficient for sharing across handlers.
- **MCP layer pubkey isolation** (05-02): Pass pubkey as z-base-32 String through MCP layer instead of importing pkarr types. Keeps MCP protocol layer independent of cryptography implementation.
- **Live server pubkey in get_provenance** (05-02): Replace registry.curator.pubkey with live server pubkey in get_provenance tool. Server identity matters for verification, not static registry metadata.
- **Keypair generation in startup sequence** (05-02): Generate keypair after logging init (so warnings visible) but before Registry loading and McpHandler construction. Ensures identity available when building dependent components.
- **Rust 1.85 for edition 2024** (06-01): Use lukemathwalker/cargo-chef:latest-rust-1.85 for Docker builds. Edition 2024 requires Rust 1.85+, not available in 1.84.
- **debian:bookworm-slim over Alpine** (06-01): Accept 133MB image size with debian base for better glibc compatibility vs Alpine's ~25MB musl libc. Prioritize production stability over size optimization.
- **Baked registry.json with disk mount override** (06-01): COPY registry.json to /app/registry.json in Docker image as fallback. Render disk mount at /data/registry.json overrides in production for runtime updates.
- **Simple URL-based MCP config** (06-02): Use `"url": "https://api.3gs.ai/mcp"` for MCP client config — cleaner than node command wrapper for HTTP POST MCP servers.
- **Mermaid for architecture diagrams** (07-01): Use GitHub-native mermaid rendering in README.md for architecture visualization. No external images needed, version-controlled, renders in GitHub/GitLab/Obsidian.
- **README is map, docs/ are territory** (07-01): README explains what/why/how briefly with links to SCHEMA.md/METHODOLOGY.md/PUBKY.md for complete documentation. Avoid duplication across docs.
- **reqwest 0.12 for integration tests** (07-03): Use reqwest 0.12 (not 0.13) to match axum 0.8's hyper 1.x dependency - ensures consistent HTTP stack across test and production code.
- **Ephemeral test keypairs** (07-03): Test helper generates random keypair (no PKARR_SECRET_KEY env var) - test isolation without external state.
- **compile-time registry loading in tests** (07-03): Use include_str!("../../registry.json") for test fixtures - eliminates filesystem dependency and ensures tests use exact registry from source tree.

### Pending Todos

None yet.

### Blockers/Concerns

**Architecture decisions validated:**
- Local-first approach confirmed by research — no blocking Pubky SDK dependency
- axum 0.8 + tokio stack is standard Rust pattern
- MCP protocol implementation will be manual (no mature Rust MCP library)

**Human verification needed for Phase 6:**
- Render service deployment (create service, set PKARR_SECRET_KEY, trigger deploy)
- Production API at api.3gs.ai (configure custom domain in Render)
- End-to-end MCP request flow in production
- Landing page links to live API endpoints

## Session Continuity

Last session: 2026-02-04 — Phase 7 execution in progress
Stopped at: Completed 07-02-PLAN.md (Deep-Dive Documentation)
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

**Phase 4 Status:** Complete ✓
- 04-01: HTTP server foundation ✓
- 04-02: Main integration and server startup ✓

**Phase 5 Status:** Complete ✓
- 05-01: Identity module foundation ✓
- 05-02: Server integration ✓

**Phase 6 Status:** Complete ✓ (human verification pending for production)
- 06-01: Docker build & Render deployment ✓
- 06-02: Landing page & DNS setup ✓

**Phase 7 Status:** In progress
- 07-01: Comprehensive project README ✓
- 07-02: Registry schema documentation ✓
- 07-03: Test Infrastructure & Registry Integration Tests ✓
- 07-04: TBD (pending)
