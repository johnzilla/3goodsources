---
phase: 14-community-contributions
plan: 02
subsystem: api
tags: [contributions, proposals, rest, mcp, axum, uuid, integration-tests]

requires:
  - phase: 14-community-contributions
    provides: Contribution types, loader, and seed data from Plan 01

provides:
  - REST endpoints for proposals (GET /proposals, GET /proposals/{id})
  - MCP tools for proposals (list_proposals, get_proposal)
  - Integration test suite for contributions endpoints
  - Full server startup with contributions module

affects: []

tech-stack:
  added: []
  patterns: [proposal endpoints replicate identity endpoint pattern, MCP tools replicate identity tool pattern]

key-files:
  created:
    - tests/integration_contributions.rs
  modified:
    - src/server.rs
    - src/mcp/handler.rs
    - src/mcp/tools.rs
    - src/main.rs
    - tests/common/mod.rs
    - tests/integration_mcp.rs

key-decisions:
  - "Proposals REST endpoints follow identity endpoint pattern (list + detail by ID)"
  - "Lenient status filtering: invalid status returns empty array, not error"
  - "Proposal detail endpoint injects id field into JSON response (id is HashMap key, not in struct)"
  - "MCP tools use human-readable text output matching audit/identity tool patterns"

patterns-established:
  - "Data module wiring pattern complete: types -> loader -> AppState + McpHandler -> REST + MCP -> integration tests"

requirements-completed: [CONTRIB-04, CONTRIB-05, CONTRIB-06]

duration: 4min
completed: 2026-03-08
---

# Phase 14 Plan 02: Server Wiring for Contributions Summary

**REST endpoints and MCP tools for community proposals with status/category filtering, detail views with votes, and 13 integration tests**

## Performance

- **Duration:** 4 min (266s)
- **Started:** 2026-03-08T17:44:37Z
- **Completed:** 2026-03-08T17:49:03Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- Two REST endpoints: GET /proposals (filtered list) and GET /proposals/{id} (detail with votes)
- Two MCP tools: list_proposals and get_proposal with same filtering as REST
- 13 integration tests covering REST filtering, 404 handling, MCP tool output, and tools/list count
- Full server startup with contributions module (144 total tests passing)

## Task Commits

Each task was committed atomically:

1. **Task 1: Wire contributions into server, MCP handler, and MCP tools** - `13eb3a3` (feat)
2. **Task 2: Add integration tests for contributions REST and MCP endpoints** - `0d6c6fa` (test)

## Files Created/Modified
- `src/server.rs` - Added proposals field to AppState, proposals_endpoint and proposal_by_id_endpoint handlers, 2 new routes
- `src/mcp/handler.rs` - Added proposals field to McpHandler, updated new() and handle_tools_call
- `src/mcp/tools.rs` - Added ListProposalsParams, GetProposalParams, tool_list_proposals, tool_get_proposal, updated get_tools_list (8 tools) and handle_tool_call
- `src/main.rs` - Added contributions module, load contributions at startup, pass to McpHandler and AppState
- `tests/common/mod.rs` - Load contributions.json in spawn_test_server
- `tests/integration_contributions.rs` - 13 integration tests for REST and MCP endpoints
- `tests/integration_mcp.rs` - Updated tool count assertion from 6 to 8

## Decisions Made
- Proposals REST endpoints follow identity endpoint pattern (list + detail by ID)
- Lenient status filtering: invalid status returns empty array, not error
- Proposal detail endpoint injects id field into JSON response (id is HashMap key, not in struct)
- MCP tools use human-readable text output matching audit/identity tool patterns

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Updated integration_mcp.rs tool count assertion**
- **Found during:** Task 2
- **Issue:** Existing test_tools_list_returns_six_tools in integration_mcp.rs expected 6 tools, now 8
- **Fix:** Updated assertion from 6 to 8
- **Files modified:** tests/integration_mcp.rs
- **Verification:** Full test suite passes (144 tests)
- **Committed in:** 0d6c6fa (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Necessary update to existing test for new tool count. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 14 (Community Contributions) is now complete
- All v2.0 Community Curation milestone features implemented (phases 12-14)
- 8 MCP tools, 4 REST modules (registry, audit, identities, proposals)

---
*Phase: 14-community-contributions*
*Completed: 2026-03-08*
