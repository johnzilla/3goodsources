# Project Milestones: Three Good Sources (3GS)

## v2.0 Community Curation (Shipped: 2026-03-08)

**Delivered:** Community curation infrastructure — append-only audit log with Ed25519 signed hash-chained entries, cross-platform identity linking with proof URLs, and community contribution proposals with human/bot vote separation. All served via REST + MCP endpoints, read-only server with curator-managed JSON files.

**Phases completed:** 12-14 (6 plans total)

**Key accomplishments:**
- Append-only audit log with Ed25519 signed, SHA-256 hash-chained entries for all 30 existing sources
- Cross-platform identity linking (PKARR ↔ X/Nostr/GitHub) with independently verifiable proof URLs
- Community contribution proposals with status lifecycle and human/bot vote separation
- 6 new REST endpoints and 4 new MCP tools (8 total)
- 144 tests passing (77 unit + 67 integration), up from 78 at v1.1
- All 19 v2.0 requirements satisfied (AUDIT-01..06, IDENT-01..07, CONTRIB-01..06)

**Stats:**
- 51 files modified
- 6,029 lines of Rust (up from 2,179 at v1.1)
- 3 phases, 6 plans
- 1 day execution (2026-03-08)

**Git range:** `a38978c` (docs(12): research) → `0d6d731` (docs(phase-14): complete)

**Tech debt carried forward:** curve25519-dalek git patch dependency (DEPS-01/DEPS-02)

---

## v1 MVP (Shipped: 2026-02-03)

**Delivered:** A working MCP server in Rust that serves curated, cryptographically-signed source recommendations for AI agents via HTTP POST JSON-RPC, with 10 seed categories, fuzzy query matching, PKARR identity, Docker deployment, and a landing page at 3gs.ai.

**Phases completed:** 1-7 (17 plans total)

**Key accomplishments:**
- Built complete MCP JSON-RPC 2.0 server with four tools (get_sources, list_categories, get_provenance, get_endorsements)
- Implemented fuzzy query matching engine with Levenshtein distance + keyword boosting (19 unit tests)
- Curated 30 real sources across 10 categories (bitcoin, rust, security, privacy, self-hosting, nostr, pubky, mcp)
- Integrated PKARR Ed25519 keypair for cryptographic curator identity
- Created Docker multi-stage build with Render deployment config
- Shipped landing page at 3gs.ai with MCP client configuration
- 72 tests passing (43 unit + 29 integration) with full E2E coverage

**Stats:**
- 112 files created/modified
- 3,016 lines of Rust
- 7 phases, 17 plans
- 3 days from start to ship (2026-02-01 → 2026-02-03)

**Git range:** `40883de` (Initial commit) → `4db7bf5` (docs(07))

**What's next:** Production deployment to Render, Pubky homeserver integration, additional transports (stdio, SSE)

---

## v1.1 Migrate to DigitalOcean + Tech Debt (Shipped: 2026-02-09)

**Delivered:** Migrated live deployment from Render to DigitalOcean App Platform with custom domains (3gs.ai, api.3gs.ai), hardened CORS, cleaned dead code, and verified git history clean of secrets.

**Phases completed:** 8-11 (6 plans total)

**Key accomplishments:**
- Cleaned codebase: removed McpError enum, unused re-exports, zero clippy warnings (7 atomic commits)
- Hardened CORS with explicit origin allowlist for 3gs.ai and api.3gs.ai (6 integration tests)
- Deployed to DigitalOcean App Platform via Ansible playbook with Docker build
- Completed DNS cutover: 3gs.ai (PRIMARY) and api.3gs.ai (ALIAS) with Let's Encrypt SSL
- Added landing page route served from Rust via compile-time HTML embedding (include_str!)
- Verified git history clean of all secrets (4 comprehensive scans)

**Stats:**
- 16 files modified
- 2,179 lines of Rust (down from 3,016 after dead code removal)
- 4 phases, 6 plans
- 2 days from start to ship (2026-02-08 → 2026-02-09)

**Git range:** `73956c3` (refactor(matcher)) → `d3b2e82` (docs(v1.1))

**Tech debt carried forward:** curve25519-dalek git patch dependency (DEPS-01/DEPS-02)

**What's next:** TBD — next milestone planning

---

