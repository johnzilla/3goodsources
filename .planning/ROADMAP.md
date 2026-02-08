# Roadmap: Three Good Sources (3GS)

## Milestones

- ✅ **v1.0 MVP** - Phases 1-7 (shipped 2026-02-03)
- 🚧 **v1.1 Migrate to DigitalOcean + Tech Debt** - Phases 8-11 (in progress)

## Phases

<details>
<summary>✅ v1.0 MVP (Phases 1-7) - SHIPPED 2026-02-03</summary>

### Phase 1: Foundation
**Goal**: Project scaffolding and core infrastructure
**Plans**: 3 plans

Plans:
- [x] 01-01: Cargo workspace setup
- [x] 01-02: Docker multi-stage build
- [x] 01-03: CI/CD setup

### Phase 2: Registry Schema
**Goal**: Data model for curated sources
**Plans**: 2 plans

Plans:
- [x] 02-01: JSON schema design
- [x] 02-02: Schema validation

### Phase 3: MCP Server Core
**Goal**: HTTP JSON-RPC 2.0 server
**Plans**: 3 plans

Plans:
- [x] 03-01: axum routing and handlers
- [x] 03-02: JSON-RPC protocol implementation
- [x] 03-03: Health and registry endpoints

### Phase 4: Query Engine
**Goal**: Fuzzy matching for intent patterns
**Plans**: 2 plans

Plans:
- [x] 04-01: Levenshtein distance implementation
- [x] 04-02: Keyword boosting and scoring

### Phase 5: MCP Tools
**Goal**: Four MCP tools implemented
**Plans**: 2 plans

Plans:
- [x] 05-01: get_sources and list_categories
- [x] 05-02: get_provenance and get_endorsements

### Phase 6: PKARR Identity
**Goal**: Cryptographic curator identity
**Plans**: 2 plans

Plans:
- [x] 06-01: PKARR keypair integration
- [x] 06-02: Provenance signatures

### Phase 7: Documentation & Deployment
**Goal**: Docs, landing page, Render deployment
**Plans**: 3 plans

Plans:
- [x] 07-01: README, SCHEMA, METHODOLOGY
- [x] 07-02: Landing page at 3gs.ai
- [x] 07-03: Render deployment config

</details>

### 🚧 v1.1 Migrate to DigitalOcean + Tech Debt (In Progress)

**Milestone Goal:** Migrate live deployment from Render to DigitalOcean App Platform via Ansible, clean up v1 tech debt (CORS, dependency patch, dead code).

#### Phase 8: Tech Debt Cleanup
**Goal**: Clean codebase before migration
**Depends on**: Phase 7
**Requirements**: DEPS-01, DEPS-02, CLEAN-01
**Success Criteria** (what must be TRUE):
  1. Project builds without curve25519-dalek patch in Cargo.toml
  2. All 72 tests pass without the dependency patch
  3. Unused McpError enum removed from codebase
  4. Cargo.lock contains only released crates (no git dependencies)
**Plans**: 2 plans

Plans:
- [x] 08-01-PLAN.md -- Dead code cleanup (unused imports, fields, functions, McpError enum, unused deps)
- [x] 08-02-PLAN.md -- Dependency patch removal (curve25519-dalek [patch.crates-io] single attempt — failed, patch retained)

#### Phase 9: CORS Hardening
**Goal**: Production-ready CORS configuration
**Depends on**: Phase 8
**Requirements**: CORS-01, CORS-02
**Success Criteria** (what must be TRUE):
  1. CORS configured with specific origin allowlist (3gs.ai, api.3gs.ai)
  2. MCP agents can POST to /mcp endpoint cross-origin
  3. CorsLayer::permissive() removed from code
  4. Browser OPTIONS preflight requests succeed
  5. Hardened CORS validated via integration tests (live deployment verification deferred to Phase 10)
**Plans**: 1 plan

Plans:
- [x] 09-01-PLAN.md -- Replace permissive CORS with explicit origin allowlist and add integration tests

#### Phase 10: DigitalOcean Provisioning
**Goal**: Working DO App Platform deployment in parallel with Render
**Depends on**: Phase 9
**Requirements**: DEPLOY-01, DEPLOY-02, DEPLOY-03, DEPLOY-04, DEPLOY-05, SEC-01, SEC-02
**Success Criteria** (what must be TRUE):
  1. DO app spec YAML exists at .do/app.yaml with Docker build config
  2. Ansible playbook provisions DO app via DigitalOcean API
  3. App deployed to DO at {app-name}.ondigitalocean.app
  4. Health check at /health passes in DO console
  5. Environment variables configured (PORT, RUST_LOG, REGISTRY_PATH, PKARR_SECRET_KEY as encrypted SECRET)
  6. Auto-deploy triggers on push to main branch
  7. No secrets committed to git (PKARR_SECRET_KEY, DO API token)
  8. Render deployment still running (rollback target)
**Plans**: TBD

Plans:
- [ ] 10-01: TBD

#### Phase 11: DNS Cutover & Decommission
**Goal**: 3gs.ai and api.3gs.ai served from DigitalOcean
**Depends on**: Phase 10
**Requirements**: DNS-01, DNS-02, DNS-03, DNS-04, SEC-03
**Success Criteria** (what must be TRUE):
  1. Custom domain 3gs.ai resolves to DO App Platform
  2. Custom domain api.3gs.ai resolves to DO App Platform
  3. SSL certificates auto-provisioned via Let's Encrypt (HTTPS works)
  4. Landing page loads at https://3gs.ai from DO
  5. MCP endpoint responds at https://api.3gs.ai/mcp from DO
  6. Render deployment decommissioned (resources deleted)
  7. render.yaml removed from git
  8. Git history verified clean of secrets
**Plans**: TBD

Plans:
- [ ] 11-01: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 8 → 9 → 10 → 11

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Foundation | v1.0 | 3/3 | Complete | 2026-02-01 |
| 2. Registry Schema | v1.0 | 2/2 | Complete | 2026-02-01 |
| 3. MCP Server Core | v1.0 | 3/3 | Complete | 2026-02-02 |
| 4. Query Engine | v1.0 | 2/2 | Complete | 2026-02-02 |
| 5. MCP Tools | v1.0 | 2/2 | Complete | 2026-02-02 |
| 6. PKARR Identity | v1.0 | 2/2 | Complete | 2026-02-03 |
| 7. Documentation & Deployment | v1.0 | 3/3 | Complete | 2026-02-03 |
| 8. Tech Debt Cleanup | v1.1 | 2/2 | Complete | 2026-02-08 |
| 9. CORS Hardening | v1.1 | 1/1 | Complete | 2026-02-08 |
| 10. DigitalOcean Provisioning | v1.1 | 0/1 | Not started | - |
| 11. DNS Cutover & Decommission | v1.1 | 0/1 | Not started | - |
