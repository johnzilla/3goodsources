# Phase 15: Federation Foundation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-03
**Phase:** 15-federation-foundation
**Areas discussed:** Endorsement schema details, PeerRegistry type scope, Federation module layout

---

## Endorsement Schema Details

### Since Field Type

| Option | Description | Selected |
|--------|-------------|----------|
| String (ISO 8601 date) | Matches registry.json pattern. Simple. Forward-compatible. | ✓ |
| chrono::DateTime<Utc> | Type-safe. Matches audit/contributions pattern. Stricter. | |
| You decide | Claude picks best fit | |

**User's choice:** String (ISO 8601 date)
**Notes:** Matches existing `updated` field pattern in registry.json. Simpler for federation where peer formats may vary.

### Endorsement Type Sharing

| Option | Description | Selected |
|--------|-------------|----------|
| Shared Endorsement struct | One type everywhere, remove deny_unknown_fields from Endorsement only | |
| Separate PeerEndorsement | PeerRegistry uses own type. Full isolation but more types. | ✓ |
| You decide | Claude picks | |

**User's choice:** Separate PeerEndorsement
**Notes:** Full isolation between local strict types and peer lax types.

---

## PeerRegistry Type Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Full mirror (5 types) | PeerRegistry, PeerCurator, PeerEndorsement, PeerCategory, PeerSource | |
| Top 3 only | PeerRegistry, PeerCurator, PeerEndorsement. Reuse Category/Source. | ✓ |
| You decide | Claude picks | |

**User's choice:** Top 3 only
**Notes:** Category and Source types are stable and unlikely to gain new fields. Less duplication.

---

## Federation Module Layout

| Option | Description | Selected |
|--------|-------------|----------|
| types.rs + cache.rs + mod.rs | 3 files. Minimal. | |
| Follow full pattern (add error.rs) | 4 files. Consistent with audit/, identity/, contributions/. | ✓ |
| You decide | Claude picks | |

**User's choice:** Follow full pattern (add error.rs)
**Notes:** Consistency with established module structure across the codebase.

---

## Claude's Discretion

- Exact FederationError variant names and messages
- CachedPeer field types (Option<Instant> vs enum)
- Test organization within federation module

## Deferred Ideas

None — discussion stayed within phase scope
