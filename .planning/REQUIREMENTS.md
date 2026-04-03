# Requirements: Three Good Sources (3GS)

**Defined:** 2026-04-02
**Core Value:** Agents get curated, high-quality sources instead of SEO-gamed search results — three good sources per topic, human-vetted, cryptographically signed, served via open protocol.

## v3.0 Requirements

Requirements for Federation Test milestone. Each maps to roadmap phases.

### Federation Data Model

- [x] **FED-01**: Endorsement struct populated with pubkey, url, name (optional), and since fields
- [x] **FED-02**: PeerRegistry lax deserialization types without deny_unknown_fields for forward compatibility
- [x] **FED-03**: Self-endorsement guard filters own pubkey from peer cache with WARN log

### Federation Networking

- [ ] **NET-01**: Peer cache fetches and caches endorsed peer registries via HTTP with 10s timeout
- [ ] **NET-02**: Background tokio task refreshes peer cache every 5 minutes
- [ ] **NET-03**: Stale cache (>1hr without success) served with stale flag, unreachable peers skipped
- [ ] **NET-04**: Graceful shutdown for background refresh task via cancellation signal
- [x] **NET-05**: reqwest moved from dev-dependency to runtime dependency

### MCP Tools

- [ ] **MCP-01**: `get_federated_sources` tool queries local + cached peer registries with trust-level tagging
- [ ] **MCP-02**: `get_endorsements` tool updated to show real endorsement data (pubkey, url, name, since)
- [ ] **MCP-03**: DRY `tool_response()` helper refactored across all tools
- [ ] **MCP-04**: Async refactor of `handle_json()` and `handle_tool_call()` for RwLock reads

### Distribution

- [ ] **DIST-01**: `3gs fork` CLI subcommand scaffolds a new node with keypair, skeleton files, and .env
- [ ] **DIST-02**: Fork parses args before Config::load() to avoid requiring env vars
- [ ] **DIST-03**: Docker image published to GHCR (multi-platform linux/amd64 + linux/arm64)

## Future Requirements

Deferred to future release. Tracked but not in current roadmap.

### Federation Advanced

- **FEDA-01**: Multi-hop trust traversal (gossip protocol)
- **FEDA-02**: Automated category import from endorsed peers
- **FEDA-03**: Rate limiting on GET /registry endpoint
- **FEDA-04**: GUI for managing endorsements

### Write API

- **WAPI-01**: Submission API for identity claims
- **WAPI-02**: Submission API for contribution proposals

## Out of Scope

| Feature | Reason |
|---------|--------|
| Token/incentive mechanism | Anti-token thesis — Bitcoin-only settlement |
| Full gossip protocol | Weekend scope constraint — one-hop only for demand test |
| Automated identity verification | Manual curation is the point for v3.0 |
| Blog post / outreach content | User's responsibility, not code |
| Rate limiting | Captured as future req FEDA-03 — self-endorsement guard sufficient for v3.0 |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| FED-01 | Phase 15 | Complete |
| FED-02 | Phase 15 | Complete |
| FED-03 | Phase 15 | Complete |
| NET-05 | Phase 15 | Complete |
| NET-01 | Phase 16 | Pending |
| NET-02 | Phase 16 | Pending |
| NET-03 | Phase 16 | Pending |
| NET-04 | Phase 16 | Pending |
| MCP-01 | Phase 16 | Pending |
| MCP-02 | Phase 16 | Pending |
| MCP-03 | Phase 16 | Pending |
| MCP-04 | Phase 16 | Pending |
| DIST-01 | Phase 17 | Pending |
| DIST-02 | Phase 17 | Pending |
| DIST-03 | Phase 18 | Pending |

**Coverage:**
- v3.0 requirements: 15 total
- Mapped to phases: 15
- Unmapped: 0

---
*Requirements defined: 2026-04-02*
*Last updated: 2026-04-02 — traceability mapped to Phases 15-18*
