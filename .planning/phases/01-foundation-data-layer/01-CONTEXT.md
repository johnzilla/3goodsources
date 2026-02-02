# Phase 1: Foundation & Data Layer - Context

**Gathered:** 2026-02-01
**Status:** Ready for planning

<domain>
## Phase Boundary

Establish the Rust project structure, define the registry JSON schema as Rust types, create registry.json with 10 seed categories (3 sources each), and load it into immutable in-memory state on startup. Structured logging throughout. No HTTP server, no MCP protocol, no query matching — those are later phases.

</domain>

<decisions>
## Implementation Decisions

### Registry schema strictness
- Strict validation: `#[serde(deny_unknown_fields)]` on all types — reject unknown fields on load
- Every category must have exactly 3 sources (not 1-3, not more) — validation enforces this
- `endorsements` array is required but can be empty `[]`
- Each category must have at least 3 `query_patterns`
- All required fields enforced at deserialization — if it parses, it's valid

### Seed data curation
- Use researched sources as-is for MVP — replace with hand-curated sources later
- Generic placeholder for curator info (not real name/identity yet) — real info added at deploy time
- PKARR pubkey field uses `"pk:placeholder"` until Phase 5 adds real keypair
- `why` field: one sentence per source, brief and direct

### Startup behavior
- Missing registry.json: crash with clear fatal error and path attempted
- Malformed registry.json: crash with specific serde error (line/column if available)
- Registry path: require explicit `REGISTRY_PATH` env var — no default, no guessing
- Log full summary on successful load: category count, source count, registry version

### Project structure
- Nested modules as specified: `src/mcp/`, `src/registry/`, `src/pubky/`, `src/error.rs`
- One error enum per module: `RegistryError`, `McpError`, `PubkyError` (using thiserror)
- `anyhow` in `main.rs` only
- Logging: env-switchable format — pretty/colored for dev (default), JSON structured for production (via `LOG_FORMAT=json` or similar env var)
- Rust edition 2024, latest stable toolchain (update Docker image accordingly if rust:1.84-slim doesn't support edition 2024)

### Claude's Discretion
- Exact Cargo.toml dependency versions (pin to latest compatible)
- Internal type naming conventions (Registry, Category, Source, etc.)
- Whether to use `serde_json::from_reader` vs `from_str` for loading
- Tracing subscriber configuration details

</decisions>

<specifics>
## Specific Ideas

- The schema IS the protocol — the JSON structure defines what 3GS is. Getting the types right here matters more than most foundation phases.
- Source types enum: documentation, tutorial, video, article, tool, repo, forum, book, course, api
- Category slugs are kebab-case strings (e.g., `bitcoin-node-setup`, `rust-learning`)
- Ranks are 1, 2, 3 — not 0-indexed

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 01-foundation-data-layer*
*Context gathered: 2026-02-01*
