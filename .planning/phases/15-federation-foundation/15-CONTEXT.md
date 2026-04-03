# Phase 15: Federation Foundation - Context

**Gathered:** 2026-04-03
**Status:** Ready for planning

<domain>
## Phase Boundary

Define the data model for endorsements and peer registries. Populate the existing empty Endorsement struct with real fields, create lax PeerRegistry types for forward-compatible federation, add self-endorsement guard, and move reqwest to runtime dependency. This is the foundation that Phases 16-18 build on.

</domain>

<decisions>
## Implementation Decisions

### Endorsement Schema
- **D-01:** Endorsement struct gains 4 fields: `pubkey: String`, `url: String`, `name: Option<String>`, `since: String`
- **D-02:** `since` field is a plain String (ISO 8601 date format like "2026-04-03"), matching registry.json's `updated` field pattern. NOT chrono::DateTime — keeps it simple and forward-compatible with any date format a peer might use.
- **D-03:** No `deny_unknown_fields` on Endorsement (forward compatibility for federation)
- **D-04:** Existing `registry.json` has `"endorsements": []` — empty array deserializes fine with new struct. No migration needed.

### PeerRegistry Type Scope
- **D-05:** Separate PeerEndorsement type (not shared with local Endorsement). Full isolation between local strict types and peer lax types.
- **D-06:** Top 3 lax types only: PeerRegistry, PeerCurator, PeerEndorsement. All without `deny_unknown_fields`.
- **D-07:** Reuse existing Category and Source types for peer registries. These are stable types unlikely to gain new fields, and duplication isn't justified.
- **D-08:** PeerRegistry uses `#[serde(default)]` on endorsements and categories fields for graceful handling of missing fields.

### Federation Module Layout
- **D-09:** Follow established 4-file module pattern: `src/federation/mod.rs`, `types.rs`, `cache.rs`, `error.rs`
- **D-10:** `types.rs` contains: PeerRegistry, PeerCurator, PeerEndorsement, FederatedMatch, TrustLevel, PeerStatus, CachedPeer
- **D-11:** `cache.rs` contains: PeerCache struct with tokio::sync::RwLock, reqwest::Client, refresh logic, self-endorsement guard
- **D-12:** `error.rs` contains: FederationError enum (PeerFetchError, PeerParseError, etc.)

### Self-Endorsement Guard
- **D-13:** PeerCache::new() filters out endorsements where pubkey matches local node's pubkey
- **D-14:** Filtered self-endorsements logged at WARN level

### Dependency Changes
- **D-15:** reqwest moves from `[dev-dependencies]` to `[dependencies]` with `features = ["json"]`
- **D-16:** No other new dependencies for this phase

### Claude's Discretion
- Exact FederationError variant names and messages
- Whether CachedPeer fields use Option<Instant> or a separate enum for tracking state
- Test file organization within the federation module

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Data Model
- `src/registry/types.rs` — Current Registry, Endorsement (empty), Category, Source structs with deny_unknown_fields
- `src/registry/mod.rs` — Module re-exports pattern to follow

### Module Pattern
- `src/audit/mod.rs` + `src/audit/types.rs` + `src/audit/error.rs` + `src/audit/loader.rs` — Established 4-file module pattern
- `src/identity/` — Same pattern, most recent module added

### Eng Review Plan
- `~/.claude/plans/purring-marinating-taco.md` — Full eng-reviewed implementation plan with architecture decisions, test coverage diagram, and failure modes

### Design Doc
- `~/.gstack/projects/johnzilla-3goodsources/john-main-design-20260402-212653.md` — Approved design from /office-hours with endorsement data model spec

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/registry/types.rs` — Endorsement struct (line 33) to modify in place; Category and Source types to reuse in PeerRegistry
- `src/pubky/identity.rs:generate_or_load_keypair()` — Used by fork CLI (Phase 17), not this phase directly
- `Keypair::secret_key()` returns `[u8; 32]` — Verified in pkarr source, available for future phases

### Established Patterns
- All data modules follow mod.rs/types.rs/loader.rs/error.rs structure
- All types use `#[derive(Debug, Clone, Serialize, Deserialize)]`
- Local types use `#[serde(deny_unknown_fields)]` — peer types deliberately omit this
- Errors use thiserror with descriptive variants

### Integration Points
- `src/registry/types.rs:15` — Registry.endorsements field already typed as `Vec<Endorsement>`
- `src/lib.rs` — Will need `pub mod federation;` added
- `src/main.rs` — Will need federation module imported (Phase 16 wiring, not this phase)
- `Cargo.toml:36` — reqwest currently in dev-dependencies, move to dependencies

</code_context>

<specifics>
## Specific Ideas

No specific requirements — standard Rust data modeling. Follow established codebase patterns.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 15-federation-foundation*
*Context gathered: 2026-04-03*
