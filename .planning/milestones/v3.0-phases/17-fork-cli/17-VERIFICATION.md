---
phase: 17-fork-cli
verified: 2026-04-02T00:00:00Z
status: passed
score: 5/5 must-haves verified
re_verification: false
---

# Phase 17: Fork CLI Verification Report

**Phase Goal:** A new curator can scaffold a ready-to-run 3GS node by running a single CLI command without needing any environment variables pre-configured
**Verified:** 2026-04-02
**Status:** PASSED
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Running `cargo run -- fork --endorse <pubkey> --url <url>` creates a new directory with all scaffolded files | VERIFIED | Behavioral spot-check confirmed: 5 files created (registry.json, identities.json, contributions.json, audit_log.json, .env), exit 0 |
| 2 | The fork subcommand works without any environment variables set (no .env, no REGISTRY_PATH, etc.) | VERIFIED | Fork arg interception block in main.rs at lines 41-52 executes before `Config::load()` at line 55; `fork::run()` imports only external crates (pkarr, hex, serde_json, chrono) — zero `use crate::` imports |
| 3 | The scaffolded .env, registry.json, identities.json, contributions.json, and audit_log.json are all valid for the server to load | VERIFIED | registry.json has correct schema (version, updated, curator, endorsements, categories); contributions.json is `{}` matching HashMap<Uuid, Proposal> loader; identities.json is `{}`, audit_log.json is `[]`; .env contains all 6 required vars |
| 4 | Running fork with missing required flags prints usage and exits 1 | VERIFIED | `cargo run -- fork` (no flags) prints "Error: --endorse is required" + USAGE block, exits with code 1 |
| 5 | Running fork when output directory already exists prints error and exits 1 | VERIFIED | Re-running with same `--output` path prints "already exists" error message, exits with code 1 |

**Score:** 5/5 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/fork.rs` | Fork CLI scaffolding logic | VERIFIED | 265 lines (exceeds min_lines: 100); exports `pub fn run`; contains `Keypair::random`, `--endorse`/`--url` parsing, all 5 file writes, 5 unit tests |
| `src/main.rs` | Fork arg interception before Config::load() | VERIFIED | Fork check block at lines 41-52; `Config::load()` at line 55; `mod fork;` declared at line 6 |
| `src/lib.rs` | Module declaration for fork | VERIFIED | `pub mod fork;` at line 6, alphabetically ordered after `pub mod federation;` |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src/main.rs` | `src/fork.rs` | args check before Config::load() calls fork::run() | VERIFIED | `crate::fork::run(args)` at line 44, fork block ends before Config::load() at line 55 |
| `src/fork.rs` | `pkarr::Keypair` | Keypair::random() for key generation | VERIFIED | `let keypair = pkarr::Keypair::random();` at line 81 |
| `src/fork.rs` | `registry.json skeleton` | serde_json::to_string_pretty for Registry-compatible JSON | VERIFIED | `serde_json::to_string_pretty(&registry)` at line 119; json! macro builds version/updated/curator/endorsements/categories structure |

### Data-Flow Trace (Level 4)

Not applicable — fork.rs is a CLI scaffolding module, not a rendering component. It writes static skeleton files with values derived from runtime inputs (keypair, CLI flags, current date). No dynamic data rendering path to trace.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Scaffolds 5-file directory with correct content | `cargo run -- fork --endorse testpubkey123abc --url http://localhost:3001 --name "Test Curator" --output /tmp/3gs-fork-verify-test` | Created directory with all 5 files; registry.json valid JSON with endorsement pubkey/url; .env contains all 6 required vars; PKARR_SECRET_KEY is 64-char hex | PASS |
| Missing flags exit 1 with usage | `cargo run -- fork` (no flags) | Printed "Error: --endorse is required" + full USAGE block; exit code 1 | PASS |
| Existing directory exits 1 with error | `cargo run -- fork --endorse X --url Y --output /tmp/3gs-fork-verify-test` (dir already exists) | Printed "already exists" error; exit code 1 | PASS |
| All 5 unit tests pass | `cargo test --lib fork` | 5 tests: test_missing_endorse_flag, test_missing_url_flag, test_unknown_flag, test_scaffolds_directory, test_existing_directory_errors | PASS (5/5 ok) |
| Project compiles cleanly | `cargo build` | Finished dev profile with 0 errors (20 pre-existing warnings, none in fork.rs) | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| DIST-01 | 17-01-PLAN.md | `3gs fork` CLI subcommand scaffolds a new node with keypair, skeleton files, and .env | SATISFIED | src/fork.rs: full implementation; behavioral spot-check confirms all 5 files created with correct content |
| DIST-02 | 17-01-PLAN.md | Fork parses args before Config::load() to avoid requiring env vars | SATISFIED | main.rs lines 41-52: fork block before line 55 Config::load(); fork.rs has zero crate:: imports |

All requirement IDs declared in 17-01-PLAN.md frontmatter (DIST-01, DIST-02) are accounted for. No orphaned requirements — REQUIREMENTS.md traceability table maps both IDs exclusively to Phase 17.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| None found | — | — | — | — |

fork.rs contains no TODO/FIXME/placeholder comments, no empty implementations, no hardcoded empty returns. All 5 file writes produce real, non-empty content. The module is substantive and complete.

Note: 20 compiler warnings exist in the broader codebase (unused imports, dead code in federation types). None are in fork.rs and none block compilation or the fork goal.

### Human Verification Required

None. All observable truths were verified programmatically via behavioral spot-checks. The fork command runs, produces correct files, and handles error cases with correct exit codes. No visual UI, external services, or runtime-only behaviors require human confirmation for this phase.

### Gaps Summary

No gaps. Phase 17 goal is fully achieved. A new curator can:

1. Run `3gs fork --endorse <pubkey> --url <peer_url>` with no environment variables set
2. Receive a scaffolded directory containing all 5 required files
3. Change into that directory and start the server using the generated .env

The fork check intercepts args before Config::load() (verified by code position), the module is decoupled from all server logic (no crate:: imports), and all error cases (missing flags, existing directory) produce non-zero exits with clear messages.

---

_Verified: 2026-04-02_
_Verifier: Claude (gsd-verifier)_
