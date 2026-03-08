# Roadmap: Three Good Sources (3GS)

## Milestones

- ✅ **v1.0 MVP** - Phases 1-7 (shipped 2026-02-03)
- ✅ **v1.1 Migrate to DigitalOcean + Tech Debt** - Phases 8-11 (shipped 2026-02-09)
- 🚧 **v2.0 Community Curation** - Phases 12-14 (in progress)

## Phases

<details>
<summary>✅ v1.0 MVP (Phases 1-7) - SHIPPED 2026-02-03</summary>

- [x] Phase 1: Foundation (3/3 plans) — completed 2026-02-01
- [x] Phase 2: Registry Schema (2/2 plans) — completed 2026-02-01
- [x] Phase 3: MCP Server Core (3/3 plans) — completed 2026-02-02
- [x] Phase 4: Query Engine (2/2 plans) — completed 2026-02-02
- [x] Phase 5: MCP Tools (2/2 plans) — completed 2026-02-02
- [x] Phase 6: PKARR Identity (2/2 plans) — completed 2026-02-03
- [x] Phase 7: Documentation & Deployment (3/3 plans) — completed 2026-02-03

</details>

<details>
<summary>✅ v1.1 Migrate to DigitalOcean + Tech Debt (Phases 8-11) - SHIPPED 2026-02-09</summary>

- [x] Phase 8: Tech Debt Cleanup (2/2 plans) — completed 2026-02-08
- [x] Phase 9: CORS Hardening (1/1 plan) — completed 2026-02-08
- [x] Phase 10: DigitalOcean Provisioning (1/1 plan) — completed 2026-02-08
- [x] Phase 11: DNS Cutover & Decommission (2/2 plans) — completed 2026-02-09

</details>

### 🚧 v2.0 Community Curation (In Progress)

**Milestone Goal:** Add public audit log, identity linking, and community contribution infrastructure -- all read-only on server, curator-managed JSON files.

- [x] **Phase 12: Audit Log** - Append-only curation transparency with signed entries, hash chain, REST + MCP endpoints (completed 2026-03-08)
- [ ] **Phase 13: Identity Linking** - Cross-platform identity mapping with proof URLs, REST + MCP endpoints
- [ ] **Phase 14: Community Contributions** - Proposal lifecycle with human/bot vote separation, REST + MCP endpoints

## Phase Details

### Phase 12: Audit Log
**Goal**: Every registry change is publicly auditable through a signed, hash-chained audit log
**Depends on**: Phase 11 (v1.1 complete)
**Requirements**: AUDIT-01, AUDIT-02, AUDIT-03, AUDIT-04, AUDIT-05, AUDIT-06
**Success Criteria** (what must be TRUE):
  1. Audit log JSON file contains a signed entry for every existing source (all 30 from v1.0) with timestamp, action, category, and actor
  2. Each audit entry has an Ed25519 signature verifiable against the actor's public key using a defined canonical format
  3. Each audit entry links to the previous entry via a previous_hash field forming a hash chain
  4. GET /audit returns audit entries and accepts since, category, and action query parameters for filtering
  5. get_audit_log MCP tool returns audit entries with the same filtering capabilities as the REST endpoint
**Plans**: 2 plans

Plans:
- [ ] 12-01-PLAN.md — Audit module foundation + signing utility + audit_log.json generation
- [ ] 12-02-PLAN.md — Server integration, REST/MCP endpoints, and integration tests

### Phase 13: Identity Linking
**Goal**: Curator and future contributors have verifiable cross-platform identities linking PKARR keys to public profiles
**Depends on**: Phase 12
**Requirements**: IDENT-01, IDENT-02, IDENT-03, IDENT-04, IDENT-05, IDENT-06, IDENT-07
**Success Criteria** (what must be TRUE):
  1. identities.json maps PKARR pubkeys to platform handles (X, Nostr, GitHub) with human/bot type classification and proof URLs for each claim
  2. Bot identities reference a human operator's pubkey, establishing a chain of accountability
  3. GET /identities returns all registered identities, and GET /identities/{pubkey} returns a single identity with all linked platforms
  4. get_identity MCP tool returns identity info for a given pubkey matching the REST endpoint data
  5. John Turner's identity is registered with real, independently verifiable platform proofs
**Plans**: 2 plans

Plans:
- [ ] 13-01-PLAN.md — Identity module foundation (types, loader, error, seed identities.json)
- [ ] 13-02-PLAN.md — Server integration, REST/MCP endpoints, and integration tests

### Phase 14: Community Contributions
**Goal**: Community members can propose source changes and the curator can manage proposals with transparent human/bot vote tracking
**Depends on**: Phase 13
**Requirements**: CONTRIB-01, CONTRIB-02, CONTRIB-03, CONTRIB-04, CONTRIB-05, CONTRIB-06
**Success Criteria** (what must be TRUE):
  1. contributions.json holds proposals with a defined status lifecycle (pending, approved, rejected, withdrawn) supporting add/update/remove actions for sources and categories
  2. Each proposal tracks human and bot votes separately, classified by the voter's identity type from the identity registry
  3. GET /proposals returns proposals filterable by status and category, and GET /proposals/{id} returns a single proposal with full vote details
  4. list_proposals and get_proposal MCP tools expose proposal data to agents with the same filtering and detail as the REST endpoints
**Plans**: TBD

Plans:
- [ ] 14-01: TBD
- [ ] 14-02: TBD
- [ ] 14-03: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 12 → 13 → 14

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1-7 | v1.0 | 17/17 | Complete | 2026-02-03 |
| 8-11 | v1.1 | 6/6 | Complete | 2026-02-09 |
| 12. Audit Log | 2/2 | Complete    | 2026-03-08 | - |
| 13. Identity Linking | v2.0 | 0/2 | Planning complete | - |
| 14. Community Contributions | v2.0 | 0/? | Not started | - |
