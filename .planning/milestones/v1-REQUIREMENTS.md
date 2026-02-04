# Requirements Archive: v1 MVP

**Archived:** 2026-02-03
**Status:** SHIPPED

This is the archived requirements specification for v1.
For current requirements, see `.planning/REQUIREMENTS.md` (created for next milestone).

---

**Defined:** 2026-02-01
**Core Value:** Agents get curated, high-quality sources instead of SEO-gamed search results — three good sources per topic, human-vetted, cryptographically signed, served via open protocol.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### MCP Server

- [x] **MCP-01**: Server accepts JSON-RPC 2.0 requests via HTTP POST at `/mcp`
- [x] **MCP-02**: `initialize` method returns protocol version, server info, and capabilities
- [x] **MCP-03**: `tools/list` method returns all available tools with input schemas
- [x] **MCP-04**: `tools/call` method dispatches to the correct tool handler and returns MCP content format
- [x] **MCP-05**: `get_sources` tool accepts a query string, fuzzy matches against category query_patterns, and returns the matching category's three ranked sources plus registry version and curator pubkey
- [x] **MCP-06**: `list_categories` tool returns all category slugs with their domain tags
- [x] **MCP-07**: `get_provenance` tool returns curator name, PKARR pubkey, registry version, endorsements list, and verification instructions
- [x] **MCP-08**: `get_endorsements` tool returns endorsed curators, optionally filtered by scope/domain

### Registry

- [x] **REG-01**: Server loads registry from local `registry.json` file on startup into immutable in-memory state
- [x] **REG-02**: Registry JSON follows defined schema: version, updated date, curator info, endorsements array, categories map with query_patterns and ranked sources
- [x] **REG-03**: Registry contains 10 seed categories with 3 real, researched sources each
- [x] **REG-04**: Each source has rank (1-3), name, URL, type (documentation/tutorial/video/article/tool/repo/forum/book/course/api), and why field
- [x] **REG-05**: Query matching uses normalized Levenshtein distance (strsim crate) against all category query_patterns
- [x] **REG-06**: Keyword matching boosts score when query contains terms from category slug or patterns
- [x] **REG-07**: Match threshold of 0.4 — below threshold returns error with available categories and suggestion to request new categories via GitHub

### Identity

- [x] **IDENT-01**: Server generates or loads a PKARR keypair on startup
- [x] **IDENT-02**: Private key stored via `PKARR_SECRET_KEY` environment variable, never committed to repo
- [x] **IDENT-03**: Public key included in provenance responses and health endpoint

### Endpoints

- [x] **ENDP-01**: `POST /mcp` — MCP JSON-RPC endpoint handling all MCP methods
- [x] **ENDP-02**: `GET /health` — returns 200 OK with version and PKARR pubkey
- [x] **ENDP-03**: `GET /registry` — returns raw registry JSON for transparency

### Infrastructure

- [x] **INFRA-01**: Multi-stage Dockerfile: `rust:1.84-slim` builder, `debian:bookworm-slim` runtime with ca-certificates, exposes port 3000
- [x] **INFRA-02**: `render.yaml` for Render paid tier deployment with env vars for RUST_LOG, PKARR_SECRET_KEY
- [x] **INFRA-03**: Static landing page at root explaining what 3GS is, how to connect, how to verify
- [x] **INFRA-04**: CORS middleware (permissive for MVP)
- [x] **INFRA-05**: Structured logging via tracing/tracing-subscriber

### Documentation

- [x] **DOCS-01**: README.md — project description, how it works, how to run locally, how to connect MCP client, how to verify registry, how to request categories, domains listed
- [x] **DOCS-02**: docs/SCHEMA.md — detailed registry.json format documentation
- [x] **DOCS-03**: docs/METHODOLOGY.md — source selection criteria, quality over quantity, transparency, verification
- [x] **DOCS-04**: docs/PUBKY.md — why Pubky, how verification works, how endorsements work, future federated vision

### Testing

- [x] **TEST-01**: Query matching tests — queries hit expected categories (e.g., "learn rust" matches rust-learning)
- [x] **TEST-02**: Query matching tests — unrelated queries return no match (below threshold)
- [x] **TEST-03**: MCP protocol tests — initialize, tools/list, tools/call return correct JSON-RPC responses
- [x] **TEST-04**: Registry loading tests — local registry.json loads and parses correctly

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| MCP-01 | Phase 3 | Complete |
| MCP-02 | Phase 3 | Complete |
| MCP-03 | Phase 3 | Complete |
| MCP-04 | Phase 3 | Complete |
| MCP-05 | Phase 3 | Complete |
| MCP-06 | Phase 3 | Complete |
| MCP-07 | Phase 3 | Complete |
| MCP-08 | Phase 3 | Complete |
| REG-01 | Phase 1 | Complete |
| REG-02 | Phase 1 | Complete |
| REG-03 | Phase 1 | Complete |
| REG-04 | Phase 1 | Complete |
| REG-05 | Phase 2 | Complete |
| REG-06 | Phase 2 | Complete |
| REG-07 | Phase 2 | Complete |
| IDENT-01 | Phase 5 | Complete |
| IDENT-02 | Phase 5 | Complete |
| IDENT-03 | Phase 5 | Complete |
| ENDP-01 | Phase 4 | Complete |
| ENDP-02 | Phase 4 | Complete |
| ENDP-03 | Phase 4 | Complete |
| INFRA-01 | Phase 6 | Complete |
| INFRA-02 | Phase 6 | Complete |
| INFRA-03 | Phase 6 | Complete |
| INFRA-04 | Phase 4 | Complete |
| INFRA-05 | Phase 1 | Complete |
| DOCS-01 | Phase 7 | Complete |
| DOCS-02 | Phase 7 | Complete |
| DOCS-03 | Phase 7 | Complete |
| DOCS-04 | Phase 7 | Complete |
| TEST-01 | Phase 7 | Complete |
| TEST-02 | Phase 7 | Complete |
| TEST-03 | Phase 7 | Complete |
| TEST-04 | Phase 7 | Complete |

## Milestone Summary

**Shipped:** 34 of 34 v1 requirements
**Adjusted:**
- INFRA-01: Used Rust 1.85 (not 1.84) due to edition 2024 requirement
- INFRA-02: Used Render paid tier (starter) instead of free tier to avoid cold starts; omitted PUBKY_HOMESERVER (not used in v1)

**Dropped:** None

---
*Archived: 2026-02-03 as part of v1 milestone completion*
