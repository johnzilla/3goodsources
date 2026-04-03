---
phase: 16-core-federation
plan: 01
subsystem: federation
tags: [reqwest, http-client, peer-cache, federation, rust, tokio]

# Dependency graph
requires:
  - phase: 15-federation-foundation
    provides: PeerCache struct, CachedPeer, PeerStatus, PeerRegistry types, and RwLock<HashMap> storage

provides:
  - PeerCache with reqwest::Client field (10s timeout)
  - fetch_peer() async method — fetches /registry, sets Fresh/Stale/Unreachable
  - refresh_all() async method — iterates peers sequentially, fetches each
  - get_all_cached() returning Vec<CachedPeerSnapshot>
  - CachedPeerSnapshot struct for read-only snapshots with stale flag

affects: [16-02, 16-03, any phase using PeerCache for federated queries]

# Tech tracking
tech-stack:
  added: [reqwest runtime usage in federation (was already in Cargo.toml)]
  patterns: [release-read-lock before async IO, acquire-write-lock after response]

key-files:
  created: []
  modified:
    - src/federation/cache.rs
    - src/federation/mod.rs

key-decisions:
  - "Release read lock before HTTP call to avoid holding lock across await point — correct async RwLock usage"
  - "stale flag in CachedPeerSnapshot = (status == Stale), not (status != Fresh) — Unreachable is not stale"
  - "On failure with no prior success: status = Unreachable; on failure with registry present and >1hr: status = Stale"

patterns-established:
  - "Pattern: Acquire read lock to get URL, release, make HTTP call, acquire write lock to update — prevents lock-across-await"
  - "Pattern: CachedPeerSnapshot as a cloned value type for safe hand-off without lock leakage"

requirements-completed: [NET-01, NET-02, NET-03]

# Metrics
duration: 2min
completed: 2026-04-02
---

# Phase 16 Plan 01: Core Federation - HTTP Peer Cache Summary

**PeerCache upgraded from data structure to networked client: reqwest::Client with 10s timeout, fetch_peer()/refresh_all()/get_all_cached() async methods, 1-hour staleness threshold, and 10 passing tests**

## Performance

- **Duration:** ~2 min
- **Started:** 2026-04-02T00:41:40Z
- **Completed:** 2026-04-02T00:43:25Z
- **Tasks:** 2 (combined into single commit — same file)
- **Files modified:** 2

## Accomplishments

- Added `reqwest::Client` with 10-second timeout to PeerCache struct
- Implemented `fetch_peer()` with proper async lock release before HTTP, Fresh/Stale/Unreachable state machine
- Implemented `refresh_all()` with sequential peer iteration and info logging
- Implemented `get_all_cached()` returning cloned `CachedPeerSnapshot` values (no lock exposure)
- Added `CachedPeerSnapshot` struct exported from `federation` module
- Added 5 new unit tests covering empty cache, snapshot field correctness, stale flag, and no-panic refresh
- All 10 tests pass (5 original + 5 new)

## Task Commits

1. **Tasks 1 + 2: Add reqwest::Client and fetch/refresh/snapshot methods + unit tests** - `0699223` (feat)

**Plan metadata:** (pending docs commit)

## Files Created/Modified

- `src/federation/cache.rs` - Added client field, fetch_peer, refresh_all, get_all_cached, CachedPeerSnapshot, and 5 new tests
- `src/federation/mod.rs` - Added CachedPeerSnapshot to pub use exports

## Decisions Made

- Release read lock before HTTP call to avoid holding RwLock across an await point — correct async pattern
- `stale` flag = `(status == PeerStatus::Stale)` — Unreachable is not stale (peer was never reached, not gone stale)
- On first failure with no prior success: status stays `Unreachable` (no registry data)
- On failure with existing registry + >1hr since last success: status becomes `Stale`

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- PeerCache now has full HTTP networking capability
- Phase 16-02 can wire background refresh loop using `refresh_all()`
- Phase 16-03 can use `get_all_cached()` for federated source queries
- All existing 77 unit tests still pass

## Self-Check: PASSED

- `src/federation/cache.rs` exists with all required methods
- `src/federation/mod.rs` exports CachedPeerSnapshot
- Commit `0699223` exists in git log
- `cargo build` succeeds with no errors
- `cargo test --lib federation::cache` — 10/10 tests pass

---
*Phase: 16-core-federation*
*Completed: 2026-04-02*
