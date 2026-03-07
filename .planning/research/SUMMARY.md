# Project Research Summary

**Project:** Three Good Sources (3GS) MCP Server
**Domain:** Curated trust registry with audit trail, identity linking, and community contributions
**Researched:** 2026-03-07
**Confidence:** HIGH

## Executive Summary

Three Good Sources v2.0 extends a read-only Rust MCP server with three new data domains: an append-only audit log for curation transparency, cross-platform identity linking for curator credibility, and a community contribution/proposal system. The existing architecture -- JSON files loaded at startup, served via Arc in AppState, exposed through both REST and MCP endpoints -- is perfectly suited for these additions. The v2.0 work is fundamentally additive: three new modules following the exact pattern established by the existing `registry/` module, three new JSON data files, five new HTTP routes, and three new MCP tools. No architectural changes are required.

The recommended approach is to build each data domain as an independent module (audit, identity, contributions) in dependency order. The stack additions are minimal: only `ed25519-dalek`, `chrono`, and `sha2` as new crate dependencies, all of which are already transitive dependencies via `pkarr`. The key design decision -- offline signing with runtime verification -- preserves the server's read-only philosophy. All signing happens on the curator's workstation; the server only loads and serves pre-signed data.

The primary risks are schema evolution conflicts (the existing `deny_unknown_fields` pattern will break append-only data), signature canonicalization failures (signing JSON without a defined canonical format produces unverifiable signatures), and AppState bloat as three new data stores join the existing registry. All three risks have clear mitigations identified in research. The most important decision to make early: use `#[serde(default)]` instead of `deny_unknown_fields` for all new data types, and define the audit entry signing format specification before writing any entries.

## Key Findings

### Recommended Stack

The existing stack (axum 0.8, tokio, serde, pkarr) handles nearly everything. Only three new crates are needed, all already present as transitive dependencies.

**New dependencies:**
- **ed25519-dalek 2.1**: Sign/verify audit entries -- pkarr wraps this internally but its API targets DNS packets, not arbitrary messages. Direct dependency gives clean `SigningKey::sign()` / `VerifyingKey::verify()` semantics
- **chrono 0.4**: ISO 8601 timestamps for audit entries -- better serde integration than the `time` crate, `to_rfc3339()` convenience
- **sha2 0.10**: SHA-256 content hashing for audit entry attestation -- RustCrypto standard, zero new transitive deps

**Explicitly not adding:** No database (server is read-only), no reqwest (no outbound HTTP), no axum-extra (built-in `Query` extractor suffices), no ring (would duplicate ed25519-dalek).

### Expected Features

**Must have (table stakes):**
- Append-only `audit_log.json` with `GET /audit` endpoint and `get_audit_log` MCP tool, filterable by action/category/date
- `identities.json` mapping PKARR pubkeys to GitHub/X/Nostr platform claims with proof URLs, served via `GET /identities` and `get_identity` MCP tool
- `contributions.json` for community source proposals with status lifecycle, served via `GET /proposals` and MCP tools
- Human-readable descriptions on audit entries, curator rationale on proposal decisions

**Should have (differentiators):**
- Ed25519-signed audit entries (huge trust differentiator, but adds canonicalization complexity)
- Hash chain linking audit entries (tamper-evident, detects history modification)
- Audit log RSS/Atom feed (low effort, high transparency value)
- Proposal vote/endorsement counts from GitHub reactions

**Defer to v2.1+:**
- Cross-platform identity proof verification at runtime (fragile external HTTP calls)
- Automated human vs bot vote detection (unsolved research problem; use identity-status-based classification instead)
- Write API for submissions (antithetical to read-only architecture)
- OAuth-based identity verification, user accounts, Nostr relay infrastructure

### Architecture Approach

Three new parallel modules (`src/audit/`, `src/identity/`, `src/contributions/`) each containing `mod.rs`, `types.rs`, `loader.rs`, and `error.rs` -- directly replicating the established `src/registry/` pattern. Each loads its JSON file via `tokio::fs::read_to_string`, deserializes with serde, validates, wraps in `Arc<T>`, and stores in AppState. New routes use axum's `Query` and `Path` extractors (new patterns for this project but well-documented). The `handle_tool_call` function adds three new match arms for the new MCP tools.

**Major components:**
1. **`src/audit/`** -- Load, validate, and serve audit log entries; introduces query parameter filtering pattern
2. **`src/identity/`** -- Load, validate, and serve identity claims; introduces path parameter extraction (`/identities/{pubkey}`)
3. **`src/contributions/`** -- Load, validate, and serve proposals with status lifecycle; most complex schema (votes, state machine)
4. **Extended `server.rs`** -- AppState gains three new `Arc<T>` fields, router gains five new routes
5. **Extended `mcp/tools.rs`** -- Three new tool implementations, tools/list grows from 4 to 7

### Critical Pitfalls

1. **`deny_unknown_fields` breaking schema evolution** -- The existing pattern works for stable registry types but will prevent adding fields to append-only audit entries. Use `#[serde(default)]` on all new types instead. Decision must be made before any JSON files are written.

2. **Audit signatures not verifiable** -- Signing pretty-printed JSON or relying on serde_json's non-deterministic field ordering produces signatures that can never be independently verified. Define a canonical signing format (sorted keys, compact JSON, or explicit field concatenation) before writing the first entry.

3. **Identity claims without proof artifacts** -- Mapping pubkeys to platform handles without storing verification evidence (proof URLs, tweet IDs, gist URLs) makes the identity system meaningless. Every claim must have a `proof_url` and `verification_method` from day one.

4. **Proposal lifecycle without state machine** -- Proposals need defined state transitions (submitted -> under_review -> approved -> applied) with validation preventing invalid transitions. Without this, proposals end up in undefined states.

5. **Startup time regression** -- Loading four JSON files sequentially with full hash chain and signature validation could slow startup. Use `tokio::try_join!` for parallel loading and consider graceful degradation (core registry always loads; new features optional).

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Audit Log Module
**Rationale:** Simplest schema (flat list of entries), establishes the "new module" pattern that identity and contributions will copy. Delivers immediate transparency value. All three new crate dependencies (`ed25519-dalek`, `sha2`, `chrono`) land here.
**Delivers:** `src/audit/` module (4 files), `audit_log.json` with retroactive entries for existing 30 sources, `GET /audit` endpoint with query filtering, `get_audit_log` MCP tool
**Addresses:** Audit log table stakes (all 6 features), timestamp standardization (RFC 3339)
**Avoids:** Pitfall 1 (set `serde(default)` precedent), Pitfall 2 (define canonical signing format), Pitfall 9 (size monitoring), Pitfall 12 (timestamp convention), Pitfall 14 (pagination params)

### Phase 2: Identity Linking Module
**Rationale:** Introduces path parameter routes (`/identities/{pubkey}`) and the first new MCP tool. This pattern is then reused by contributions. Links the curator's PKARR pubkey to platform handles for credibility.
**Delivers:** `src/identity/` module (4 files), `identities.json` with curator's own identity, `GET /identities` and `GET /identities/{pubkey}` endpoints, `get_identity` MCP tool
**Addresses:** Identity linking table stakes (all 6 features), proof URL storage
**Avoids:** Pitfall 4 (require proof artifacts on every claim), Pitfall 8 (MCP backward compatibility regression gate)

### Phase 3: Community Contributions Module
**Rationale:** Most complex schema (proposals with votes, status lifecycle, submitter metadata). Reuses all patterns established in phases 1-2. Depends on having a GitHub Issues template for submissions.
**Delivers:** `src/contributions/` module (4 files), `contributions.json`, `GET /proposals` and `GET /proposals/{id}` endpoints, `list_proposals` and `get_proposal` MCP tools
**Addresses:** Contributions table stakes (all 6 features), proposal state machine, vote type classification
**Avoids:** Pitfall 6 (classify by identity status, not behavioral detection), Pitfall 10 (define state machine before implementing)

### Phase 4: Integration, Polish, and Deploy
**Rationale:** Cross-cutting concerns that span all three new modules. Verifies backward compatibility, updates documentation and deployment config.
**Delivers:** Updated landing page HTML, `.env.example` and Dockerfile with new env vars, parallel startup loading (`tokio::try_join!`), updated `get_provenance` tool mentioning identity linking, end-to-end integration tests across all new endpoints
**Addresses:** Pitfall 7 (startup time), Pitfall 8 (full v1.0 test regression), Pitfall 11 (CORS verification), Pitfall 13 (curator workflow documentation)

### Phase Ordering Rationale

- **Audit first** because it has no dependencies on the other two modules, establishes the module pattern, and is the lowest-risk highest-value feature
- **Identity second** because it introduces the path parameter pattern needed by contributions, and the curator's identity entry can be referenced in audit log entries
- **Contributions third** because it is the most complex schema and benefits from patterns established in phases 1-2
- **Integration last** because cross-cutting concerns (startup optimization, backward compatibility, deployment config) only make sense after all modules exist
- All three data modules are architecturally independent -- they share AppState but have no code-level dependencies on each other

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 1 (Audit Log):** Signature canonicalization format needs a concrete specification. The research identifies the problem and two approaches (sorted-key JSON vs field concatenation) but the choice needs validation with actual ed25519-dalek signing code. Also: whether to include hash chain in v2.0 table stakes or defer to v2.1.
- **Phase 3 (Contributions):** Proposal state machine transitions need precise definition. The research suggests 6 states but the actual workflow (GitHub Issues -> curator review -> JSON update) may simplify this.

Phases with standard patterns (skip research-phase):
- **Phase 2 (Identity):** NIP-39 / Keybase proof model is well-documented. Schema is straightforward. Path parameter routes are standard axum.
- **Phase 4 (Integration):** All patterns are established by this point. Standard deployment and testing work.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Only 3 new crates, all already transitive deps. Versions and feature flags verified. |
| Features | HIGH | Clear table stakes / differentiator / anti-feature separation. Schemas fully specified with JSON examples. |
| Architecture | HIGH | Based on direct analysis of all 20 existing source files. New modules replicate established patterns exactly. |
| Pitfalls | MEDIUM-HIGH | Cryptographic signing pitfalls well-documented in literature. Identity linking patterns based on NIP-39/Keybase (proven). AppState bloat and startup regression are pragmatic concerns, not speculative. |

**Overall confidence:** HIGH

### Gaps to Address

- **ed25519-dalek / pkarr key interchangeability:** Research confirms they use the same 32-byte Ed25519 seed, but this was not tested with actual code. Verify with `cargo tree -i ed25519-dalek` and a round-trip signing test in phase 1.
- **Signing canonicalization choice:** Two approaches identified (sorted-key compact JSON vs explicit field concatenation). Need to pick one and write a verification test before committing to the format.
- **AppState refactoring vs flat fields:** PITFALLS.md recommends grouping new data under `CommunityState`; ARCHITECTURE.md recommends flat `Arc<T>` fields matching existing style. Recommendation: start flat (matches codebase conventions), refactor later if a v3.0 adds more domains.
- **Hash chain in v2.0 scope:** FEATURES.md lists it as a differentiator (defer to v2.1). PITFALLS.md assumes it exists and warns about integrity. Recommendation: include `previous_hash` as an optional field in the schema but do not validate the chain in v2.0. This preserves the option without adding complexity.
- **Graceful degradation vs fail-fast:** PITFALLS.md suggests optional loading (serve without audit if it fails). Current codebase uses fail-fast (missing registry = server won't start). Recommendation: fail-fast for v2.0 (all files are curator-managed and committed to git; they should always be valid).

## Sources

### Primary (HIGH confidence)
- Direct codebase analysis of all 20 `.rs` source files, Cargo.toml, and existing test suite
- [ed25519-dalek docs](https://docs.rs/ed25519-dalek/) -- SigningKey/VerifyingKey API
- [axum 0.8 extractors](https://docs.rs/axum/latest/axum/extract/) -- Query, Path, State patterns
- [NIP-39: External Identities in Profiles](https://github.com/nostr-protocol/nips/blob/master/39.md) -- identity proof model
- [pkarr GitHub](https://github.com/pubky/pkarr) -- PKARR protocol, keypair internals
- PROJECT.md v2.0 feature requirements

### Secondary (MEDIUM confidence)
- [Martin Fowler: Audit Log Pattern](https://martinfowler.com/eaaDev/AuditLog.html) -- canonical audit log design
- [Keybase Proof System](https://keybase.io/blog/keybase-proofs-for-mastodon-and-everyone) -- cross-platform identity verification prior art
- [Ed25519 validation criteria](https://hdevalence.ca/blog/2020-10-04-its-25519am/) -- signature verification semantics
- [Mattermost Audit Log JSON Schema](https://docs.mattermost.com/comply/embedded-json-audit-log-schema.html) -- real-world audit log schema reference
- [State of the Crates 2025](https://ohadravid.github.io/posts/2024-12-state-of-the-crates/) -- chrono vs time assessment
- [Sybil Attack Resistance in Voting](https://arxiv.org/abs/2407.01844) -- why automated bot detection should be deferred

### Tertiary (LOW confidence)
- [Append-Only Logs: Design Patterns](https://medium.com/@komalshehzadi/append-only-logs-the-immutable-diary-of-data-58c36a871c7c) -- hash chain patterns (blog post)
- [serde_json memory usage](https://github.com/serde-rs/json/issues/635) -- large file parsing concerns (GitHub issue)

---
*Research completed: 2026-03-07*
*Ready for roadmap: yes*
