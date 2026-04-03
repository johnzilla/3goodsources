---
phase: 17-fork-cli
plan: 01
subsystem: cli
tags: [pkarr, serde_json, chrono, fork, scaffolding, keypair]

# Dependency graph
requires:
  - phase: 15-federation-foundation
    provides: Endorsement data model, pkarr keypair patterns
provides:
  - fork CLI subcommand that scaffolds a ready-to-run 3GS node in one command
  - src/fork.rs with pub fn run() — decoupled from server logic
  - main.rs fork arg interception before Config::load()
affects: [18-docker-publish, any phase that documents fork usage]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Fork-before-config: early arg check block in async fn main() before Config::load() using std::process::exit() — never enters tokio runtime"
    - "Decoupled CLI module: fork.rs imports only external crates (pkarr, hex, serde_json, chrono) — no crate:: imports"

key-files:
  created:
    - src/fork.rs
  modified:
    - src/main.rs
    - src/lib.rs

key-decisions:
  - "Fork module has no crate:: imports — decoupled from server logic per D-01; external crate deps only"
  - "contributions.json skeleton is '{}' (flat empty HashMap) matching actual loader deserialization, not wrapped proposals object"
  - "mod fork added to main.rs mod declarations (not just lib.rs) because binary crate resolves modules independently"

patterns-established:
  - "Fork-before-config: intercept subcommands at top of main() with raw std::env::args() before any Config::load()"

requirements-completed: [DIST-01, DIST-02]

# Metrics
duration: 1min
completed: 2026-04-03
---

# Phase 17 Plan 01: Fork CLI Summary

**`3gs fork` subcommand that scaffolds a complete 5-file node directory (keypair, registry, .env, identities, contributions, audit log) with a single command and no environment variables required**

## Performance

- **Duration:** 1 min
- **Started:** 2026-04-03T16:11:04Z
- **Completed:** 2026-04-03T16:12:42Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- `src/fork.rs`: `pub fn run()` with manual arg parsing (`--endorse`, `--url`, `--name`, `--output`), pkarr keypair generation, 5-file scaffolding, usage/error messages, and unit tests
- `src/main.rs`: fork arg interception block inserted before `Config::load()` — fork path uses `std::process::exit()`, never enters tokio runtime
- `src/lib.rs`: `pub mod fork` added alphabetically after `pub mod federation`
- Generated node directory contains: `registry.json`, `identities.json`, `contributions.json`, `audit_log.json`, `.env` — all valid formats for the server to load

## Task Commits

Each task was committed atomically:

1. **Task 1: Create fork module with scaffolding logic** - `688cd85` (feat)
2. **Task 2: Wire fork into main.rs and verify end-to-end** - `40a2d39` (feat)

**Plan metadata:** (final commit, see below)

## Files Created/Modified
- `src/fork.rs` — Fork CLI subcommand: arg parsing, keypair gen, 5-file node scaffolding, unit tests
- `src/main.rs` — Fork arg interception block + `mod fork` declaration before server startup
- `src/lib.rs` — Added `pub mod fork` alphabetically after federation

## Decisions Made
- Fork module imports no `crate::` paths — only external crates (`pkarr`, `hex`, `serde_json`, `chrono`) for full decoupling from server logic
- `contributions.json` skeleton is `{}` (flat empty object) matching `HashMap<Uuid, Proposal>` deserialization in `src/contributions/loader.rs` — not the `{ "proposals": {} }` form mentioned in context
- `mod fork` must appear in both `main.rs` and `lib.rs` because the binary crate resolves modules from its own `mod` declarations independently of the library

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Added `mod fork` to main.rs mod declarations**
- **Found during:** Task 2 (wiring fork into main.rs)
- **Issue:** Plan said to add `mod fork` to `lib.rs` only; but `crate::fork::run()` in the binary crate resolved to "unresolved import" because binary modules are independent of library modules
- **Fix:** Added `mod fork;` to the `mod` list in `main.rs` (alphabetical, after `mod federation;`)
- **Files modified:** src/main.rs
- **Verification:** `cargo build` succeeded after adding the declaration
- **Committed in:** 40a2d39 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug — missing mod declaration in binary crate)
**Impact on plan:** Required for compilation. No scope change.

## Issues Encountered
- Binary crate module resolution differs from library crate: `crate::fork` in main.rs needs `mod fork;` declared in main.rs itself, not only in lib.rs. Caught immediately by compiler, fixed in Task 2.

## User Setup Required
None - no external service configuration required. The fork command itself requires no env vars.

## Known Stubs
None — all scaffolded files are real, valid content for server startup.

## Next Phase Readiness
- Fork CLI ships as part of the binary — any `cargo build` produces a working `3gs fork` command
- Phase 18 (Docker publish) can build the image and the fork subcommand will be available in the container
- Curators can now bootstrap a federation node with: `3gs fork --endorse <pubkey> --url <peer_url>`

---
*Phase: 17-fork-cli*
*Completed: 2026-04-03*

## Self-Check: PASSED

- FOUND: src/fork.rs
- FOUND: src/main.rs (modified)
- FOUND: src/lib.rs (modified)
- FOUND commit: 688cd85 (feat: add fork subcommand scaffolding module)
- FOUND commit: 40a2d39 (feat: wire fork subcommand into main.rs)
