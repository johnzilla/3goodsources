# Project Research Summary

**Project:** Three Good Sources (3GS) MCP Server
**Domain:** Curated trust registry served via Model Context Protocol, built in Rust with Pubky integration
**Researched:** 2026-02-01
**Confidence:** MEDIUM

## Executive Summary

Three Good Sources is a Rust MCP server that provides AI agents with access to curated, human-vetted information sources organized by category. The product combines the Model Context Protocol (for agent integration), fuzzy query matching (for intent-based search), and Pubky's decentralized identity system (for cryptographic provenance and trust graphs). The recommended approach is to build with axum 0.8 for HTTP/JSON-RPC handling, implement MCP protocol manually (no mature Rust MCP library exists), and defer Pubky integration to Phase 2 since SDK maturity is unverified.

The critical architectural decision is whether to build "Pubky-first" or "local-first" architecture. Research reveals that Pubky SDK existence and maturity cannot be verified without crates.io access, making this a BLOCKING unknowns that determines the entire phase structure. The recommendation is to start with local-first (registry.json) to validate the core value proposition, then integrate Pubky in Phase 2 once SDK availability is confirmed. This de-risks the project and provides a working MVP within weeks.

Key risks include JSON-RPC protocol violations breaking MCP clients (silently), private key leakage destroying trust, and memory exhaustion on Render's 512MB free tier. All three are preventable through integration testing with real MCP clients, strict key management discipline, and size-limited data loading. The domain is well-understood with high confidence in Rust patterns and MCP specification, but low confidence in Pubky SDK availability requires validation before Phase 2 planning.

## Key Findings

### Recommended Stack

The stack is built around axum 0.8 for HTTP serving with tokio async runtime, manual JSON-RPC implementation using serde/serde_json, and trait-based abstraction for data sources (PubkyLoader vs LocalLoader). **Critical finding:** No mature Rust MCP server library exists, requiring custom JSON-RPC 2.0 implementation. Pubky SDK availability is unverified and represents the project's highest technical risk.

**Core technologies:**
- **axum 0.8 + tokio**: HTTP server with state management — most actively maintained Rust web framework, excellent for JSON-RPC services
- **serde + serde_json**: JSON serialization — industry standard for MCP protocol implementation
- **strsim**: Fuzzy string matching — lightweight Levenshtein distance for query matching
- **thiserror + anyhow**: Error handling — structured errors mapped to JSON-RPC codes
- **Pubky SDK (UNVERIFIED)**: Decentralized identity — requires immediate verification on crates.io before Phase 2

**Critical unknowns:**
- Pubky SDK existence/maturity (HIGH PRIORITY verification needed)
- Axum version (0.7 vs 0.8 — check crates.io)
- MCP Rust library availability (likely doesn't exist, plan for manual implementation)

**Deployment stack:**
- Docker multi-stage build for size optimization (<15MB binary)
- Render free tier (512MB RAM, cold starts after 15min idle)
- Expected performance: <100MB memory, 2-5s cold start

### Expected Features

MCP servers are judged by protocol compliance and query relevance. The MVP requires tools primitive (search_sources), resources primitive (category browsing), and fuzzy query matching that balances precision and recall. Cryptographic provenance via Pubky is the key differentiator but can be deferred to Phase 2.

**Must have (table stakes):**
- MCP tools/resources primitives — core protocol compliance, expected by all MCP clients
- Query matching (fuzzy) — agents need intent-based search, not exact string match
- Source metadata (name, URL, type, why) — minimum information for agent to select sources
- Category organization — agents browse by domain (e.g., "rust-learning", "bitcoin-node-setup")
- JSON-based storage — parseable, version-controllable, human-editable
- Error handling (MCP-compliant) — graceful failures with actionable messages
- Multiple sources per query — return ranked list (top 3), not single result

**Should have (competitive):**
- Cryptographic provenance (Pubky signatures) — proves curator identity, key differentiator
- Intent pattern matching — better than keyword search, curator defines query patterns
- Source ranking (curator explicit 1/2/3) — human curation beats algorithmic ranking
- Curator attribution — shows who vetted each source, builds accountability
- Trust graph integration — leverage Pubky web-of-trust for multi-curator networks

**Defer (v2+):**
- Multi-curator aggregation — requires network effect, defer until adoption
- Scoped trust domains — complexity not needed until multi-curator scenarios
- Update subscription — nice to have after core works
- Offline-first sync — add when deployment complexity justified

**Anti-features (explicitly avoid):**
- Algorithmic ranking — defeats human curation purpose
- User voting/stars — becomes gameable popularity contest
- Automated source discovery — brings in SEO spam
- Search engine integration — defeats anti-SEO stance

### Architecture Approach

The architecture follows a four-layer pattern: HTTP (axum router), MCP Protocol (JSON-RPC parsing/dispatch), Business Logic (registry matching), and Data (RegistryLoader trait). Key decisions include Arc-wrapped immutable registry (no RwLock overhead), trait-based loader abstraction (PubkyLoader vs LocalLoader), and stateless handlers with shared AppState. This structure allows Pubky integration to be plugged in later without refactoring core logic.

**Major components:**

1. **HTTP Layer** — axum router with single POST /mcp endpoint, State extractor for AppState, returns JSON-RPC responses
2. **MCP Protocol Layer** — JSON-RPC 2.0 parsing, method dispatch (initialize, tools/list, tools/call), error mapping to standard codes
3. **Business Logic Layer** — Registry (Arc-wrapped), query matching pipeline (fuzzy search, scoring, top-3 ranking), domain types (Source, MatchResult)
4. **Data Layer** — RegistryLoader trait with LocalLoader (reads registry.json) and PubkyLoader (fetches from homeserver with fallback)
5. **Error Layer** — thiserror-based domain errors, conversion to JSON-RPC error codes with context

**Key patterns:**
- Stateless handlers with Arc<Registry> shared state
- Trait-based abstraction for pluggable data sources
- Immutable registry (load once at startup, no runtime updates)
- Unidirectional data flow: HTTP → JSON-RPC → Domain → Data

**Data flow:**
```
Startup: Config → RegistryLoader (Pubky or Local) → Registry → Arc<Registry> → AppState
Request: POST /mcp → JSON-RPC parse → method dispatch → registry.search() → ranked results → JSON-RPC response
```

### Critical Pitfalls

The top five pitfalls span protocol compliance, security, resource limits, dependency management, and cross-origin access. All are preventable but require discipline during Phase 1 implementation.

1. **JSON-RPC protocol violations** — Missing `jsonrpc: "2.0"` or returning both result+error breaks MCP clients silently. Prevention: Integration test with real MCP client (Claude Desktop), validate every response field.

2. **Private key leakage** — PKARR keys committed to git destroy trust permanently (keys can't be rotated). Prevention: Create .gitignore BEFORE generating keys, store in env vars only, use git-secrets hooks.

3. **Memory exhaustion** — Loading unbounded registry.json crashes server on Render's 512MB limit. Prevention: Enforce 10MB max registry size, lazy load if needed, test with ulimit -v 512000 locally.

4. **Pubky SDK breaking changes** — Pre-1.0 SDK has no semver guarantees, cargo update breaks builds. Prevention: Pin exact versions (=0.1.3), create abstraction layer to isolate SDK, implement local fallback.

5. **CORS misconfiguration** — Missing CORS headers block browser-based MCP clients. Prevention: Add CORS middleware from start, handle OPTIONS preflight, test with browser fetch() not just curl.

**Additional pitfalls requiring phase-specific attention:**
- Fuzzy matching too aggressive (Phase 3) — causing false positives, hurting relevance
- Cold start times >30s (Phase 4) — Render free tier spin-down, requires Docker optimization
- Missing error context (Phase 1) — generic "Internal error" makes debugging impossible
- Registry validation missing (Phase 1) — malformed JSON crashes server at runtime

## Implications for Roadmap

The research reveals a clear three-phase structure: (1) Core MCP with local registry to validate value proposition, (2) Pubky integration once SDK verified, (3) Trust graph and multi-curator features. This ordering de-risks the Pubky unknowns while delivering a working MVP quickly.

### Phase 1: Core MCP Server (Local-First)
**Rationale:** Validate core value (curated sources via MCP) without Pubky dependency. Pubky SDK availability is unverified, making it a blocking risk. Building local-first allows MVP delivery in 1-2 weeks while Pubky SDK verification happens in parallel.

**Delivers:**
- Working MCP server on localhost
- JSON-RPC 2.0 protocol compliance
- tools/call for search_sources
- resources/list for category browsing
- Fuzzy query matching (Levenshtein-based)
- 10 seed categories with 3 real sources each
- registry.json schema with validation

**Addresses (from FEATURES.md):**
- MCP tools/resources primitives (table stakes)
- Query matching with fuzzy search
- Source metadata (name, URL, type, why)
- Category organization
- JSON-based storage
- Error handling (MCP-compliant)

**Avoids (from PITFALLS.md):**
- JSON-RPC protocol violations (Pitfall 1) — integration test with MCP client
- Missing error context (Pitfall 8) — structured errors with details
- Registry validation (Pitfall 9) — fail fast on malformed data
- CORS misconfiguration (Pitfall 5) — add middleware from start

**Stack (from STACK.md):**
- axum 0.8 + tokio (HTTP server)
- serde + serde_json (JSON parsing)
- strsim (fuzzy matching)
- thiserror + anyhow (errors)
- tracing (logging)

**Architecture (from ARCHITECTURE.md):**
- HTTP layer with single POST /mcp endpoint
- MCP Protocol layer (JSON-RPC parsing/dispatch)
- Business Logic layer (Registry, matching pipeline)
- Data layer (LocalLoader only in Phase 1)

**Research flag:** NO — MCP protocol is well-documented, Rust patterns are standard. Skip phase research, proceed directly to requirements definition.

---

### Phase 2: Pubky Integration
**Rationale:** Add cryptographic provenance and decentralized identity after verifying Pubky SDK availability. This phase depends on crates.io verification of pubky/pubky-core/pkarr crates. If SDK is immature or missing, defer to Phase 3 or implement manual PKARR (fallback plan).

**Delivers:**
- PubkyLoader implementation
- Registry fetched from Pubky homeserver
- Fallback to local registry.json on Pubky failure
- Cryptographic signatures on sources
- Curator attribution (Pubky identity)
- Trust scores integrated into ranking

**Addresses (from FEATURES.md):**
- Cryptographic provenance (differentiator)
- Curator attribution (accountability)
- Trust graph foundation (for Phase 3)

**Avoids (from PITFALLS.md):**
- Private key leakage (Pitfall 2) — .gitignore + env vars only
- Pubky SDK breaking changes (Pitfall 4) — pin versions, abstraction layer
- Memory exhaustion (Pitfall 3) — size limits on homeserver fetches

**Stack (from STACK.md):**
- pubky or pubky-core (VERIFY on crates.io first)
- pkarr (VERIFY availability)
- Fallback: ed25519-dalek + bs58 (manual PKARR if no SDK)
- reqwest (HTTP client for homeserver API)

**Architecture (from ARCHITECTURE.md):**
- Implement PubkyLoader (RegistryLoader trait)
- Factory function: create_loader(config) chooses Pubky or Local
- Pubky client wrapper (src/pubky/client.rs)
- Identity management (src/pubky/identity.rs)

**Research flag:** YES — HIGH PRIORITY. Before starting Phase 2, run `/gsd:research-phase` to:
1. Verify Pubky SDK existence (crates.io search: pubky, pubky-core, pkarr)
2. Assess SDK maturity (version >=0.3, recent updates, documentation)
3. Check homeserver API documentation
4. Validate PKARR key operations

**Blocker:** If Pubky SDK doesn't exist or is pre-0.2, implement manual PKARR or defer to Phase 3.

---

### Phase 3: Render Deployment
**Rationale:** Deploy to Render free tier with production optimizations. This phase addresses cold starts, memory limits, and observability.

**Delivers:**
- Multi-stage Docker build (<15MB binary)
- Render deployment (render.yaml)
- Health check endpoint
- PORT environment variable handling
- UptimeRobot keepalive (prevents cold starts)
- Tracing and structured logging
- Size-limited registry loading
- Memory usage monitoring

**Addresses (from STACK.md):**
- Docker configuration for Render
- Binary size optimization (profile.release settings)
- Memory efficiency (<400MB target)

**Avoids (from PITFALLS.md):**
- Cold start times (Pitfall 7) — multi-stage build, keepalive
- Memory exhaustion (Pitfall 3) — 10MB registry limit, testing with ulimit
- Port binding (Pitfall 13) — read PORT env var
- Missing health check (Pitfall 12) — GET /health endpoint

**Research flag:** NO — Render deployment is standard Rust/Docker patterns. Proceed with implementation.

---

### Phase 4: Trust Graph & Multi-Curator
**Rationale:** Enable network effects by aggregating sources from multiple trusted curators. Depends on Pubky trust graph primitives.

**Delivers:**
- Multi-curator registry merging
- Scoped trust domains (expert per category)
- Trust score calculation (endorsement count)
- Conflict detection (disputed sources)
- Update subscription (push notifications)

**Addresses (from FEATURES.md):**
- Multi-curator aggregation (network effect)
- Scoped trust domains (nuanced trust)
- Trust graph integration (full Pubky value)

**Research flag:** YES — MEDIUM PRIORITY. Research trust graph algorithms, endorsement counting, and conflict resolution strategies.

---

### Phase Ordering Rationale

**Why local-first then Pubky:**
- Pubky SDK availability is unverified (CRITICAL UNKNOWN)
- Local-first validates core value (curated sources) independently
- Trait-based architecture makes Pubky a plug-in, not a rewrite
- Delivers working MVP in 1-2 weeks vs 3-4 weeks with Pubky unknowns

**Why Pubky before deployment:**
- Identity/provenance is the key differentiator
- Deploying without Pubky makes product "just another JSON registry"
- Testing Pubky integration on localhost is easier than debugging on Render
- SDK verification can happen during Phase 1 (parallel track)

**Why deployment before trust graph:**
- Trust graph requires multiple curators (network effect)
- Single curator deployment builds foundation for network
- Validate core mechanics before adding complexity
- Early adopters provide feedback on UX before scaling

**Dependency graph:**
```
Phase 1 (Core MCP) → foundational, no dependencies
  ↓
Phase 2 (Pubky) → depends on SDK verification (parallel to Phase 1)
  ↓
Phase 3 (Deployment) → depends on working Pubky integration
  ↓
Phase 4 (Trust Graph) → depends on deployed multi-user system
```

### Research Flags

**Needs phase-specific research:**
- **Phase 2 (Pubky Integration):** CRITICAL — SDK availability, API surface, homeserver protocol, identity management patterns. Run `/gsd:research-phase` BEFORE starting Phase 2 requirements.
- **Phase 4 (Trust Graph):** MEDIUM — Trust calculation algorithms, conflict resolution, endorsement mechanics. Research during Phase 3 completion.

**Standard patterns (skip phase research):**
- **Phase 1 (Core MCP):** Well-documented MCP protocol + standard Rust patterns. Proceed directly to requirements.
- **Phase 3 (Deployment):** Standard Rust/Docker/Render patterns. Proceed directly to requirements.

**Pubky SDK verification (parallel track during Phase 1):**
```bash
# Run immediately, inform Phase 2 planning
cargo search pubky
cargo search pkarr
curl https://crates.io/api/v1/crates/pubky-core
# Check: version, downloads, last_updated, documentation_url
```

**Decision tree for Phase 2:**
```
IF pubky-core exists AND version >= 0.3.0 AND docs exist:
  → Proceed with PubkyLoader implementation
ELSE IF pkarr exists AND version >= 0.2.0:
  → Implement manual Pubky protocol on top of pkarr
ELSE:
  → Defer Pubky to Phase 4, ship local-first MVP
  → Monitor Pubky SDK maturity quarterly
```

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack (Rust ecosystem) | HIGH | axum, tokio, serde are proven; exact versions need verification |
| Stack (Pubky SDK) | LOW | Existence unverified without crates.io access — BLOCKING for Phase 2 |
| Features (MCP protocol) | HIGH | Official spec fetched, clear requirements |
| Features (curation patterns) | MEDIUM | Based on similar systems (awesome lists, package registries) |
| Architecture (Rust patterns) | HIGH | Standard layered architecture, trait abstraction, state management |
| Architecture (Pubky integration) | MEDIUM | General decentralized identity patterns understood, specifics TBD |
| Pitfalls (protocol compliance) | HIGH | JSON-RPC spec is clear, MCP compliance requirements known |
| Pitfalls (deployment) | MEDIUM | Render constraints documented, performance estimates based on typical Rust services |
| Pitfalls (Pubky-specific) | LOW | SDK error model and stability characteristics unknown |

**Overall confidence:** MEDIUM

**Confidence rationale:**
- HIGH confidence in Rust ecosystem, MCP protocol, and deployment patterns (well-documented domains)
- MEDIUM confidence in curation features and Pubky architecture (based on training data and similar systems)
- LOW confidence in Pubky SDK specifics (no crates.io verification, pre-1.0 maturity concerns)

**The Pubky SDK unknowns are the project's primary technical risk.** The local-first phase structure de-risks this by making Pubky a Phase 2 addition rather than a foundational dependency.

### Gaps to Address

**CRITICAL (must resolve before Phase 2):**
- **Pubky SDK availability:** Does `pubky-core` or `pkarr` crate exist? What version? What API surface? This is BLOCKING for Phase 2 architecture decisions.
- **Axum version:** Is 0.8 released (Feb 2026)? Any breaking changes from 0.7? Check crates.io before Phase 1.
- **MCP Rust library:** Does `mcp-rs` or similar exist? Could save implementation time. Search crates.io + GitHub.

**IMPORTANT (resolve during phase planning):**
- **Pubky homeserver protocol:** API documentation, authentication, rate limits (needed for Phase 2 requirements)
- **Trust graph calculations:** Endorsement counting, conflict resolution, scoped trust (needed for Phase 4)
- **Fuzzy matching threshold:** What score cutoff balances precision/recall? (tunable during Phase 1 implementation)

**NICE TO HAVE (can defer or decide during implementation):**
- **Hot-reload support:** Do we need runtime registry updates? (Defer unless user demand)
- **Batch JSON-RPC requests:** Do MCP clients use batching? (Add if spec requires or clients request)
- **Query telemetry:** Should we track query patterns (with privacy)? (Decide based on UX research)

**How to handle unknowns:**

1. **Pubky SDK verification (do first, blocks Phase 2):**
   ```bash
   cargo search pubky
   cargo search pkarr
   # Check version, downloads, docs
   # Run BEFORE Phase 2 requirements definition
   ```

2. **MCP protocol details (validate during Phase 1):**
   - Test with Claude Desktop integration
   - Check MCP GitHub for Rust examples
   - Validate JSON-RPC message formats

3. **Performance characteristics (validate during Phase 3):**
   - Benchmark with realistic registry sizes
   - Test on Render free tier early
   - Monitor memory usage under load

**Contingency plans:**
- If Pubky SDK missing/immature: Ship local-first MVP (Phase 1), monitor SDK quarterly, integrate when mature
- If fuzzy matching inadequate: Add n-gram indexing or semantic search in post-MVP enhancement
- If Render free tier insufficient: Migrate to paid tier ($7/month) or alternative (Railway, Fly.io)

## Sources

### Primary (HIGH confidence)

**Official documentation:**
- Model Context Protocol specification (fetched 2026-02-01 from modelcontextprotocol.io)
- JSON-RPC 2.0 specification (standard protocol definition)
- Rust API guidelines (trait design, error handling, module layout)
- axum documentation (state management, extractors, routing patterns)

**Verified real sources (30 URLs manually checked):**
- All seed category sources (rust-learning, bitcoin-node-setup, etc.) confirmed live and authoritative
- Official project docs, maintained repos, trusted community resources

### Secondary (MEDIUM confidence)

**Ecosystem knowledge (as of January 2025 training cutoff):**
- Rust web framework comparison (axum vs actix-web vs warp)
- Fuzzy matching libraries (strsim, rapidfuzz)
- Error handling patterns (thiserror + anyhow)
- Docker multi-stage builds for Rust

**Deployment platforms:**
- Render free tier constraints (512MB RAM, cold starts, pricing)
- Alternative platforms (Railway, Fly.io) for comparison

**Curation patterns:**
- awesome-lists methodology (community curation, quality criteria)
- Package registry patterns (npm, crates.io, PyPI)
- Web-of-trust concepts (PGP key signing, decentralized identity)

### Tertiary (LOW confidence — needs verification)

**Pubky ecosystem (unverified):**
- Pubky SDK structure (assumed pubky-core or pubky crate, not confirmed)
- PKARR protocol (general understanding from training data, specifics TBD)
- Pubky homeserver API (inferred from decentralized identity patterns)
- Pubky trust graph mechanics (conceptual, implementation details unknown)

**Version-specific details:**
- axum 0.8 availability and changes (training cutoff January 2025, now February 2026)
- Latest Rust crate versions (all versions unverified without crates.io access)

**Performance estimates:**
- Render free tier cold start times (2-5s estimated, needs measurement)
- Memory usage projections (<100MB estimated, depends on registry size)
- Query throughput (10K req/s estimated for Rust/axum, not benchmarked)

**Verification needed before implementation:**
1. crates.io search for: pubky, pubky-core, pkarr, mcp, mcp-rs, axum
2. Pubky GitHub repos for SDK documentation and examples
3. MCP GitHub for Rust implementations or guides
4. Render platform for current free tier limits and performance

---

**Research completed:** 2026-02-01
**Ready for roadmap:** YES

**Next steps for orchestrator:**
1. Verify Pubky SDK on crates.io (parallel to Phase 1 planning)
2. Proceed to requirements definition for Phase 1 (Core MCP)
3. Flag Phase 2 for `/gsd:research-phase` before requirements definition
4. Schedule trust graph research during Phase 3 implementation
