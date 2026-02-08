# Phase 8: Tech Debt Cleanup - Context

**Gathered:** 2026-02-08
**Status:** Ready for planning

<domain>
## Phase Boundary

Clean up fragile dependencies and dead code before migration. Code-only changes validated locally (build + tests). No deployment to any platform in this phase.

</domain>

<decisions>
## Implementation Decisions

### curve25519-dalek strategy
- One attempt: remove the `[patch.crates-io]` section from Cargo.toml
- Build locally — if it compiles and tests pass, the patch is gone
- If build fails, revert the patch immediately and move on (keep existing behavior)
- No extended investigation — single attempt, pass or fail

### Dead code removal scope
- Remove McpError enum (scoped requirement)
- Quick sweep: run cargo clippy/warnings, remove any obviously unused code
- Also remove unused dependencies from Cargo.toml (not just dead Rust code)
- Don't go hunting — if it's flagged by tooling, remove it; otherwise leave it

### Validation approach
- Local build + all tests passing is sufficient (`cargo build && cargo test`)
- No deployment to any platform in this phase — this milestone is migrating away from Render
- If all 72+ tests pass, phase is validated

### Commit strategy
- One commit per fix, not a single monolith commit
- Separate commits for: dependency patch removal, dead code/McpError removal, unused dep cleanup
- Each commit should independently build and pass tests
- Easy to revert individual changes if something breaks downstream

### Claude's Discretion
- Order of operations (which fix to tackle first)
- Exact clippy lints to act on vs ignore
- Whether to update any adjacent code that becomes simpler after removals

</decisions>

<specifics>
## Specific Ideas

- User is not deeply familiar with the curve25519-dalek patch — keep changes minimal and explain what happened in commit messages
- "Quick sweep" means tooling-driven, not manual code review — trust compiler warnings and clippy

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 08-tech-debt-cleanup*
*Context gathered: 2026-02-08*
