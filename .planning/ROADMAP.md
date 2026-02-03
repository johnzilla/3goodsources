# Roadmap: Three Good Sources (3GS)

## Overview

This roadmap delivers a working MCP server in Rust that serves curated sources via HTTP POST. We start local-first (no Pubky SDK dependency), building from data layer up through protocol implementation to deployment. The 7-phase structure follows natural dependency boundaries: foundation, business logic (matching), protocol layer (MCP), HTTP transport, identity (PKARR), infrastructure (Docker/Render), and final documentation/testing. Every requirement maps to exactly one phase, ensuring complete coverage while maintaining independent verification at each stage.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Foundation & Data Layer** - Project scaffolding, registry schema, and local JSON loading
- [x] **Phase 2: Query Matching Engine** - Fuzzy search with Levenshtein distance and keyword boosting
- [x] **Phase 3: MCP Protocol Implementation** - JSON-RPC 2.0 handling and tool dispatch
- [x] **Phase 4: HTTP Transport Layer** - axum server with endpoints and CORS middleware
- [x] **Phase 5: Identity & Provenance** - PKARR keypair generation and provenance tools
- [x] **Phase 6: Infrastructure & Deployment** - Docker build and Render deployment
- [ ] **Phase 7: Documentation & Testing** - Final docs and comprehensive test suite

## Phase Details

### Phase 1: Foundation & Data Layer
**Goal**: Establish project structure with registry loading and validated data layer
**Depends on**: Nothing (first phase)
**Requirements**: REG-01, REG-02, REG-03, REG-04, INFRA-05
**Success Criteria** (what must be TRUE):
  1. Project builds with Rust 1.84 and axum 0.8 dependencies
  2. Registry JSON loads from disk into in-memory state on startup
  3. Registry schema validates correctly (version, curator, categories, sources with rank 1-3)
  4. All 10 seed categories present with 3 researched sources each
  5. Structured logging outputs startup and registry load events
**Plans**: 3 plans

Plans:
- [x] 01-01-PLAN.md -- Scaffold Rust project with types, module structure, and dependencies
- [x] 01-02-PLAN.md -- Implement registry loader, validation, config, and structured logging
- [x] 01-03-PLAN.md -- Create registry.json seed data with 10 categories and 30 sources

### Phase 2: Query Matching Engine
**Goal**: Implement fuzzy query matching that maps user queries to categories
**Depends on**: Phase 1
**Requirements**: REG-05, REG-06, REG-07
**Success Criteria** (what must be TRUE):
  1. Query "learn rust" matches rust-learning category
  2. Query "bitcoin node" matches bitcoin-node-setup category
  3. Query "email server" matches self-hosted-email category
  4. Queries below 0.4 threshold return helpful error with category list
  5. Keyword boosting increases scores when query contains slug terms
**Plans**: 2 plans

Plans:
- [x] 02-01-PLAN.md -- Matcher module scaffolding, config, errors, and text normalization pipeline
- [x] 02-02-PLAN.md -- TDD: Scoring engine and match_query public API

### Phase 3: MCP Protocol Implementation
**Goal**: Handle MCP JSON-RPC 2.0 protocol with all four tools
**Depends on**: Phase 2
**Requirements**: MCP-01, MCP-02, MCP-03, MCP-04, MCP-05, MCP-06, MCP-07, MCP-08
**Success Criteria** (what must be TRUE):
  1. initialize method returns correct protocol version and capabilities
  2. tools/list returns all four tools with valid JSON schemas
  3. tools/call dispatches to correct handler for each tool
  4. get_sources returns matching category with three ranked sources
  5. list_categories returns all category slugs with domain tags
  6. get_provenance returns curator info and verification instructions
  7. get_endorsements returns endorsements list (empty for v1)
  8. All JSON-RPC responses include jsonrpc: "2.0" field
**Plans**: 2 plans

Plans:
- [x] 03-01-PLAN.md -- MCP protocol foundation: JSON-RPC types, handler, initialize handshake, and dispatch
- [x] 03-02-PLAN.md -- TDD: Tool implementations (get_sources, list_categories, get_provenance, get_endorsements)

### Phase 4: HTTP Transport Layer
**Goal**: Serve MCP protocol over HTTP POST with health/registry endpoints
**Depends on**: Phase 3
**Requirements**: ENDP-01, ENDP-02, ENDP-03, INFRA-04
**Success Criteria** (what must be TRUE):
  1. Server starts on port 3000 (or PORT env var)
  2. POST /mcp accepts JSON-RPC requests and returns valid responses
  3. GET /health returns 200 OK with version (without pubkey for now)
  4. GET /registry returns raw registry.json for transparency
  5. CORS headers permit cross-origin requests
  6. Server handles malformed JSON gracefully with error responses
**Plans**: 2 plans

Plans:
- [x] 04-01-PLAN.md -- Server module with axum routes, AppState, CORS, and dependencies
- [x] 04-02-PLAN.md -- Main.rs integration wiring and end-to-end verification

### Phase 5: Identity & Provenance
**Goal**: Add PKARR keypair for cryptographic identity
**Depends on**: Phase 4
**Requirements**: IDENT-01, IDENT-02, IDENT-03
**Success Criteria** (what must be TRUE):
  1. Server generates PKARR keypair if PKARR_SECRET_KEY not set
  2. Private key loaded from PKARR_SECRET_KEY environment variable
  3. Public key returned in GET /health endpoint
  4. Public key included in get_provenance tool response
  5. Warning logged if keypair generated (not loaded from env)
**Plans**: 2 plans

Plans:
- [x] 05-01-PLAN.md -- Identity module with keypair generation and Config secret key field
- [x] 05-02-PLAN.md -- Wire pubkey into AppState, health endpoint, and get_provenance tool

### Phase 6: Infrastructure & Deployment
**Goal**: Deploy to Render paid tier with Docker and static landing page at 3gs.ai
**Depends on**: Phase 5
**Requirements**: INFRA-01, INFRA-02, INFRA-03
**Success Criteria** (what must be TRUE):
  1. Multi-stage Dockerfile builds successfully (<50MB image)
  2. Docker container runs locally and accepts MCP requests
  3. render.yaml deploys to Render paid tier
  4. Static landing page served at root (/) explains 3GS and connection
  5. Production server responds at api.3gs.ai domain
  6. Health endpoint returns version and pubkey in production
**Plans**: 2 plans

Plans:
- [x] 06-01-PLAN.md -- Docker build infrastructure and Render deployment config
- [x] 06-02-PLAN.md -- Static landing page and DNS setup documentation

### Phase 7: Documentation & Testing
**Goal**: Complete documentation and comprehensive test suite
**Depends on**: Phase 6
**Requirements**: DOCS-01, DOCS-02, DOCS-03, DOCS-04, TEST-01, TEST-02, TEST-03, TEST-04
**Success Criteria** (what must be TRUE):
  1. README.md explains how to run locally and connect MCP client
  2. docs/SCHEMA.md documents registry.json format completely
  3. docs/METHODOLOGY.md describes source selection criteria
  4. docs/PUBKY.md explains verification and future federated vision
  5. Query matching tests verify expected category matches
  6. Query matching tests verify unrelated queries fail appropriately
  7. MCP protocol tests validate all JSON-RPC message formats
  8. Registry loading tests confirm correct parsing and validation
**Plans**: TBD

Plans:
- [ ] 07-01: TBD
- [ ] 07-02: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5 → 6 → 7

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation & Data Layer | 3/3 | Complete | 2026-02-02 |
| 2. Query Matching Engine | 2/2 | Complete | 2026-02-02 |
| 3. MCP Protocol Implementation | 2/2 | Complete | 2026-02-02 |
| 4. HTTP Transport Layer | 2/2 | Complete | 2026-02-02 |
| 5. Identity & Provenance | 2/2 | Complete | 2026-02-03 |
| 6. Infrastructure & Deployment | 2/2 | Complete | 2026-02-03 |
| 7. Documentation & Testing | 0/TBD | Not started | - |

---
*Roadmap created: 2026-02-01*
*Last updated: 2026-02-03 after Phase 6 execution complete*
