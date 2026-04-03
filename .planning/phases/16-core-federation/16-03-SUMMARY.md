---
phase: 16-core-federation
plan: 03
subsystem: federation
tags: [rust, tokio, axum, mcp, federation, peer-cache, background-task, shutdown]

# Dependency graph
requires:
  - phase: 16-core-federation/16-01
    provides: PeerCache with fetch_peer, refresh_all, get_all_cached, CachedPeerSnapshot
  - phase: 16-core-federation/16-02
    provides: async McpHandler, handle_tool_call async, tool_response DRY helper
provides:
  - PeerCache wired into AppState and McpHandler via Arc
  - Background refresh loop (5-min interval) with tokio::select + watch::channel shutdown
  - get_federated_sources MCP tool querying local + peer registries with trust tagging
  - Tool count is 9 (was 8)
  - Graceful server shutdown signals background task to stop cleanly
affects: [17-fork-cli, 18-docker-publish, integration-tests]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "tokio::sync::watch channel for broadcast shutdown signals to background tasks"
    - "tokio::select! for cooperative shutdown in background refresh loops"
    - "Build temporary Registry from PeerRegistry to reuse match_query across peer data"
    - "PeerCache shared via Arc<PeerCache> across AppState and McpHandler"

key-files:
  created: []
  modified:
    - src/main.rs
    - src/server.rs
    - src/mcp/handler.rs
    - src/mcp/tools.rs
    - tests/common/mod.rs
    - tests/integration_contributions.rs
    - tests/integration_mcp.rs

key-decisions:
  - "tokio::sync::watch channel chosen for shutdown signal — broadcast to multiple receivers, cheap clone, idiomatic for this pattern"
  - "Initial refresh_all() runs before server binds so first request has fresh peer data"
  - "Temporary Registry constructed from PeerRegistry fields to reuse existing match_query — avoids duplicating matching logic"
  - "Unreachable peers (never fetched) are skipped; Stale peers (fetched but old) are included with [STALE] tag"

patterns-established:
  - "Background task shutdown: watch::channel(false) + select! on changed() + await handle after serve"
  - "Peer registry adapter: construct crate::registry::types::Registry from PeerRegistry to reuse matcher"

requirements-completed: [NET-04, MCP-01, NET-02]

# Metrics
duration: 5min
completed: 2026-04-03
---

# Phase 16 Plan 03: Federation Integration Summary

**PeerCache integrated into server with 5-minute background refresh, watch-channel shutdown, and get_federated_sources MCP tool returning trust-tagged local+peer results**

## Performance

- **Duration:** 5 min
- **Started:** 2026-04-03T14:07:37Z
- **Completed:** 2026-04-03T14:11:24Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- PeerCache created on startup from registry endorsements, wired into AppState and McpHandler via Arc
- Background refresh loop spawns on startup, runs every 5 minutes, shuts down cleanly after axum::serve exits
- get_federated_sources tool queries local registry (trust: direct) and all non-unreachable peers (trust: endorsed), with [STALE] flag for stale peers
- Tool count updated from 8 to 9 across unit tests, integration tests, and tools/list response
- All 154 tests pass (87 unit + 67 integration)

## Task Commits

Each task was committed atomically:

1. **Task 1: Wire PeerCache into AppState, McpHandler, spawn refresh loop** - `9d76b79` (feat)
2. **Task 2: Implement get_federated_sources tool, update tool count to 9** - `41ef593` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified
- `src/main.rs` - Added mod federation, PeerCache creation, watch channel, refresh loop spawn, shutdown signal
- `src/server.rs` - Added peer_cache: Arc<PeerCache> to AppState
- `src/mcp/handler.rs` - Added peer_cache field and parameter to McpHandler, updated test_handler()
- `src/mcp/tools.rs` - Added GetFederatedSourcesParams, updated handle_tool_call signature, implemented tool_get_federated_sources, updated get_tools_list to 9 tools
- `tests/common/mod.rs` - Added PeerCache creation and passing to McpHandler and AppState (deviation fix)
- `tests/integration_contributions.rs` - Updated test_mcp_tools_list_returns_eight to nine (deviation fix)
- `tests/integration_mcp.rs` - Updated tools count assertion from 8 to 9 (deviation fix)

## Decisions Made
- tokio::sync::watch channel for shutdown signal — broadcast semantics, cheap clone, idiomatic for this pattern
- Initial refresh_all() runs before server binds so first MCP request has fresh peer data available
- Temporary Registry constructed from PeerRegistry fields to reuse existing match_query — avoids duplicating matching logic
- Unreachable peers (never fetched) are silently skipped; Stale peers are included with [STALE] tag per plan spec

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated integration test common module and two test files for new AppState/McpHandler signatures**
- **Found during:** Task 2 verification (cargo test)
- **Issue:** tests/common/mod.rs, tests/integration_contributions.rs, and tests/integration_mcp.rs all referenced the old 6-parameter McpHandler::new and 8-field AppState, causing compile errors and assertion failures
- **Fix:** Added PeerCache import and construction to common/mod.rs; updated McpHandler::new and AppState struct calls; updated tool count assertions from 8 to 9 in both integration test files
- **Files modified:** tests/common/mod.rs, tests/integration_contributions.rs, tests/integration_mcp.rs
- **Verification:** cargo test passes 154 tests (0 failed)
- **Committed in:** 41ef593 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking — integration test updates required by interface changes)
**Impact on plan:** Necessary for test suite to compile and pass. No scope creep.

## Issues Encountered
None beyond the expected integration test updates after adding peer_cache to public interfaces.

## Known Stubs
None — get_federated_sources fully wired to real PeerCache.get_all_cached(). No placeholder data.

## User Setup Required
None — no external service configuration required.

## Next Phase Readiness
- Federation integration is complete: PeerCache, background refresh, get_federated_sources tool all operational
- Phase 17 (fork CLI) can proceed — depends only on Phase 15 data model
- Phase 18 (Docker publish) can proceed — depends only on Phase 15
- All 5 phase-level success criteria from 16-CONTEXT.md are met

---
*Phase: 16-core-federation*
*Completed: 2026-04-03*
