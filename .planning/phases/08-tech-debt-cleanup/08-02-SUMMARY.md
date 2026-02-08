---
phase: 08-tech-debt-cleanup
plan: 02
subsystem: dependency-management
tags: [dependencies, patch-removal, tech-debt, curve25519-dalek]
dependency_graph:
  requires: [08-01]
  provides: [deps-02-attempted]
  affects: [pkarr, ed25519-dalek, curve25519-dalek]
tech_stack:
  added: []
  patterns: [single-attempt-validation, safe-rollback]
key_files:
  created: []
  modified: []
  deleted: []
decisions:
  - "curve25519-dalek patch must remain - pre.5 version has crypto_common import error"
  - "Patch removal failed at compile time - crypto_common module not found in digest crate"
  - "Rollback successful - patch restored, all 72 tests passing with git dependency"
metrics:
  duration: 90s
  tasks_completed: 1
  files_modified: 0
  commits: 0
  completed_date: 2026-02-08
---

# Phase 08 Plan 02: curve25519-dalek Patch Removal Attempt Summary

**One-liner:** Attempted to remove curve25519-dalek git patch; compilation failed with crypto_common import error, patch restored, requirements DEPS-01/DEPS-02 remain open.

## Objective Achieved

Single-attempt patch removal executed as planned. Build failed at compilation, patch was immediately restored, and codebase returned to known-good state. Per user decision, this is an acceptable outcome - the attempt was made and documented.

## Tasks Completed

### Task 1: Attempt curve25519-dalek patch removal (No commit - failure path)

**Baseline verification:**
- Initial build with patch: ✅ Success
- Initial tests (72 total): ✅ All passing
- curve25519-dalek version: v5.0.0-pre.6 (git patch applied)

**Patch removal attempt:**

1. Created backup: `Cargo.toml.backup`
2. Removed 4 lines from Cargo.toml (lines 26-29):
   ```toml
   # Workaround for pkarr 5.0.2 dependency issues with pre-release curve25519-dalek
   # Patch to use compatible git version that compiles
   [patch.crates-io]
   curve25519-dalek = { git = "https://github.com/dalek-cryptography/curve25519-dalek", branch = "main" }
   ```
3. Ran `cargo clean` to force fresh dependency resolution
4. Ran `cargo build`

**Build failure:**

```
error[E0432]: unresolved import `digest::crypto_common`
   --> /home/john/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/curve25519-dalek-5.0.0-pre.5/src/edwards.rs:109:65
    |
109 |     FixedOutput, HashMarker, array::typenum::U64, consts::True, crypto_common::BlockSizeUser,
    |                                                                 ^^^^^^^^^^^^^ could not find `crypto_common` in `digest`

error: could not compile `curve25519-dalek` (lib) due to 1 previous error
```

**Root cause:**
- Without the patch, cargo resolves to curve25519-dalek v5.0.0-pre.5 from crates.io
- This version has an import error: `digest::crypto_common` module doesn't exist
- The git patch (main branch, pre.6) has the correct imports for the current digest crate version
- pkarr 5.0.2 → ed25519-dalek 3.0.0-pre.5 → curve25519-dalek 5.0.0-pre.5 dependency chain requires the patch

**Restoration:**

1. Restored `Cargo.toml` from backup
2. Deleted stale `Cargo.lock` (was pointing to pre.5)
3. Ran `cargo build`: ✅ Success (using pre.6 from git)
4. Ran `cargo test`: ✅ All 72 tests passing
5. Deleted backup file

**Verification results:**

✅ `grep "patch.crates-io" Cargo.toml` - patch present (expected)
✅ `grep "git+" Cargo.lock` - git dependency present (expected)
✅ `cargo build` succeeds with patch
✅ `cargo test` passes all 72 tests with patch
✅ Codebase in known-good state

**Per plan instructions:** No commit made on failure path.

## Deviations from Plan

**Auto-fixed Issues:**

**1. [Rule 3 - Blocking] Cargo.lock stale after restoration**
- **Found during:** Task 1 restoration verification
- **Issue:** After restoring Cargo.toml, cargo build failed with same crypto_common error. The patch warning showed "was not used in the crate graph" because Cargo.lock was locked to pre.5 version.
- **Fix:** Deleted Cargo.lock to force fresh resolution, allowing patch to apply correctly to pre.6 from git.
- **Files modified:** Cargo.lock (deleted and regenerated)
- **Commit:** None (restoration step, not a feature)

No other deviations - plan executed exactly as written.

## Verification Results

✅ Patch removal was attempted (single attempt per user decision)
✅ Build failure captured and documented
✅ Cargo.toml restored to original state with patch intact
✅ `cargo build` succeeds with patch
✅ `cargo test` passes all 72 tests
✅ No commits made (failure path as specified in plan)

## Self-Check: PASSED

**Created files exist:**
- N/A (no files created)

**Modified files exist:**
- N/A (no files modified - restoration successful)

**Commits exist:**
- N/A (no commits on failure path per plan - expected and correct)

**Restoration verification:**
```bash
FOUND: [patch.crates-io] section in Cargo.toml
FOUND: git+https://github.com/dalek-cryptography/curve25519-dalek in Cargo.lock
PASSED: cargo build with patch
PASSED: cargo test (72 tests)
```

All verification checks passed.

## Impact

**Immediate:**
- curve25519-dalek patch remains in Cargo.toml (required for compilation)
- DEPS-01 requirement remains open (patch not removed)
- DEPS-02 requirement remains open (still using git dependency)
- Fragile git dependency continues into infrastructure migration

**Technical debt:**
- Git dependency on curve25519-dalek main branch persists
- Project dependent on upstream curve25519-dalek stability
- Risk: Future git commits could break builds (low probability, but non-zero)

**Phase status:**
- Per user decision, this outcome is acceptable
- Phase 8 can complete with DEPS-01/DEPS-02 open
- Documented failure meets phase requirements

**Why patch removal failed:**
- curve25519-dalek v5.0.0-pre.5 on crates.io is incompatible with current digest crate versions
- The git patch (pre.6 from main branch) has updated imports that work
- pkarr → ed25519-dalek → curve25519-dalek chain requires pre-release versions
- Pre-release ecosystem has version skew issues

**Alternatives considered (per context):**
- None - single attempt was the user decision
- Extended investigation explicitly forbidden in plan
- No version pinning experiments permitted

## Next Steps

Phase 08 complete. Both plans executed:
- Plan 01: Dead code removal ✅ (7 commits, zero warnings)
- Plan 02: Patch removal attempt ✅ (documented failure)

Ready to proceed to Phase 09: DigitalOcean App Platform configuration.

**Recommendations:**
- Monitor curve25519-dalek releases - stable v5.0.0 may resolve this
- Check pkarr updates - newer versions may drop pre-release dependency
- Revisit patch removal after Phase 11 (post-migration) if ecosystem stabilizes
