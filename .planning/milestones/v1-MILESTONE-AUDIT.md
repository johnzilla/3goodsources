---
milestone: v1
audited: 2026-02-03T12:00:00Z
status: tech_debt
scores:
  requirements: 34/34
  phases: 7/7
  integration: 13/13
  flows: 7/7
gaps:
  requirements: []
  integration: []
  flows: []
tech_debt:
  - phase: 05-identity-and-provenance
    items:
      - "curve25519-dalek git patch in Cargo.toml (workaround for pre-release dependency issue)"
      - "Keypair not stored in AppState (only PublicKey) — needed for future signing"
  - phase: 06-infrastructure-deployment
    items:
      - "Docker image 133MB (plan specified <50MB, debian base chosen for compatibility)"
      - "PUBKY_HOMESERVER env var not in render.yaml (v1 doesn't publish to homeserver)"
      - "Render service creation, DNS for api.3gs.ai, and production verification still need human setup"
  - phase: 03-mcp-protocol-implementation
    items:
      - "McpError enum defined but unused (ToolCallError used instead)"
  - phase: 02-query-matching-engine
    items:
      - "MatchResult.score field populated but not exposed to consumers"
  - phase: 01-foundation-data-layer
    items:
      - "RegistryError::DuplicateSlug variant never constructed (HashMap prevents duplicates)"
---

# v1 Milestone Audit Report

**Milestone:** v1 (Initial Release)
**Audited:** 2026-02-03
**Status:** TECH DEBT (no blockers, accumulated items need review)

## Requirements Coverage

All 34 v1 requirements are satisfied.

| Requirement | Phase | Status |
|-------------|-------|--------|
| MCP-01: JSON-RPC 2.0 via HTTP POST at /mcp | Phase 3 | ✓ Satisfied |
| MCP-02: initialize returns protocol version, info, capabilities | Phase 3 | ✓ Satisfied |
| MCP-03: tools/list returns tools with input schemas | Phase 3 | ✓ Satisfied |
| MCP-04: tools/call dispatches to correct handler | Phase 3 | ✓ Satisfied |
| MCP-05: get_sources fuzzy matches and returns 3 sources | Phase 3 | ✓ Satisfied |
| MCP-06: list_categories returns slugs with domain tags | Phase 3 | ✓ Satisfied |
| MCP-07: get_provenance returns curator info and pubkey | Phase 3 | ✓ Satisfied |
| MCP-08: get_endorsements returns endorsements list | Phase 3 | ✓ Satisfied |
| REG-01: Loads registry from local registry.json on startup | Phase 1 | ✓ Satisfied |
| REG-02: Registry follows defined schema | Phase 1 | ✓ Satisfied |
| REG-03: 10 seed categories with 3 sources each | Phase 1 | ✓ Satisfied |
| REG-04: Sources have rank, name, URL, type, why | Phase 1 | ✓ Satisfied |
| REG-05: Normalized Levenshtein distance matching | Phase 2 | ✓ Satisfied |
| REG-06: Keyword boosting for slug terms | Phase 2 | ✓ Satisfied |
| REG-07: 0.4 threshold with helpful error | Phase 2 | ✓ Satisfied |
| IDENT-01: Generates or loads PKARR keypair | Phase 5 | ✓ Satisfied |
| IDENT-02: Private key from PKARR_SECRET_KEY env | Phase 5 | ✓ Satisfied |
| IDENT-03: Public key in provenance and health | Phase 5 | ✓ Satisfied |
| ENDP-01: POST /mcp endpoint | Phase 4 | ✓ Satisfied |
| ENDP-02: GET /health with version and pubkey | Phase 4+5 | ✓ Satisfied |
| ENDP-03: GET /registry returns raw JSON | Phase 4 | ✓ Satisfied |
| INFRA-01: Multi-stage Dockerfile | Phase 6 | ✓ Satisfied |
| INFRA-02: render.yaml for Render deployment | Phase 6 | ✓ Satisfied |
| INFRA-03: Static landing page at root | Phase 6 | ✓ Satisfied |
| INFRA-04: CORS middleware (permissive) | Phase 4 | ✓ Satisfied |
| INFRA-05: Structured logging via tracing | Phase 1 | ✓ Satisfied |
| DOCS-01: README.md | Phase 7 | ✓ Satisfied |
| DOCS-02: docs/SCHEMA.md | Phase 7 | ✓ Satisfied |
| DOCS-03: docs/METHODOLOGY.md | Phase 7 | ✓ Satisfied |
| DOCS-04: docs/PUBKY.md | Phase 7 | ✓ Satisfied |
| TEST-01: Query matching — expected hits | Phase 7 | ✓ Satisfied |
| TEST-02: Query matching — expected misses | Phase 7 | ✓ Satisfied |
| TEST-03: MCP protocol tests | Phase 7 | ✓ Satisfied |
| TEST-04: Registry loading tests | Phase 7 | ✓ Satisfied |

**Score: 34/34 requirements satisfied**

## Phase Verification Summary

| Phase | Status | Score | Verified |
|-------|--------|-------|----------|
| 1. Foundation & Data Layer | PASSED | 5/5 | 2026-02-02 |
| 2. Query Matching Engine | PASSED | 5/5 | 2026-02-02 |
| 3. MCP Protocol Implementation | PASSED | 16/16 | 2026-02-02 |
| 4. HTTP Transport Layer | PASSED | 14/14 | 2026-02-02 |
| 5. Identity & Provenance | PASSED | 5/5 | 2026-02-03 |
| 6. Infrastructure & Deployment | HUMAN_NEEDED | 4/6 | 2026-02-02 |
| 7. Documentation & Testing | PASSED | 8/8 | 2026-02-04 |

**Score: 7/7 phases complete (Phase 6 artifacts complete, deployment needs manual steps)**

## Cross-Phase Integration

**Status: PASS**

All 13 cross-phase connections verified:

| From → To | Export | Status |
|-----------|--------|--------|
| Phase 1 → 2 | Registry type → match_query | ✓ Wired |
| Phase 1 → 3 | Registry → McpHandler | ✓ Wired |
| Phase 1 → 4 | Registry → AppState | ✓ Wired |
| Phase 2 → 3 | match_query → tool_get_sources | ✓ Wired |
| Phase 2 → 3 | MatchConfig → McpHandler | ✓ Wired |
| Phase 3 → 4 | McpHandler → mcp_endpoint | ✓ Wired |
| Phase 5 → 3 | pubkey_z32 → get_provenance | ✓ Wired |
| Phase 5 → 4 | PublicKey → health_endpoint | ✓ Wired |
| Phase 5 → main | keypair → startup | ✓ Wired |
| Phase 6 → 4 | render.yaml → health route | ✓ Wired |
| Phase 6 → config | env vars → Config struct | ✓ Wired |
| Phase 7 → lib | tests → server exports | ✓ Wired |
| Phase 7 → all | integration tests → full stack | ✓ Wired |

**Orphaned exports: 5 (all benign)**
- MatchResult.score — internal field, not logged
- McpError enum — defined but ToolCallError used
- JsonRpcResponse::tool_result() — convenience helper
- InitializeParams fields — deserialized but not inspected (per MCP spec)
- RegistryError::DuplicateSlug — HashMap prevents the condition

**Missing connections: 0**

## E2E Flow Verification

All 7 critical flows verified:

| Flow | Status |
|------|--------|
| 1. Agent query → get_sources → fuzzy match → 3 sources | ✓ Complete |
| 2. Agent → get_provenance → curator + PKARR pubkey | ✓ Complete |
| 3. Agent → list_categories → 10 categories | ✓ Complete |
| 4. GET /health → status + version + pubkey | ✓ Complete |
| 5. GET /registry → full registry JSON | ✓ Complete |
| 6. MCP handshake: initialize → tools/list → tools/call | ✓ Complete |
| 7. Error handling: malformed, unknown, pre-init | ✓ Complete |

**Score: 7/7 flows verified**

## Test Results

```
Unit tests:              43 passed
Integration (registry):   7 passed
Integration (MCP):       12 passed
Integration (matching):  10 passed
─────────────────────────────────
Total:                   72 passed, 0 failed
```

## Tech Debt Summary

### Phase 1: Foundation & Data Layer
- `RegistryError::DuplicateSlug` variant never constructed (HashMap prevents duplicates) — dead code

### Phase 2: Query Matching Engine
- `MatchResult.score` field populated but not exposed to MCP consumers — could be useful for debugging

### Phase 3: MCP Protocol Implementation
- `McpError` enum defined but unused (`ToolCallError` used instead) — dead code

### Phase 5: Identity & Provenance
- `curve25519-dalek` git patch in Cargo.toml — workaround for pre-release dependency issue, fragile
- Keypair not stored in AppState (only PublicKey) — needed if future phases require signing

### Phase 6: Infrastructure & Deployment
- Docker image 133MB vs planned <50MB — debian base chosen for compatibility over Alpine
- `PUBKY_HOMESERVER` env var not in render.yaml — v1 doesn't publish to homeserver, acceptable
- **Render service creation, DNS for api.3gs.ai, and production verification require manual human setup** — documented in DNS-SETUP.md and Phase 6 VERIFICATION.md

### Total: 8 items across 5 phases

---

*Audited: 2026-02-03*
*Auditor: Claude (gsd-milestone-audit)*
