# Requirements: Three Good Sources (3GS)

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

- [ ] **REG-01**: Server loads registry from local `registry.json` file on startup into immutable in-memory state
- [ ] **REG-02**: Registry JSON follows defined schema: version, updated date, curator info, endorsements array, categories map with query_patterns and ranked sources
- [ ] **REG-03**: Registry contains 10 seed categories with 3 real, researched sources each
- [ ] **REG-04**: Each source has rank (1-3), name, URL, type (documentation/tutorial/video/article/tool/repo/forum/book/course/api), and why field
- [ ] **REG-05**: Query matching uses normalized Levenshtein distance (strsim crate) against all category query_patterns
- [ ] **REG-06**: Keyword matching boosts score when query contains terms from category slug or patterns
- [ ] **REG-07**: Match threshold of 0.4 — below threshold returns error with available categories and suggestion to request new categories via GitHub

### Identity

- [x] **IDENT-01**: Server generates or loads a PKARR keypair on startup
- [x] **IDENT-02**: Private key stored via `PKARR_SECRET_KEY` environment variable, never committed to repo
- [x] **IDENT-03**: Public key included in provenance responses and health endpoint

### Endpoints

- [ ] **ENDP-01**: `POST /mcp` — MCP JSON-RPC endpoint handling all MCP methods
- [ ] **ENDP-02**: `GET /health` — returns 200 OK with version and PKARR pubkey
- [ ] **ENDP-03**: `GET /registry` — returns raw registry JSON for transparency/debugging

### Infrastructure

- [ ] **INFRA-01**: Multi-stage Dockerfile: `rust:1.84-slim` builder, `debian:bookworm-slim` runtime with ca-certificates, exposes port 3000
- [ ] **INFRA-02**: `render.yaml` for Render free tier deployment with env vars for RUST_LOG, PKARR_SECRET_KEY, PUBKY_HOMESERVER
- [ ] **INFRA-03**: Static landing page at root explaining what 3GS is, how to connect, how to verify
- [ ] **INFRA-04**: CORS middleware (permissive for MVP)
- [ ] **INFRA-05**: Structured logging via tracing/tracing-subscriber

### Documentation

- [ ] **DOCS-01**: README.md — project description, how it works, how to run locally, how to connect MCP client, how to verify registry, how to request categories, domains listed
- [ ] **DOCS-02**: docs/SCHEMA.md — detailed registry.json format documentation
- [ ] **DOCS-03**: docs/METHODOLOGY.md — source selection criteria, quality over quantity, transparency, verification
- [ ] **DOCS-04**: docs/PUBKY.md — why Pubky, how verification works, how endorsements work, future federated vision

### Testing

- [ ] **TEST-01**: Query matching tests — queries hit expected categories (e.g., "learn rust" matches rust-learning)
- [ ] **TEST-02**: Query matching tests — unrelated queries return no match (below threshold)
- [ ] **TEST-03**: MCP protocol tests — initialize, tools/list, tools/call return correct JSON-RPC responses
- [ ] **TEST-04**: Registry loading tests — local registry.json loads and parses correctly

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Pubky Integration

- **PUBKY-01**: Publish registry to Pubky homeserver at `/3gs/registry`
- **PUBKY-02**: Registry automatically signed by PKARR key on publish
- **PUBKY-03**: Endorsements published as Pubky trust relationships scoped by domain tag
- **PUBKY-04**: Agents can traverse trust graph to find curators for specific domains
- **PUBKY-05**: Fallback to local registry when homeserver unreachable

### Transport

- **TRANS-01**: stdio transport for Claude Desktop native integration
- **TRANS-02**: SSE (Streamable HTTP) transport per newer MCP standard

### Advanced Features

- **ADV-01**: Agent feedback loop (did the source help?)
- **ADV-02**: Community voting on sources via Nostr/Pubky events
- **ADV-03**: Multiple registries with trust graph traversal
- **ADV-04**: Domain-specific forks (3gs-woodworking, 3gs-homestead)

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Algorithmic ranking | Defeats human curation purpose — curator manually ranks top 3 |
| User voting/stars | Becomes popularity contest, gameable — trust the curator |
| Automated source discovery | Brings in SEO spam — manual curation only |
| Search engine integration | Defeats anti-SEO purpose — curated patterns only |
| ML recommendations | Black box, not transparent — explicit curator choices |
| Dynamic content scraping | Fragile, maintenance nightmare — static URLs only |
| Analytics/tracking | Privacy violation — no user tracking |
| Source comments/discussion | Scope creep, moderation burden — curator picks sources |
| Mobile app | Web landing page sufficient for MVP |
| Routstr integration | Future pricing transparency, not MVP |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

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
| INFRA-01 | Phase 6 | Pending |
| INFRA-02 | Phase 6 | Pending |
| INFRA-03 | Phase 6 | Pending |
| INFRA-04 | Phase 4 | Complete |
| INFRA-05 | Phase 1 | Complete |
| DOCS-01 | Phase 7 | Pending |
| DOCS-02 | Phase 7 | Pending |
| DOCS-03 | Phase 7 | Pending |
| DOCS-04 | Phase 7 | Pending |
| TEST-01 | Phase 7 | Pending |
| TEST-02 | Phase 7 | Pending |
| TEST-03 | Phase 7 | Pending |
| TEST-04 | Phase 7 | Pending |

**Coverage:**
- v1 requirements: 34 total
- Mapped to phases: 34
- Unmapped: 0 (complete coverage)

**By Phase:**
- Phase 1 (Foundation & Data Layer): 5 requirements (REG-01, REG-02, REG-03, REG-04, INFRA-05)
- Phase 2 (Query Matching Engine): 3 requirements (REG-05, REG-06, REG-07)
- Phase 3 (MCP Protocol Implementation): 8 requirements (MCP-01 through MCP-08)
- Phase 4 (HTTP Transport Layer): 4 requirements (ENDP-01, ENDP-02, ENDP-03, INFRA-04)
- Phase 5 (Identity & Provenance): 3 requirements (IDENT-01, IDENT-02, IDENT-03)
- Phase 6 (Infrastructure & Deployment): 3 requirements (INFRA-01, INFRA-02, INFRA-03)
- Phase 7 (Documentation & Testing): 8 requirements (DOCS-01 through DOCS-04, TEST-01 through TEST-04)

---
*Requirements defined: 2026-02-01*
*Last updated: 2026-02-03 after Phase 5 execution complete*
