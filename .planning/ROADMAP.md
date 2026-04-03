# Roadmap: Three Good Sources (3GS)

## Milestones

- ✅ **v1.0 MVP** — Phases 1-7 (shipped 2026-02-03)
- ✅ **v1.1 Migrate to DigitalOcean + Tech Debt** — Phases 8-11 (shipped 2026-02-09)
- ✅ **v2.0 Community Curation** — Phases 12-14 (shipped 2026-03-08)
- 🚧 **v3.0 Federation Test** — Phases 15-18 (in progress)

## Phases

<details>
<summary>✅ v1.0 MVP (Phases 1-7) — SHIPPED 2026-02-03</summary>

- [x] Phase 1: Foundation (3/3 plans) — completed 2026-02-01
- [x] Phase 2: Registry Schema (2/2 plans) — completed 2026-02-01
- [x] Phase 3: MCP Server Core (3/3 plans) — completed 2026-02-02
- [x] Phase 4: Query Engine (2/2 plans) — completed 2026-02-02
- [x] Phase 5: MCP Tools (2/2 plans) — completed 2026-02-02
- [x] Phase 6: PKARR Identity (2/2 plans) — completed 2026-02-03
- [x] Phase 7: Documentation & Deployment (3/3 plans) — completed 2026-02-03

</details>

<details>
<summary>✅ v1.1 Migrate to DigitalOcean + Tech Debt (Phases 8-11) — SHIPPED 2026-02-09</summary>

- [x] Phase 8: Tech Debt Cleanup (2/2 plans) — completed 2026-02-08
- [x] Phase 9: CORS Hardening (1/1 plan) — completed 2026-02-08
- [x] Phase 10: DigitalOcean Provisioning (1/1 plan) — completed 2026-02-08
- [x] Phase 11: DNS Cutover & Decommission (2/2 plans) — completed 2026-02-09

</details>

<details>
<summary>✅ v2.0 Community Curation (Phases 12-14) — SHIPPED 2026-03-08</summary>

- [x] Phase 12: Audit Log (2/2 plans) — completed 2026-03-08
- [x] Phase 13: Identity Linking (2/2 plans) — completed 2026-03-08
- [x] Phase 14: Community Contributions (2/2 plans) — completed 2026-03-08

</details>

### 🚧 v3.0 Federation Test (In Progress)

**Milestone Goal:** Turn 3GS from a single-node curation server into a federated web-of-trust protocol where curators endorse each other and AI agents query across the network.

- [x] **Phase 15: Federation Foundation** - Endorsement data model, peer types, self-guard, and reqwest runtime dependency (completed 2026-04-03)
- [ ] **Phase 16: Core Federation** - Async refactor, peer cache networking, federated tool, and DRY cleanup
- [ ] **Phase 17: Fork CLI** - `3gs fork` subcommand for scaffolding new nodes
- [ ] **Phase 18: Docker Distribution** - Multi-platform Docker image published to GHCR

## Phase Details

### Phase 15: Federation Foundation
**Goal**: The data model for endorsements and peer registries is defined and safe — the types that everything else builds on exist, are forward-compatible, and self-endorsement is guarded at the type level
**Depends on**: Phase 14
**Requirements**: FED-01, FED-02, FED-03, NET-05
**Success Criteria** (what must be TRUE):
  1. An `Endorsement` struct with pubkey, url, name (optional), and since fields compiles and is used by the registry schema
  2. `PeerRegistry` deserialization tolerates unknown fields so a newer peer's registry does not panic an older node
  3. A peer whose pubkey matches the local node's own pubkey is filtered from the peer cache with a WARN log entry
  4. `reqwest` is listed as a runtime dependency (not dev-only) and the server builds without errors
**Plans**: 2 plans
Plans:
- [x] 15-01-PLAN.md — Endorsement struct, federation types/errors, reqwest runtime dep
- [x] 15-02-PLAN.md — PeerCache with self-endorsement guard and unit tests

### Phase 16: Core Federation
**Goal**: AI agents can query sources across the federated network — the server fetches and caches peer registries on a background schedule, and the `get_federated_sources` tool returns merged results with trust-level tagging
**Depends on**: Phase 15
**Requirements**: NET-01, NET-02, NET-03, NET-04, MCP-01, MCP-02, MCP-03, MCP-04
**Success Criteria** (what must be TRUE):
  1. The server fetches each endorsed peer's `/registry` endpoint on startup and every 5 minutes, with a 10-second per-peer timeout
  2. A peer that fails to respond is skipped; a cache entry older than 1 hour is served with a stale flag
  3. The background refresh task shuts down cleanly when the server receives a termination signal (no hanging tasks)
  4. Calling `get_federated_sources` returns sources from the local registry and all cached peer registries, each tagged with its trust level (local vs. peer pubkey)
  5. Calling `get_endorsements` returns real endorsement data (pubkey, url, name, since) rather than placeholder values
**Plans**: TBD
**UI hint**: no

### Phase 17: Fork CLI
**Goal**: A new curator can scaffold a ready-to-run 3GS node by running a single CLI command without needing any environment variables pre-configured
**Depends on**: Phase 15
**Requirements**: DIST-01, DIST-02
**Success Criteria** (what must be TRUE):
  1. Running `3gs fork` produces a new directory with a generated keypair, skeleton registry JSON, and a populated `.env` file
  2. The `fork` subcommand parses its arguments before `Config::load()` so it does not require `PKARR_SECRET_KEY` or other env vars to be present
  3. The scaffolded node compiles and starts without modification
**Plans**: TBD

### Phase 18: Docker Distribution
**Goal**: Any curator can pull and run a 3GS node on any machine (Intel or ARM) using a single Docker command from GHCR
**Depends on**: Phase 15
**Requirements**: DIST-03
**Success Criteria** (what must be TRUE):
  1. A multi-platform Docker image (`linux/amd64` and `linux/arm64`) is published to GHCR under the project's namespace
  2. `docker pull ghcr.io/[owner]/3goodsources:latest` succeeds and the image runs the server correctly
  3. The GHCR publish is repeatable via CI or documented manual workflow (not a one-off manual push)
**Plans**: TBD

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1-7 | v1.0 | 17/17 | Complete | 2026-02-03 |
| 8-11 | v1.1 | 6/6 | Complete | 2026-02-09 |
| 12-14 | v2.0 | 6/6 | Complete | 2026-03-08 |
| 15. Federation Foundation | v3.0 | 2/2 | Complete    | 2026-04-03 |
| 16. Core Federation | v3.0 | 0/? | Not started | - |
| 17. Fork CLI | v3.0 | 0/? | Not started | - |
| 18. Docker Distribution | v3.0 | 0/? | Not started | - |
