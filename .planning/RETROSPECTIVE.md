# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v2.0 — Community Curation

**Shipped:** 2026-03-08
**Phases:** 3 | **Plans:** 6 | **Sessions:** ~3

### What Was Built
- Append-only audit log with Ed25519 signed, SHA-256 hash-chained entries (40 retroactive entries for all existing sources)
- Cross-platform identity linking (PKARR to X, Nostr, GitHub) with independently verifiable proof URLs
- Community contribution proposals with status lifecycle (pending/approved/rejected/withdrawn) and human/bot vote separation
- 6 new REST endpoints, 4 new MCP tools (8 total)
- Offline signing utility for audit entry generation

### What Worked
- Module pattern replication: all 3 new modules (audit, identity, contributions) followed src/registry/ structure — types.rs, loader.rs, error.rs, mod.rs — making each phase predictable
- Two-plan pattern per phase (data layer + server wiring) gave clean dependency chains with no inter-wave blocking
- Read-only server philosophy kept scope contained — no auth, no write endpoints, no state management
- Fail-fast on startup catches all data errors before serving traffic
- Shared filtering pattern (filter_entries) kept REST + MCP consistent with zero duplication

### What Was Inefficient
- SUMMARY.md one_liner field not populated by executors — accomplishment extraction required manual reading
- v2.0 phase numbering continued from v1.1 (12-14) which means phase directories accumulate across milestones

### Patterns Established
- Canonical signing format: `timestamp|action|category|sha256(data)|actor` — deterministic and independently verifiable
- `#[serde(default)]` over `deny_unknown_fields` for schema evolution
- Flat AppState fields matching existing codebase conventions
- Identity type classification (human/bot) with operator chain for accountability

### Key Lessons
1. Read-only server + offline tooling is a powerful separation — keeps the server simple while pushing complexity to curator workflows
2. Consistent module structure reduces planning overhead — by phase 14 the pattern was automatic
3. Voter identity cross-referencing (contributions referencing identities) validates the module ordering decision

### Cost Observations
- Model mix: ~20% opus (orchestration), ~80% sonnet (execution)
- Sessions: ~3 (research/planning, execution, completion)
- Notable: 6 plans executed in a single session with wave-based parallelization

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Sessions | Phases | Key Change |
|-----------|----------|--------|------------|
| v1.0 | ~10 | 7 | Initial GSD workflow, 17 plans |
| v1.1 | ~4 | 4 | Streamlined planning, 6 plans |
| v2.0 | ~3 | 3 | Module replication pattern, 6 plans |

### Cumulative Quality

| Milestone | Tests | LOC (Rust) | New Dependencies |
|-----------|-------|------------|-----------------|
| v1.0 | 72 | 3,016 | Core stack (axum, tokio, serde, pkarr) |
| v1.1 | 78 | 2,179 | None (dead code removed) |
| v2.0 | 144 | 6,029 | ed25519-dalek, sha2, uuid, chrono |

### Top Lessons (Verified Across Milestones)

1. Two-plan phase pattern (data layer + server wiring) works reliably for feature additions
2. Consistent module structure eliminates design decisions and speeds up execution
3. Read-only + offline tooling keeps server simple across all milestones
