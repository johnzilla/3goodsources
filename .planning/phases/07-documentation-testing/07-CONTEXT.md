# Phase 7: Documentation & Testing - Context

**Gathered:** 2026-02-03
**Status:** Ready for planning

<domain>
## Phase Boundary

Complete project documentation (README.md, docs/SCHEMA.md, docs/METHODOLOGY.md, docs/PUBKY.md) and comprehensive test suite (query matching, MCP protocol, registry loading). Documentation explains what 3GS is, how to use it, how sources are curated, and how identity/verification works. Tests validate correctness with real data and real system calls.

</domain>

<decisions>
## Implementation Decisions

### Documentation audience & tone
- Primary audience: broader tech audience — explain the concept first (why curated sources matter), then technical setup
- Comprehensive README — covers concept, quickstart, all endpoints, MCP tools, registry format; docs/ files for deep dives only
- Professional & neutral tone — clear, factual, standard open-source project style
- Include architecture diagram (ASCII or mermaid) showing request flow: Agent -> HTTP POST -> MCP -> Registry -> Sources

### Test coverage & style
- Core paths + edge cases: happy paths plus malformed JSON, empty queries, missing registry, threshold boundaries, unknown tools
- Unit tests inline (#[cfg(test)] mod tests) AND integration tests in tests/ directory
- **No mocks, no stubs, no fabricated returns** — every test uses real data and real method calls
- Prefer integration tests over isolated units with mocked boundaries
- If a dependency is unavailable, skip the test with a clear message rather than faking it
- Tests validate actual registry.json seed data — assert specific categories exist (rust-learning, bitcoin-node-setup), verify source counts, check real URLs
- Integration tests spawn a real HTTP server on a random port and make real HTTP requests with reqwest — true end-to-end

### Source methodology depth
- Fully transparent — document exact criteria: why 3 sources, what makes a source "good", ranking methodology, rejection reasons, bias acknowledgment
- Include a worked example walking through how a specific category was curated (e.g., rust-learning) showing why each source was chosen and what was rejected
- Document the query matching algorithm in METHODOLOGY.md: normalization pipeline, fuzzy scoring (Levenshtein), keyword boosting, thresholds
- Include maintenance process: how new categories get added, how sources get updated, review cadence, community contribution path

### Pubky/verification docs
- Mostly practical — focus on what works now: PKARR keypair, how to verify provenance, what the pubkey means. Brief mention of future federation.
- Step-by-step verification guide: get pubkey from /health, call get_provenance, verify the key matches. Concrete commands with curl examples.
- Brief PKARR/Pubky primer for unfamiliar readers — what PKARR is, how it relates to Pubky, why decentralized identity matters for source curation
- Document endorsement concept — explain what endorsements will be (other curators vouching for sources), note it's scaffolded but empty in v1

### Claude's Discretion
- README section ordering and navigation structure
- Exact mermaid diagram style/complexity
- SCHEMA.md depth and formatting approach
- Test helper organization and shared fixtures
- How to handle test parallelism with real HTTP servers (random ports)

</decisions>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 07-documentation-testing*
*Context gathered: 2026-02-03*
