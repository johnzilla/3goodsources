---
phase: 05-identity-and-provenance
plan: 02
subsystem: identity
tags: [pkarr, pubkey, z-base-32, provenance, health-check]

# Dependency graph
requires:
  - phase: 05-01
    provides: generate_or_load_keypair function, PubkyError types, pkarr dependency
provides:
  - Server AppState with PKARR public key
  - Health endpoint exposing server pubkey in z-base-32 format
  - get_provenance tool returning live server identity
  - Main.rs startup sequence with keypair generation
  - Ephemeral vs persistent identity based on PKARR_SECRET_KEY env var
affects: [06-infrastructure]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "PublicKey stored in AppState (Copy type, no Arc needed)"
    - "Pubkey z-base-32 string passed through MCP layer as String"
    - "Keypair generation happens after logging init but before MCP handler creation"

key-files:
  created: []
  modified:
    - src/server.rs
    - src/main.rs
    - src/mcp/handler.rs
    - src/mcp/tools.rs

key-decisions:
  - "Store PublicKey directly in AppState (Copy type)"
  - "Pass pubkey as z-base-32 String through MCP layer to avoid pkarr type dependency"
  - "Replace registry.curator.pubkey with live server pubkey in get_provenance"
  - "Keypair generation in startup sequence between MatchConfig and Registry loading"

patterns-established:
  - "State extractor pattern: health_endpoint uses State<Arc<AppState>> to access pubkey"
  - "Pubkey conversion at boundaries: PublicKey -> String (z32) at MCP layer boundary"

# Metrics
duration: 6min
completed: 2026-02-03
---

# Phase 05 Plan 02: Server Identity Integration Summary

**PKARR public key wired through server stack: AppState storage, health endpoint exposure, get_provenance tool integration, with ephemeral/persistent identity via PKARR_SECRET_KEY**

## Performance

- **Duration:** 6 min
- **Started:** 2026-02-03T01:23:43Z
- **Completed:** 2026-02-03T01:30:02Z
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments

- Health endpoint returns server pubkey in z-base-32 format
- get_provenance tool shows live server identity (not empty registry metadata)
- Server logs pubkey at startup with ephemeral keypair warning when appropriate
- Deterministic identity when PKARR_SECRET_KEY environment variable is set

## Task Commits

Each task was committed atomically:

1. **Task 1: Add pubkey to AppState and update health endpoint** - `f23726d` (feat)
2. **Task 2: Wire pubkey through McpHandler and get_provenance tool** - `48a5af8` (feat)
3. **Task 3: Wire keypair generation into main.rs startup** - `14e7057` (feat)

## Files Created/Modified

- `src/server.rs` - Added pkarr::PublicKey import, pubkey field to AppState, State extractor in health_endpoint, pubkey in health JSON response
- `src/main.rs` - Keypair generation in startup sequence, pubkey_z32 passed to McpHandler::new, public_key stored in AppState
- `src/mcp/handler.rs` - Added pubkey_z32: String field to McpHandler, updated constructor signature, passed to handle_tool_call
- `src/mcp/tools.rs` - Updated handle_tool_call and tool_get_provenance signatures to accept pubkey_z32, replaced registry.curator.pubkey fallback with live pubkey

## Decisions Made

**PublicKey storage strategy:** Store PublicKey directly in AppState (not Arc<PublicKey>) because PublicKey is Copy. Simple and efficient.

**MCP layer isolation:** Pass pubkey as z-base-32 String through MCP layer instead of importing pkarr types into MCP module. Keeps MCP protocol layer independent of cryptography implementation details.

**Startup sequence order:** Generate keypair after logging initialization (so warnings are visible) but before Registry loading and McpHandler construction. Ensures identity is available when building dependent components.

**Live vs registry pubkey:** Replace registry.curator.pubkey (empty string in v1 registry.json) with live server pubkey in get_provenance tool. The server identity is what matters for verification, not static metadata.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## Next Phase Readiness

Identity layer complete. Server has cryptographic identity that is:
- Visible in health checks
- Exposed via get_provenance tool
- Logged at startup
- Ephemeral by default, persistent with PKARR_SECRET_KEY

Ready for Phase 6 (Infrastructure) deployment planning.

**For future signing implementation:** The keypair is generated in main.rs but not stored in AppState. When Phase 7 implements registry signing, will need to either:
1. Store full keypair in AppState (requires making it thread-safe), OR
2. Re-derive keypair from PKARR_SECRET_KEY when needed for signing

Recommend approach 2 (re-derive) - avoids keeping secret key material in memory permanently.

---
*Phase: 05-identity-and-provenance*
*Completed: 2026-02-03*
