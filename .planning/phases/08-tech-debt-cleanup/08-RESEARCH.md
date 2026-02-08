# Phase 8: Tech Debt Cleanup - Research

**Researched:** 2026-02-08
**Domain:** Rust dependency management, dead code removal, cargo tooling
**Confidence:** HIGH

## Summary

Phase 8 focuses on cleaning fragile dependencies and dead code before the infrastructure migration. The primary work involves: (1) attempting to remove the `[patch.crates-io]` curve25519-dalek patch from Cargo.toml, (2) removing dead code flagged by compiler warnings and clippy, specifically the unused McpError enum, and (3) removing any unused dependencies from Cargo.toml. This is a code-only phase with no deployment—all changes validated locally via build and tests.

The current dependency patch exists because pkarr 5.0.2 depends on ed25519-dalek 3.0.0-pre.5, which in turn depends on curve25519-dalek 5.0.0-pre.5 (a pre-release version). The patch redirects to the curve25519-dalek git main branch to work around compatibility issues with the pre-release crate. Removing the patch is a single-attempt operation: if cargo build and tests pass without it, the patch is successfully removed; if not, it stays.

Dead code cleanup is tooling-driven using cargo clippy and compiler warnings. Current findings show 7 warnings including unused imports (MatchResult, Category, Curator, etc.), unused fields (score, protocol_version, capabilities, client_info), unused functions (tool_result), unused enum variants (DuplicateSlug), and the never-used McpError enum. Each fix should be a separate atomic commit that independently builds and passes all 115 tests (43 unit + 43 unit test mode + 10 integration_matching + 12 integration_mcp + 7 integration_registry).

**Primary recommendation:** Execute fixes in order of increasing risk: (1) remove trivial unused imports/fields first (safest, compiler-verified), (2) remove McpError and unused functions (medium risk, requires careful grep), (3) attempt curve25519-dalek patch removal last (highest risk, upstream dependency change). Each commit should build and test independently to enable easy rollback.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### curve25519-dalek strategy
- One attempt: remove the `[patch.crates-io]` section from Cargo.toml
- Build locally — if it compiles and tests pass, the patch is gone
- If build fails, revert the patch immediately and move on (keep existing behavior)
- No extended investigation — single attempt, pass or fail

#### Dead code removal scope
- Remove McpError enum (scoped requirement)
- Quick sweep: run cargo clippy/warnings, remove any obviously unused code
- Also remove unused dependencies from Cargo.toml (not just dead Rust code)
- Don't go hunting — if it's flagged by tooling, remove it; otherwise leave it

#### Validation approach
- Local build + all tests passing is sufficient (`cargo build && cargo test`)
- No deployment to any platform in this phase — this milestone is migrating away from Render
- If all 72+ tests pass, phase is validated (actual count: 115 tests)

#### Commit strategy
- One commit per fix, not a single monolith commit
- Separate commits for: dependency patch removal, dead code/McpError removal, unused dep cleanup
- Each commit should independently build and pass tests
- Easy to revert individual changes if something breaks downstream

### Claude's Discretion

- Order of operations (which fix to tackle first)
- Exact clippy lints to act on vs ignore
- Whether to update any adjacent code that becomes simpler after removals

### Specific Ideas
- User is not deeply familiar with the curve25519-dalek patch — keep changes minimal and explain what happened in commit messages
- "Quick sweep" means tooling-driven, not manual code review — trust compiler warnings and clippy

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope

</user_constraints>

## Standard Stack

### Core Tools
| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| cargo | 1.92.0 | Rust build system and package manager | Official Rust toolchain, de facto standard |
| rustc | 1.92.0 | Rust compiler | Stable release channel, edition 2024 compatible |
| cargo clippy | 0.1.92 | Linter for common mistakes and code improvements | Official Rust linting tool, catches dead code |
| cargo test | (built-in) | Test runner for unit and integration tests | Standard Rust testing framework |

### Supporting Tools
| Tool | Version | Purpose | When to Use |
|------|---------|---------|-------------|
| cargo-machete | latest | Fast unused dependency detection | Quick first pass, imprecise but fast |
| cargo-udeps | latest (requires nightly) | Accurate unused dependency detection | Precise detection, requires nightly rust |
| cargo tree | (built-in) | View dependency tree | Understand dependency relationships |
| grep/ripgrep | system | Search for symbol usage | Verify dead code removal safety |

### Tool Selection for This Phase

**For dead code detection:** Use built-in `cargo clippy --all-targets -- -W unused -W dead-code` (HIGH confidence)
- Already installed and configured
- Zero additional setup
- Directly flags the 7 warnings in current codebase

**For unused dependency detection:** Use `cargo-machete` if available, otherwise manual Cargo.toml review (MEDIUM confidence)
- cargo-machete is fast and good enough for simple projects
- cargo-udeps requires nightly and compilation overhead
- Current project has only 19 direct dependencies, manual review feasible

**For symbol usage verification:** Use `grep -r "SymbolName" --include="*.rs"` (HIGH confidence)
- Before removing any code, grep to confirm no usage
- Standard tool, works everywhere
- Critical safety check before deletion

## Architecture Patterns

### Recommended Execution Structure

```
Phase 8 Execution Flow:
1. Baseline verification (cargo build && cargo test)
2. Dead code cleanup (multiple atomic commits)
   ├── Remove unused imports (lowest risk)
   ├── Remove unused struct fields (low risk)
   ├── Remove unused functions/methods (medium risk)
   └── Remove McpError enum (medium risk, explicit requirement)
3. Unused dependency removal (medium risk)
4. Dependency patch removal (highest risk, single attempt)
5. Final verification (cargo build && cargo test)
```

### Pattern 1: Atomic Commit per Fix

**What:** Each code fix is a separate, self-contained commit that leaves the codebase in a working state
**When to use:** Always in refactoring/tech debt work
**Benefits:** Easy rollback, clear history, bisectable, reviewable

**Example workflow:**
```bash
# Fix 1: Remove unused import
# Edit src/matcher/mod.rs - remove MatchResult from exports
cargo build && cargo test
git add src/matcher/mod.rs
git commit -m "refactor(matcher): remove unused MatchResult export

MatchResult is exported but never used in the codebase.
Flagged by clippy unused_imports warning.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"

# Fix 2: Remove unused McpError enum
# First verify it's truly unused
grep -r "McpError" --include="*.rs" src/
# Edit src/mcp/error.rs - delete file or enum
cargo build && cargo test
git add src/mcp/error.rs
git commit -m "refactor(mcp): remove unused McpError enum

McpError enum is never constructed or used.
Flagged by clippy dead_code warning.
Requirement CLEAN-01 fulfilled.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

### Pattern 2: Safe Dependency Patch Removal

**What:** Single-attempt removal of [patch.crates-io] section with immediate rollback on failure
**When to use:** When upstream dependencies may have resolved compatibility issues

**Example workflow:**
```bash
# Attempt patch removal
cp Cargo.toml Cargo.toml.backup
# Remove [patch.crates-io] section (lines 26-29)
cargo clean  # Force fresh dependency resolution
cargo build

# If build succeeds:
cargo test
# If tests pass:
git add Cargo.toml Cargo.lock
git commit -m "deps: remove curve25519-dalek patch

Attempt to remove [patch.crates-io] curve25519-dalek override.
Build and all 115 tests pass with published crates.
Requirements DEPS-01 and DEPS-02 fulfilled.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"

# If build/test fails:
git restore Cargo.toml
cargo build  # Restore working state
# Keep patch, document in phase results
```

### Pattern 3: Tooling-Driven Dead Code Detection

**What:** Let compiler and clippy identify unused code rather than manual code review
**When to use:** Always in tech debt cleanup—trust the tooling

**Example:**
```bash
# Get list of all warnings
cargo clippy --all-targets -- -W unused -W dead-code 2>&1 | tee clippy-warnings.txt

# Current warnings identified:
# 1. unused import: MatchResult (src/matcher/mod.rs:8)
# 2. unused imports: Category, Curator, Endorsement, SourceType, Source (src/registry/mod.rs:8)
# 3. unused field: score (src/matcher/scorer.rs:14)
# 4. unused enum: McpError (src/mcp/error.rs:2) [duplicate warning]
# 5. unused function: tool_result (src/mcp/types.rs:90)
# 6. unused fields: protocol_version, capabilities, client_info (src/mcp/types.rs:116)
# 7. unused variant: DuplicateSlug (src/registry/error.rs:21)

# Process each warning as separate commit
```

### Anti-Patterns to Avoid

- **Monolith refactoring commit:** Combining all fixes into one commit makes rollback impossible and review difficult
- **Extended investigation:** User explicitly wants single-attempt patch removal, not deep debugging sessions
- **Manual code hunting:** Only remove what tooling flags, don't search for more issues
- **Breaking tests between commits:** Each commit must independently build and pass all tests
- **Deployment in tech debt phase:** This phase validates locally only, no platform deployment

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Unused dependency detection | Manual Cargo.toml review | cargo-machete or cargo-udeps | Transitive deps, feature flags, conditional compilation make manual review error-prone |
| Dead code detection | Manual code analysis | cargo clippy with dead_code lint | Compiler knows exactly what's used/unused, including cross-crate visibility |
| Dependency version resolution | Manual Cargo.lock editing | cargo update or cargo clean + build | Cargo resolver handles complex version constraints and features |
| Commit message formatting | Free-form messages | Conventional commits format | Standardized format enables automation, clear intent, easy filtering |

**Key insight:** Rust's compiler and cargo tooling are extremely sophisticated. They understand feature flags, conditional compilation (cfg), platform-specific code, proc macros, and cross-crate visibility rules. Trust the tooling—attempting manual detection will miss edge cases and introduce errors.

## Common Pitfalls

### Pitfall 1: Removing Code That Appears Unused But Isn't

**What goes wrong:** Clippy flags code as unused, you remove it, then compilation fails because it's actually used via re-exports, macros, or cfg-gated code
**Why it happens:** Complex visibility rules, conditional compilation, or indirect usage through re-exports
**How to avoid:** Always build and test after each removal; grep for symbol usage before deleting
**Warning signs:** If grep shows hits but clippy says unused, check for cfg attributes or feature-gated usage

**Example from current codebase:**
```rust
// src/matcher/mod.rs:8
pub use scorer::{match_query, MatchResult};
// MatchResult is flagged as unused, but verify with:
// grep -r "MatchResult" --include="*.rs" src/
// If no hits outside this file, safe to remove
```

### Pitfall 2: Cargo.lock Divergence After Patch Removal

**What goes wrong:** Remove [patch.crates-io], run cargo build, it succeeds, but Cargo.lock still references git dependencies
**Why it happens:** Cargo doesn't automatically downgrade to released versions without cargo clean or cargo update
**How to avoid:** After removing patch, run `cargo clean` to force fresh resolution, or `cargo update` to resolve new versions
**Warning signs:** Cargo.lock contains git+https URLs instead of registry+https URLs

**Current state:**
```toml
# Cargo.lock currently has:
[[package]]
name = "curve25519-dalek"
version = "5.0.0-pre.5"
source = "git+https://github.com/dalek-cryptography/curve25519-dalek?branch=main#59305b4e..."

# After successful patch removal, should become:
[[package]]
name = "curve25519-dalek"
version = "4.1.3"  # or whatever stable version resolves
source = "registry+https://github.com/rust-lang/crates.io-index"
```

### Pitfall 3: Forgetting to Test All Test Targets

**What goes wrong:** Run cargo test, tests pass, commit, but integration tests or examples break
**Why it happens:** cargo test without --all-targets only runs unit tests by default
**How to avoid:** Always use `cargo build && cargo test` (test implies build, catches both compilation and test failures)
**Warning signs:** CI fails even though local tests passed

**Current project test structure:**
- Unit tests: 43 tests in lib code (runs twice, once per build)
- Integration tests: 29 tests across 3 files (integration_matching, integration_mcp, integration_registry)
- Total: 115 test executions
- Command: `cargo test` runs all by default for this simple project structure

### Pitfall 4: Removing Unused Code That's Part of Public API

**What goes wrong:** Remove unused enum variant or struct field, breaks downstream consumers (not applicable here, but common)
**Why it happens:** Crate is a library, unused in current crate doesn't mean unused by dependents
**How to avoid:** Check if crate is published library; if yes, consider deprecation instead of removal
**Warning signs:** Exported types/functions flagged as unused

**Current project status:** Binary crate (bin "three-good-sources"), not a library published to crates.io, so safe to remove unused exports

### Pitfall 5: Atomic Commits That Don't Actually Build

**What goes wrong:** Commit passes tests but later refactoring reveals it never compiled correctly
**Why it happens:** Tests cached from previous build, didn't catch new compile errors
**How to avoid:** Run `cargo clean && cargo build && cargo test` for each commit (or at least `cargo build`)
**Warning signs:** CI catches compilation errors that weren't seen locally

**Best practice for this phase:**
```bash
# For each commit:
cargo build  # Explicit compile check
cargo test   # Run all tests
# Then commit if both succeed
```

## Code Examples

Verified patterns from official sources and current codebase:

### Removing Unused Imports
```rust
// BEFORE (src/matcher/mod.rs:8)
pub use scorer::{match_query, MatchResult};

// AFTER
pub use scorer::match_query;
```

### Removing Unused Struct Fields
```rust
// BEFORE (src/matcher/scorer.rs:14)
pub struct MatchResult {
    pub score: f64,  // Flagged: field is never read
    pub matched_spans: Vec<(usize, usize)>,
}

// OPTION 1: Remove field entirely
pub struct MatchResult {
    pub matched_spans: Vec<(usize, usize)>,
}

// OPTION 2: Keep for future use but suppress warning
pub struct MatchResult {
    #[allow(dead_code)]
    pub score: f64,
    pub matched_spans: Vec<(usize, usize)>,
}

// Recommendation: Remove entirely per user's "quick sweep" directive
```

### Removing Entire Unused Enum
```rust
// BEFORE (src/mcp/error.rs) - entire file
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    #[error("JSON parsing failed: {0}")]
    ParseError(String),
    #[error("Response serialization failed: {0}")]
    SerializationError(String),
}

// AFTER: Delete entire file, or replace with empty/comment-only file
// Then remove from mod.rs:
// BEFORE (src/mcp/mod.rs)
pub mod error;

// AFTER (remove this line if file deleted)
```

### Removing [patch.crates-io] Section
```toml
# BEFORE (Cargo.toml lines 26-29)
# Workaround for pkarr 5.0.2 dependency issues with pre-release curve25519-dalek
# Patch to use compatible git version that compiles
[patch.crates-io]
curve25519-dalek = { git = "https://github.com/dalek-cryptography/curve25519-dalek", branch = "main" }

# AFTER: Delete these 4 lines entirely
# (Next section [dev-dependencies] starts at line 31)
```

### Verifying Symbol Usage Before Removal
```bash
# Before removing McpError enum, verify it's truly unused:
grep -r "McpError" --include="*.rs" src/

# Expected output (only definition, no usage):
# src/mcp/error.rs:2:pub enum McpError {
# src/mcp/error.rs:3:    #[error("JSON parsing failed: {0}")]
# src/mcp/error.rs:5:    #[error("Response serialization failed: {0}")]

# If grep shows usage beyond definition, DO NOT REMOVE
# If only definition lines appear, safe to remove

# Before removing MatchResult export:
grep -r "MatchResult" --include="*.rs" src/

# Expected output:
# src/matcher/mod.rs:8:pub use scorer::{match_query, MatchResult};
# src/matcher/scorer.rs:10:pub struct MatchResult {
# If no other files reference it, safe to remove from exports
```

### Cargo Commands for Phase 8
```bash
# Initial baseline
cargo clean
cargo build
cargo test

# Check for unused dependencies (if cargo-machete installed)
cargo machete

# Get detailed clippy warnings
cargo clippy --all-targets -- -W unused -W dead-code 2>&1 | tee warnings.txt

# After each code change
cargo build  # Verify compilation
cargo test   # Verify tests (runs all 115 tests)

# After removing [patch.crates-io]
cargo clean  # Force fresh dependency resolution
cargo build  # Will fetch from crates.io instead of git
cargo test   # Verify everything still works

# Verify Cargo.lock has no git dependencies
grep "git+" Cargo.lock
# Should return empty if patch successfully removed
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| cargo-prune (deprecated) | cargo-machete / cargo-udeps | 2021-2023 | Modern tools handle feature flags and workspaces better |
| Manual [patch] section management | Cargo resolver improvements | Rust 1.51+ (2021) | Better version resolution, fewer patches needed |
| Separate cargo check + cargo test | cargo test implies check | Rust 1.0+ (2015) | cargo test builds before testing, explicit check optional |
| Clippy as separate tool | Integrated via rustup component | Rust 1.29+ (2018) | Standard installation via rustup component add clippy |

**Deprecated/outdated:**
- **cargo-prune:** Older tool for unused dependency detection, superseded by cargo-machete and cargo-udeps
- **Manual dependency version management:** Cargo resolver is now sophisticated enough for most use cases, [patch] should be temporary
- **curve25519-dalek 4.x with patch:** Version 5.0.0-pre.5 has been in pre-release since early 2025, check if stable 5.0 or compatible 4.x is available

**Current Rust Ecosystem (Feb 2026):**
- Rust 1.92.0 stable (Dec 2025)
- Cargo 1.92.0 with improved resolver
- Clippy 0.1.92 (bundled with rustc)
- Edition 2024 (stable since Oct 2024)
- curve25519-dalek latest stable: 4.1.3 (2024)
- ed25519-dalek using pre-release 3.0.0-pre.5 (via pkarr dependency)

## Dependency Analysis

### Current State
```
Project: three-good-sources
├── Direct dependencies: 19 (in [dependencies])
├── Dev dependencies: 2 (in [dev-dependencies])
├── Total compiled: ~200+ (including transitive)
├── Patched: 1 (curve25519-dalek via git)
└── Total tests: 115 (43 unit + 43 unit retest + 29 integration)
```

### Dependency Chain for curve25519-dalek Patch
```
three-good-sources
└── pkarr 5.0.2 (features: keys)
    └── ed25519-dalek 3.0.0-pre.5
        └── curve25519-dalek 5.0.0-pre.5 ← PATCHED to git main branch
```

**Why patch exists:** pkarr 5.0.2 (released Jan 9, 2026) depends on ed25519-dalek 3.0.0-pre.5, which depends on curve25519-dalek 5.0.0-pre.5. The pre-release version 5.0.0-pre.5 may have compilation issues or incompatibilities, so the patch redirects to the latest git main branch to get fixes not yet in a crates.io release.

**What happens when patch is removed:**
1. Cargo will try to resolve curve25519-dalek from crates.io
2. pkarr → ed25519-dalek → curve25519-dalek dependency chain must resolve to published versions
3. Two possible outcomes:
   - **Success:** Pre-release 5.0.0-pre.5 now works, or resolver finds compatible 4.x version
   - **Failure:** Version conflict or compilation error, revert and keep patch

### Unused Code Inventory (from clippy)

| Category | Count | Specific Items | Action Required |
|----------|-------|----------------|-----------------|
| Unused imports | 2 | MatchResult, Category, Curator, Endorsement, SourceType, Source | Remove from pub use |
| Unused fields | 4 | score, protocol_version, capabilities, client_info | Remove or #[allow(dead_code)] |
| Unused functions | 1 | tool_result | Remove |
| Unused enums | 1 | McpError (entire enum) | Remove (CLEAN-01 requirement) |
| Unused enum variants | 1 | DuplicateSlug | Remove or #[allow(dead_code)] |

**Total warnings:** 7 unique warnings (8 if counting duplicate McpError warning in test build)

## Risk Assessment

### Low Risk (Safe to execute first)
- **Remove unused imports:** Compiler-verified, zero runtime impact, safe
- **Remove unused struct fields (non-pub):** Internal-only, caught at compile time
- Example: Remove MatchResult from exports, remove score field from internal struct

### Medium Risk (Requires careful verification)
- **Remove unused functions/methods:** Grep verification needed to ensure no dynamic dispatch or reflection usage
- **Remove unused enum variants:** May affect pattern matching exhaustiveness
- **Remove McpError enum:** Entire type removal, requires thorough grep check
- Example: McpError enum deletion after verifying zero usage outside definition

### High Risk (Single attempt, immediate rollback on failure)
- **Remove [patch.crates-io] section:** Changes upstream dependency resolution, may cause build or test failures
- Example: curve25519-dalek patch removal—if it fails, revert and document

### Recommended Execution Order
1. ✅ Remove unused imports (lowest risk, fastest validation)
2. ✅ Remove unused fields from internal structs
3. ✅ Remove unused functions (with grep verification)
4. ✅ Remove McpError enum (CLEAN-01 requirement, with grep verification)
5. ✅ Remove unused enum variants
6. ⚠️ Check for unused dependencies (cargo-machete or manual review)
7. 🔴 Attempt curve25519-dalek patch removal (highest risk, final step)

## Open Questions

### Question 1: curve25519-dalek Stable Release Status
- **What we know:** Latest stable on crates.io is 4.1.3 (2024), pre-release 5.0.0-pre.5 exists
- **What's unclear:** Has a stable 5.0 been released? Will ed25519-dalek 3.0.0-pre.5 work with curve25519-dalek 4.1.3?
- **Recommendation:** Attempt patch removal as specified (single attempt), let Cargo resolver decide. If it fails, we document and keep patch. No extended investigation needed per user constraints.
- **Confidence:** LOW - couldn't verify current release status from web search, crates.io pages required JavaScript

### Question 2: Unused Dependencies Beyond Dead Code
- **What we know:** Clippy flags dead Rust code but doesn't flag unused Cargo.toml dependencies
- **What's unclear:** Are there unused direct dependencies in Cargo.toml?
- **Recommendation:** Run cargo-machete if available, otherwise manual quick review of 19 direct dependencies. Check for dependencies imported but unused in src/.
- **Confidence:** MEDIUM - cargo-machete exists but is imprecise, manual review feasible for small project

### Question 3: Test Count Discrepancy
- **What we know:** User mentioned "72+ tests", actual count is 115 test executions
- **What's unclear:** Does user's count refer to unique test functions (fewer) vs total test runs (more)?
- **Recommendation:** Report 115 as actual test execution count, clarify that some tests run twice (unit tests in both normal and test mode). Success criteria: all 115 executions pass.
- **Confidence:** HIGH - cargo test output is definitive

## Sources

### Primary (HIGH confidence)
- Cargo.toml and Cargo.lock in current repository - dependency structure and versions
- `cargo test` output - 115 test executions (43+43+10+12+7)
- `cargo clippy` output - 7 warnings for unused/dead code
- `cargo tree` output - dependency chain for curve25519-dalek
- [Official Cargo Book: Overriding Dependencies](https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html)
- [Official Cargo Book: cargo test](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
- [Official Clippy Documentation](https://doc.rust-lang.org/clippy/usage.html)

### Secondary (MEDIUM confidence)
- [Atomic Commits Best Practices - GitByBit](https://gitbybit.com/gitopedia/best-practices/atomic-commits) - git workflow patterns
- [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) - commit message format
- [cargo-machete GitHub](https://github.com/bnjbvr/cargo-machete) - unused dependency detection
- [cargo-udeps GitHub](https://github.com/est31/cargo-udeps) - unused dependency detection
- [Rust Edition Guide: Replacing Dependencies with Patch](https://doc.rust-lang.org/edition-guide/rust-2018/cargo-and-crates-io/replacing-dependencies-with-patch.html)
- [dalek-cryptography/curve25519-dalek GitHub](https://github.com/dalek-cryptography/curve25519-dalek) - source repository

### Tertiary (LOW confidence)
- pkarr 5.0.2 crates.io page (couldn't fetch, JavaScript required) - dependency details
- curve25519-dalek crates.io page (couldn't fetch, JavaScript required) - latest stable version
- WebSearch results indicating curve25519-dalek 4.1.3 stable (2024) and no 5.0 stable as of Feb 2026

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - cargo, rustc, clippy versions verified locally, official tools
- Architecture patterns: HIGH - atomic commits and patch removal are well-established practices
- Dependency analysis: HIGH - verified via cargo tree and Cargo.lock inspection
- Dead code inventory: HIGH - directly from clippy output on current codebase
- Risk assessment: HIGH - based on Rust compilation guarantees and current codebase state
- curve25519-dalek status: LOW - couldn't verify latest crates.io versions due to web fetch limitations

**Research date:** 2026-02-08
**Valid until:** 2026-03-08 (30 days - stable domain, Rust releases every 6 weeks but patterns remain consistent)

**Caveats:**
- curve25519-dalek stable release status unverified due to crates.io page fetch issues
- cargo-machete availability unknown (not installed in current environment)
- Actual unused dependencies not checked yet (requires running detection tool)
