---
phase: 11-dns-cutover-decommission
plan: 01
subsystem: server, infrastructure
tags: [landing-page, domain-cutover, cleanup]
dependency_graph:
  requires:
    - "10-01 (DO app deployed and healthy)"
  provides:
    - "Landing page route at GET /"
    - "DO app spec with custom domains"
  affects:
    - "DNS cutover readiness (11-02)"
tech_stack:
  added:
    - "include_str! macro for compile-time HTML embedding"
  patterns:
    - "Static content serving via axum"
    - "DO App Platform domain configuration"
key_files:
  created: []
  modified:
    - src/server.rs
    - .do/app.yaml
  deleted:
    - render.yaml
decisions:
  - key: "Compile-time HTML embedding"
    rationale: "Use include_str! for landing page (same pattern as registry.json in tests)"
    impact: "No runtime file I/O needed, HTML baked into binary"
  - key: "Domain configuration in DO app spec"
    rationale: "Declare both api.3gs.ai (PRIMARY) and 3gs.ai (ALIAS) for SSL provisioning"
    impact: "DO will auto-provision Let's Encrypt certificates for both domains"
  - key: "Remove render.yaml"
    rationale: "Render decommissioned in Phase 10, config file no longer needed"
    impact: "Clean up obsolete infrastructure config"
metrics:
  duration: 165s
  tasks: 2
  commits: 2
  tests_added: 0
  tests_passing: 78
  completed: 2026-02-08T23:44:47Z
---

# Phase 11 Plan 01: Landing Page & Domain Prep Summary

**One-liner:** Added landing page route at GET / with compile-time HTML embedding and configured DO app spec for custom domain cutover (api.3gs.ai PRIMARY, 3gs.ai ALIAS).

## Objective

Prepare the Rust server and DO app spec for DNS cutover by adding a landing page route and declaring custom domains for SSL provisioning.

## Implementation Summary

### Task 1: Landing Page Route (commit 1a030a8)

**Files modified:** src/server.rs

Added GET / route serving docs/index.html:
- Embedded HTML at compile time with `const LANDING_HTML: &str = include_str!("../docs/index.html")`
- Created `landing_page_endpoint()` handler returning HTML with `text/html; charset=utf-8` content-type
- Added route before other endpoints in `build_router()`

**Verification:**
- cargo build succeeded without errors
- All 78 tests passed (no regressions)
- Manual test: `curl http://localhost:3001/` returned full landing page HTML containing "Three Good Sources"

### Task 2: Domain Configuration & Cleanup (commit d571ca7)

**Files modified:** .do/app.yaml
**Files deleted:** render.yaml

Updated DO app spec with custom domains:
- Added `domains:` section at top level (after `region:`, before `services:`)
- Configured `api.3gs.ai` as PRIMARY domain with TLS 1.2
- Configured `3gs.ai` as ALIAS domain with TLS 1.2
- Removed render.yaml from git (Render already decommissioned in Phase 10)

**Verification:**
- app.yaml contains both domains with correct types and TLS configuration
- render.yaml no longer exists in working tree
- git status shows render.yaml deleted

## Deviations from Plan

None - plan executed exactly as written.

## Test Results

All existing tests pass:
- 43 unit tests (src/lib.rs)
- 43 unit tests (src/main.rs)
- 6 CORS integration tests
- 10 matching integration tests
- 12 MCP integration tests
- 7 registry integration tests

Total: 78 tests passing, 0 failures

## Performance

- Duration: 165 seconds (2m 45s)
- Tasks completed: 2/2
- Commits: 2
- Files modified: 2
- Files deleted: 1

## Success Criteria - ALL MET

- [x] Rust server serves landing page at GET / (compile-time embedded HTML)
- [x] DO app spec declares api.3gs.ai (PRIMARY) and 3gs.ai (ALIAS) with TLS 1.2
- [x] render.yaml removed from git
- [x] All existing tests pass

## Next Steps

Plan 11-02 will handle the actual DNS cutover:
1. Update DNS records to point to DO App Platform
2. Verify both 3gs.ai and api.3gs.ai resolve correctly
3. Verify SSL certificates provisioned
4. Document rollback procedure

## Notes

- Landing page now accessible at both future domains (3gs.ai and api.3gs.ai)
- DO will auto-provision Let's Encrypt SSL when DNS points to their platform
- Render.yaml removal completes Render decommissioning started in Phase 10
- No auth gates encountered (local development only)
- Server uses test registry (tests/fixtures/valid_registry.json) in dev mode

## Self-Check: PASSED

Verified all claimed files and commits exist:

**Files:**
- src/server.rs: FOUND (modified with landing page route)
- .do/app.yaml: FOUND (modified with domains section)
- render.yaml: DELETED (removed from git as expected)

**Commits:**
- 1a030a8: FOUND (Task 1 - landing page route)
- d571ca7: FOUND (Task 2 - domains and cleanup)
