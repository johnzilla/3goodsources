---
phase: 12
slug: audit-log
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-07
---

# Phase 12 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test + cargo test |
| **Config file** | Cargo.toml (already configured) |
| **Quick run command** | `cargo test --lib audit` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib audit`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 12-01-01 | 01 | 1 | AUDIT-01 | unit | `cargo test --lib audit::loader::tests` | No - W0 | ⬜ pending |
| 12-01-02 | 01 | 1 | AUDIT-02 | unit | `cargo test --lib audit::loader::tests::test_signature_verification` | No - W0 | ⬜ pending |
| 12-01-03 | 01 | 1 | AUDIT-03 | unit | `cargo test --lib audit::loader::tests::test_hash_chain` | No - W0 | ⬜ pending |
| 12-02-01 | 02 | 2 | AUDIT-04 | integration | `cargo test --test integration_audit` | No - W0 | ⬜ pending |
| 12-02-02 | 02 | 2 | AUDIT-05 | unit | `cargo test --lib mcp::handler::tests::test_get_audit_log` | No - W0 | ⬜ pending |
| 12-03-01 | 03 | 1 | AUDIT-06 | integration | `cargo test --test integration_audit::test_retroactive_entries` | No - W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/audit/` module — does not exist yet
- [ ] `tests/fixtures/audit_log.json` — test fixture for valid audit log
- [ ] `tests/fixtures/audit_log_invalid.json` — test fixture for error cases
- [ ] `tests/integration_audit.rs` — integration tests for GET /audit
- [ ] uuid and chrono dependencies in Cargo.toml
- [ ] ed25519-dalek and sha2 as direct dependencies in Cargo.toml

*Wave 0 creates the module skeleton and test infrastructure.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Audit log loads at server startup | AUDIT-01 | Startup behavior verified by integration tests starting server | Run `cargo test --test integration_audit` |

*All phase behaviors have automated verification via unit and integration tests.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
