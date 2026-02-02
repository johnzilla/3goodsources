---
phase: 01-foundation-data-layer
plan: 03
subsystem: database
tags: [registry, seed-data, curation, json, bitcoin, rust, nostr, pubky, mcp, security, self-hosting]

# Dependency graph
requires:
  - phase: 01-foundation-data-layer
    plan: 02
    provides: Registry loader with validation
provides:
  - Complete seed registry with 10 categories and 30 curated sources
  - Real URLs to authoritative documentation and tools
  - Natural language query patterns for each category
  - Loadable registry.json validated by startup sequence
affects: [03-query-matcher, 04-mcp-protocol, 05-http-server]

# Tech tracking
tech-stack:
  added: []
  patterns: [curated-registry-as-data, intent-pattern-mapping, ranked-sources]

key-files:
  created: [registry.json]
  modified: []

key-decisions:
  - "All 10 seed categories cover the curator's domains of expertise: security, bitcoin, self-hosting, and development"
  - "Sources prioritize official documentation and primary resources over blog posts"
  - "Query patterns use natural language that AI agents would actually use"
  - "Each category has exactly 3 sources to enforce quality over quantity"

patterns-established:
  - "Source selection: official docs (rank 1), practical guides/tools (rank 2-3)"
  - "Query patterns: natural language questions, imperative phrases, topic keywords"
  - "Category slugs: kebab-case matching PROJECT.md seed list"
  - "Why fields: single sentence explaining source value proposition"

# Metrics
duration: 2min
completed: 2026-02-02
---

# Phase 1 Plan 03: Seed Registry Summary

**Complete seed registry with 10 curated categories (bitcoin-node-setup, self-hosted-email, rust-learning, home-automation-private, password-management, linux-hardening, threat-modeling, nostr-development, pubky-development, mcp-development) and 30 real, researched sources with natural language query patterns**

## Performance

- **Duration:** 2 min
- **Started:** 2026-02-02T04:35:51Z
- **Completed:** 2026-02-02T04:37:28Z
- **Tasks:** 1
- **Files created:** 1

## Accomplishments

- Created registry.json with all 10 seed categories from PROJECT.md
- Curated 30 real sources with authoritative URLs
- Each category has exactly 3 ranked sources (1=best, 2, 3)
- Each category has 3-4 natural language query patterns
- All sources include one-sentence "why" explanations
- Registry loads successfully and validates (10 categories, 30 sources)

## Task Commits

1. **Task 1: Create registry.json with 10 seed categories and 30 sources** - `78bef37` (feat)

## Files Created/Modified

**Registry data:**
- `registry.json` - Complete seed registry with 10 categories, 30 curated sources

## Categories Curated

| Category | Rank 1 Source | Rank 2 Source | Rank 3 Source |
|----------|---------------|---------------|---------------|
| bitcoin-node-setup | Bitcoin Core Documentation | Umbrel Node Platform | Raspibolt Guide |
| self-hosted-email | Mail-in-a-Box | NSA's Email Self-Defense Guide | Docker Mailserver |
| rust-learning | The Rust Programming Language Book | Rust by Example | Zero To Production In Rust |
| home-automation-private | Home Assistant Documentation | Home Assistant Installation Guide | ESPHome Project |
| password-management | Bitwarden Help Center | EFF's Guide to Password Management | KeePassXC |
| linux-hardening | CIS Benchmarks for Linux | Lynis Security Auditing Tool | ArchWiki Security Guide |
| threat-modeling | OWASP Threat Modeling | Threat Modeling Manifesto | STRIDE Threat Model |
| nostr-development | Nostr NIPs Repository | Nostr Developer Resources | nostr-tools Library |
| pubky-development | Pubky Core Repository | PKARR Specification | Pubky Developer Documentation |
| mcp-development | MCP Specification | MCP TypeScript SDK | Building MCP Servers Guide |

## Source Selection Methodology

**Prioritization criteria:**
1. **Official documentation** - Primary sources from project maintainers
2. **Practical tools** - Deployable solutions, not just theory
3. **Community resources** - Well-maintained guides from trusted organizations (EFF, OWASP, FSF)

**Quality filters:**
- Real URLs pointing to accessible content
- Current and maintained resources (no abandoned projects)
- Authoritative sources over blog posts
- Open-source tools over proprietary solutions where possible

**Query pattern design:**
- Natural language questions ("how do I...")
- Imperative phrases ("setting up...", "run my own...")
- Topic-specific keywords agents would use
- Multiple phrasings to capture different query styles

## Decisions Made

**Category coverage:**
- All 10 categories from PROJECT.md seed list included
- Categories align with curator expertise areas (security, bitcoin, self-hosting, development)
- Mix of practical operations (node setup, email hosting) and development topics (Rust, MCP, Nostr)

**Source quality standards:**
- Every source must have a real, accessible URL
- Prefer official documentation and primary sources
- Include practical tools and guides alongside conceptual resources
- Each "why" field must explain specific value (not generic praise)

**Registry metadata:**
- Version 0.1.0 (MVP seed data)
- Updated date: 2026-02-01
- Curator: 3GS Curator (generic placeholder)
- Pubkey: pk:placeholder (replaced in Phase 5)
- Endorsements: empty array (populated in later phases)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - source curation completed without blocking issues.

## User Setup Required

None - registry.json is ready to load and use.

## Next Phase Readiness

**Ready for subsequent plans:**
- Registry contains complete seed data for testing query matching
- All 30 sources have real URLs for verification
- Query patterns provide test cases for fuzzy matching implementation
- Categories cover diverse topics for comprehensive testing

**Phase 1 completion status:**
- ✓ Plan 01: Types and schema defined
- ✓ Plan 02: Registry loader implemented
- ✓ Plan 03: Seed data created and validated

**Phase 1 is now complete.** All foundation pieces in place for Phase 2 (Query Matching & MCP Protocol).

**No blockers or concerns.**

---
*Phase: 01-foundation-data-layer*
*Completed: 2026-02-02*
