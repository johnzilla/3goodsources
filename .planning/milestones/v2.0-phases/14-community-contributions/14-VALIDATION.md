---
phase: 14
slug: community-contributions
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-08
---

# Phase 14 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test contributions -- --nocapture` |
| **Full suite command** | `cargo test -- --nocapture` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test contributions -- --nocapture`
- **After every plan wave:** Run `cargo test -- --nocapture`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 14-01-01 | 01 | 1 | CONTRIB-01, CONTRIB-02 | unit | `cargo test contributions -- --nocapture` | ✅ | ⬜ pending |
| 14-01-02 | 01 | 1 | CONTRIB-03 | unit | `cargo test contributions -- --nocapture` | ✅ | ⬜ pending |
| 14-02-01 | 02 | 2 | CONTRIB-04, CONTRIB-05, CONTRIB-06 | integration | `cargo test -- --nocapture` | ✅ | ⬜ pending |
| 14-02-02 | 02 | 2 | CONTRIB-04, CONTRIB-05, CONTRIB-06 | integration | `cargo test integration_contributions -- --nocapture` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. Test files created within implementation tasks (unit tests in loader.rs, integration tests in tests/integration_contributions.rs).

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Demo proposal data is realistic | CONTRIB-01 | Content quality check | Review contributions.json for sensible demo data |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
