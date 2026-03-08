---
phase: 13
slug: identity-linking
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-08
---

# Phase 13 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test + tokio::test (async) |
| **Config file** | Cargo.toml [dev-dependencies] |
| **Quick run command** | `cargo test identity` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test identity`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 13-01-01 | 01 | 1 | IDENT-01, IDENT-02, IDENT-03 | unit | `cargo test identity` | ❌ W0 | ⬜ pending |
| 13-01-02 | 01 | 1 | IDENT-07 | unit | `cargo test identity::loader` | ❌ W0 | ⬜ pending |
| 13-02-01 | 02 | 2 | IDENT-04, IDENT-05 | integration | `cargo test integration_identity` | ❌ W0 | ⬜ pending |
| 13-02-02 | 02 | 2 | IDENT-06 | unit+integration | `cargo test get_identity` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/identity/types.rs` — unit tests for Identity, IdentityType, Platform, PlatformClaim deserialization
- [ ] `src/identity/loader.rs` — unit tests for load(), bot validation, empty file handling
- [ ] `tests/integration_identity.rs` — integration tests for GET /identities and GET /identities/{pubkey}
- [ ] `src/mcp/handler.rs` — add test for get_identity MCP tool call
- [ ] `identities.json` — seed data file (test fixture and production)
- [ ] `tests/common/mod.rs` — update spawn_test_server to load identities

*Existing infrastructure covers test framework — only test files need creation.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Proof URLs are independently verifiable | IDENT-07 | Requires visiting external URLs | Visit each proof URL and confirm PKARR pubkey is present |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
