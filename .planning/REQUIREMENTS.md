# Requirements: Three Good Sources (3GS)

**Defined:** 2026-02-01
**Core Value:** Agents get curated, high-quality sources instead of SEO-gamed search results — three good sources per topic, human-vetted, cryptographically signed, served via open protocol.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### MCP Server

- [ ] **MCP-01**: Server accepts JSON-RPC 2.0 requests via HTTP POST at `/mcp`
- [ ] **MCP-02**: `initialize` method returns protocol version, server info, and capabilities
- [ ] **MCP-03**: `tools/list` method returns all available tools with input schemas
- [ ] **MCP-04**: `tools/call` method dispatches to the correct tool handler and returns MCP content format
- [ ] **MCP-05**: `get_sources` tool accepts a query string, fuzzy matches against category query_patterns, and returns the matching category's three ranked sources plus registry version and curator pubkey
- [ ] **MCP-06**: `list_categories` tool returns all category slugs with their domain tags
- [ ] **MCP-07**: `get_provenance` tool returns curator name, PKARR pubkey, registry version, endorsements list, and verification instructions
- [ ] **MCP-08**: `get_endorsements` tool returns endorsed curators, optionally filtered by scope/domain

### Registry

- [ ] **REG-01**: Server loads registry from local `registry.json` file on startup into immutable in-memory state
- [ ] **REG-02**: Registry JSON follows defined schema: version, updated date, curator info, endorsements array, categories map with query_patterns and ranked sources
- [ ] **REG-03**: Registry contains 10 seed categories with 3 real, researched sources each
- [ ] **REG-04**: Each source has rank (1-3), name, URL, type (documentation/tutorial/video/article/tool/repo/forum/book/course/api), and why field
- [ ] **REG-05**: Query matching uses normalized Levenshtein distance (strsim crate) against all category query_patterns
- [ ] **REG-06**: Keyword matching boosts score when query contains terms from category slug or patterns
- [ ] **REG-07**: Match threshold of 0.4 — below threshold returns error with available categories and suggestion to request new categories via GitHub

### Identity

- [ ] **IDENT-01**: Server generates or loads a PKARR keypair on startup
- [ ] **IDENT-02**: Private key stored via `PKARR_SECRET_KEY` environment variable, never committed to repo
- [ ] **IDENT-03**: Public key included in provenance responses and health endpoint

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
| MCP-01 | — | Pending |
| MCP-02 | — | Pending |
| MCP-03 | — | Pending |
| MCP-04 | — | Pending |
| MCP-05 | — | Pending |
| MCP-06 | — | Pending |
| MCP-07 | — | Pending |
| MCP-08 | — | Pending |
| REG-01 | — | Pending |
| REG-02 | — | Pending |
| REG-03 | — | Pending |
| REG-04 | — | Pending |
| REG-05 | — | Pending |
| REG-06 | — | Pending |
| REG-07 | — | Pending |
| IDENT-01 | — | Pending |
| IDENT-02 | — | Pending |
| IDENT-03 | — | Pending |
| ENDP-01 | — | Pending |
| ENDP-02 | — | Pending |
| ENDP-03 | — | Pending |
| INFRA-01 | — | Pending |
| INFRA-02 | — | Pending |
| INFRA-03 | — | Pending |
| INFRA-04 | — | Pending |
| INFRA-05 | — | Pending |
| DOCS-01 | — | Pending |
| DOCS-02 | — | Pending |
| DOCS-03 | — | Pending |
| DOCS-04 | — | Pending |
| TEST-01 | — | Pending |
| TEST-02 | — | Pending |
| TEST-03 | — | Pending |
| TEST-04 | — | Pending |

**Coverage:**
- v1 requirements: 34 total
- Mapped to phases: 0
- Unmapped: 34 (pending roadmap creation)

---
*Requirements defined: 2026-02-01*
*Last updated: 2026-02-01 after initial definition*
