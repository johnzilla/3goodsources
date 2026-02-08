---
phase: 08-tech-debt-cleanup
plan: 01
subsystem: codebase-quality
tags: [cleanup, dead-code, compiler-warnings, tech-debt]
dependency_graph:
  requires: []
  provides: [warning-free-build, clean-01-fulfilled]
  affects: [matcher, registry, mcp, all-modules]
tech_stack:
  added: []
  patterns: [atomic-commits, dead-code-analysis]
key_files:
  created: []
  modified:
    - src/matcher/mod.rs
    - src/matcher/scorer.rs
    - src/registry/mod.rs
    - src/registry/error.rs
    - src/mcp/types.rs
    - src/mcp/mod.rs
  deleted:
    - src/mcp/error.rs
decisions:
  - "Preserve score field with #[allow(dead_code)] - used in tests and valuable for debugging"
  - "Preserve InitializeParams fields with #[allow(dead_code)] - MCP protocol spec compliance"
  - "Remove unused re-exports from mod.rs files - only export what's actually imported externally"
  - "Delete entire McpError enum and error.rs file - completely unused"
metrics:
  duration: 250s
  tasks_completed: 2
  files_modified: 6
  files_deleted: 1
  commits: 7
  warnings_fixed: 7
  completed_date: 2026-02-08
---

# Phase 08 Plan 01: Dead Code Removal Summary

**One-liner:** Removed all unused imports, fields, functions, and the McpError enum achieving zero clippy warnings with atomic commits per fix.

## Objective Achieved

Satisfied requirement CLEAN-01 (remove McpError enum) and eliminated all compiler warnings. Codebase is now warning-free with each fix committed atomically, ready for dependency patch removal in Plan 02.

## Tasks Completed

### Task 1: Remove unused imports, fields, functions, and enum variants (7 atomic commits)

Fixed 7 distinct clippy warnings in order of safety (imports → fields → functions → variants):

1. **73956c3** - Removed unused `MatchResult` re-export from matcher/mod.rs
   - Only used internally within scorer.rs and tests
   - External callers use `match_query` which returns it directly

2. **0567417** - Removed 5 unused type re-exports from registry/mod.rs
   - Category, Curator, Endorsement, Source, SourceType not imported via registry module
   - Only Registry and RegistryError are used externally
   - Internal code imports directly from registry::types

3. **50cecfc** - Suppressed dead_code warning for MatchResult.score field
   - Used in 4 test assertions but clippy ignores test usage
   - Valuable for debugging/logging
   - Added `#[allow(dead_code)]` with justification comment

4. **8a05aef** - Removed unused `tool_result` method from JsonRpcResponse
   - Tool handlers build responses directly using json! macro
   - Method was never called anywhere in codebase

5. **0c33ba1** - Suppressed dead_code warnings for InitializeParams fields
   - protocol_version, capabilities, client_info are MCP protocol spec fields
   - Used by serde deserialization, not accessed programmatically
   - Added `#[allow(dead_code)]` for protocol correctness

6. **e261cce** - Removed DuplicateSlug error variant
   - Registry loader doesn't check for duplicate slugs (HashMap handles uniqueness)
   - Never constructed anywhere

7. **750ad95** - Removed entire McpError enum and src/mcp/error.rs file
   - Defined but completely unused
   - MCP handler uses JSON-RPC error responses directly
   - **Fulfills requirement CLEAN-01**

### Task 2: Remove McpError enum and check for unused dependencies

**Part A: McpError removal** - Completed in Task 1 commit 7 (above)

**Part B: Dependency audit** - All 19 dependencies verified as used:
- anyhow, axum, dotenvy, envy, hex, pkarr, regex: Direct imports
- schemars: Used via derive macro (JsonSchema) on 4 structs
- serde, serde_json: Heavy use (derive and runtime)
- stop-words: Used in matcher/normalize.rs for stop word filtering
- strsim: Used for normalized Levenshtein similarity
- thiserror: Used via derive macro (Error) on error enums
- tokio: Runtime and async features
- tower-http: CorsLayer in server.rs
- tracing, tracing-subscriber: Logging infrastructure
- No unused dependencies found.

## Deviations from Plan

None - plan executed exactly as written.

All 7 warnings addressed with atomic commits. Each fix built and passed all 43 unit tests + 10 integration tests + 12 MCP integration tests + 7 registry tests independently.

## Verification Results

✅ `cargo clippy --all-targets -- -W unused -W dead-code` produces zero warnings
✅ `cargo build` succeeds
✅ `cargo test` passes all 72 tests
✅ `grep -r "McpError" --include="*.rs" src/` returns empty (CLEAN-01)
✅ `git log --oneline` shows 7 separate atomic commits
✅ No `#[allow(dead_code)]` except where justified (score field, InitializeParams)

## Self-Check: PASSED

**Created files exist:**
- N/A (no files created, only modified/deleted)

**Modified files exist:**
```
FOUND: src/matcher/mod.rs
FOUND: src/matcher/scorer.rs
FOUND: src/registry/mod.rs
FOUND: src/registry/error.rs
FOUND: src/mcp/types.rs
FOUND: src/mcp/mod.rs
```

**Deleted files removed:**
```
MISSING: src/mcp/error.rs (expected - deleted)
```

**Commits exist:**
```
FOUND: 73956c3 - refactor(matcher): remove unused MatchResult re-export
FOUND: 0567417 - refactor(registry): remove unused type re-exports
FOUND: 50cecfc - refactor(matcher): suppress dead_code warning for score field
FOUND: 8a05aef - refactor(mcp): remove unused tool_result method
FOUND: 0c33ba1 - refactor(mcp): suppress dead_code warnings for InitializeParams fields
FOUND: e261cce - refactor(registry): remove unused DuplicateSlug error variant
FOUND: 750ad95 - refactor(mcp): remove unused McpError enum
```

All 7 commits verified in git history.

## Impact

**Immediate:**
- Zero compiler warnings (clean build)
- CLEAN-01 requirement satisfied (McpError removed)
- Codebase ready for dependency patch removal (Plan 02)

**Quality:**
- Cleaner module exports (only export what's actually used)
- Smaller public API surface (less maintenance burden)
- Better code hygiene (no dead code accumulation)

**Testing:**
- All 72 tests remain passing
- No behavioral changes
- Each atomic commit independently tested

## Next Steps

Ready to proceed to Plan 02: Remove curve25519-dalek git patch from Cargo.toml and upgrade to released version (if available).
