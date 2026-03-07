# Technology Stack — v2.0 Community Curation

**Project:** Three Good Sources (3GS) MCP Server
**Researched:** 2026-03-07
**Overall Confidence:** HIGH
**Scope:** Additions for audit log signing, identity schema, and contribution tracking

## Executive Summary

The v2.0 features (audit log, identity linking, contribution proposals) require minimal new dependencies. The existing stack already contains nearly everything needed. The server remains read-only with JSON files loaded at startup, so no database or write-path crates are required. The critical question -- how to sign audit entries -- is answered by the existing `pkarr` crate which wraps ed25519-dalek and already provides `Keypair::from_secret_key()` and access to signing primitives through the transitive `ed25519-dalek` dependency.

**Bottom line: Add `ed25519-dalek` as a direct dependency for audit entry signing, `chrono` for timestamps, and `sha2` for content hashing. No other new crates needed.**

## Existing Stack (No Changes Needed)

These are already in `Cargo.toml` and sufficient for v2.0:

| Technology | Version | v2.0 Use |
|------------|---------|----------|
| axum 0.8 | 0.8 | New GET routes (`/audit`, `/identities`, `/proposals`) with `Query` extractor for filtering |
| tokio | 1.49 | Async file loading for new JSON data files |
| serde + serde_json | 1.0 | Deserialize audit_log.json, identities.json, proposals.json |
| pkarr | 5.0 (keys feature) | Existing `Keypair` and `PublicKey` for curator identity |
| hex | 0.4 | Encode/decode signature bytes and hashes |
| thiserror | 2.0 | Error types for new modules |
| anyhow | 1.0 | Application-level error handling |
| tracing | 0.1 | Structured logging for new endpoints |
| tower-http | 0.6 (cors) | CORS already configured for new GET endpoints |
| schemars | 1.0 | JSON schema generation for new types |

## New Dependencies

### Required Additions

| Technology | Version | Purpose | Why This, Not Alternatives | Confidence |
|------------|---------|---------|---------------------------|------------|
| ed25519-dalek | 2.1 | Sign audit log entries, verify signatures | Already a transitive dependency via pkarr/curve25519-dalek. Making it direct gives clean access to `SigningKey::sign()` and `VerifyingKey::verify()` for arbitrary byte payloads. pkarr's `Keypair` is designed for DNS packet signing, not arbitrary message signing. | HIGH |
| chrono | 0.4 | ISO 8601 timestamps for audit entries | Audit entries need precise timestamps. The `time` crate is lighter but chrono has better serde integration and `DateTime::to_rfc3339()`. For a server that does UTC-only timestamps, chrono is the pragmatic choice. | HIGH |
| sha2 | 0.10 | SHA-256 content hashing for audit entries | Each audit entry should hash the content being attested (e.g., registry state hash). sha2 is the RustCrypto standard, zero additional transitive deps since curve25519-dalek already pulls in the digest ecosystem. | HIGH |

### Explicitly NOT Adding

| Technology | Why Not |
|------------|---------|
| sqlx / diesel / any database | Server is read-only, JSON files loaded at startup. No write path. |
| base64 | hex encoding is already available and sufficient for signatures/hashes |
| uuid | Audit entry IDs can be sequential integers or timestamp-based, no need for UUIDs |
| jsonschema (validator) | schemars generates schemas, serde's `deny_unknown_fields` + custom validation functions handle validation at load time (pattern already established in registry loader) |
| ring | ed25519-dalek is already in the dependency tree, adding ring would duplicate crypto |
| axum-extra | Basic `axum::extract::Query` is sufficient for filter params like `?action=add_source&after=2026-01-01` |
| reqwest (runtime) | No outbound HTTP calls needed. Server is read-only, all data comes from local JSON files. Already a dev-dependency for integration tests. |
| tokio-cron / scheduling | No background tasks. Audit log is pre-computed JSON, not generated at runtime. |

## Integration Points with Existing PKARR Signing

### Current State
The server loads a PKARR `Keypair` from `PKARR_SECRET_KEY` env var (or generates ephemeral). It uses `PublicKey` for the `/health` endpoint and registry curator identity. The `Keypair` is created in `main.rs` but NOT stored in `AppState` -- only the `PublicKey` is kept.

### What Needs to Change
For audit entry signing, the signing happens **offline** (curator signs entries before deploying), NOT at runtime. The server only needs to **verify** signatures, not create them.

**Architecture decision:** Audit entries are signed offline by the curator using a CLI tool or script, then committed to `audit_log.json`. The server loads and serves them. Verification uses `ed25519-dalek::VerifyingKey` with the curator's public key (already available in `AppState`).

This means:
1. **Server code** needs `ed25519-dalek` for `VerifyingKey::verify()` at load time (validate audit log integrity)
2. **CLI/script** (separate binary or cargo example) needs `ed25519-dalek` for `SigningKey::sign()` to create signed audit entries
3. `AppState` does NOT need to store the `Keypair` -- current design is preserved

### Signing Flow

```
OFFLINE (curator workstation):
  1. Curator edits registry.json, identities.json, or proposals.json
  2. CLI tool creates audit entry: { action, timestamp, content_hash, details }
  3. CLI tool signs entry with PKARR secret key via ed25519-dalek SigningKey
  4. Signed entry appended to audit_log.json
  5. All files committed and deployed

RUNTIME (server):
  1. Server loads audit_log.json
  2. Server verifies each entry's signature against curator pubkey
  3. Server serves verified entries via GET /audit
```

### Key Compatibility

pkarr `Keypair::from_secret_key(&[u8; 32])` uses the same 32-byte Ed25519 seed as `ed25519_dalek::SigningKey::from_bytes(&[u8; 32])`. The keys are interchangeable:

```rust
// These produce the same Ed25519 keypair:
let pkarr_keypair = pkarr::Keypair::from_secret_key(&secret_bytes);
let dalek_signing_key = ed25519_dalek::SigningKey::from_bytes(&secret_bytes);

// Public keys match:
// pkarr_keypair.public_key() inner bytes == dalek_signing_key.verifying_key().to_bytes()
```

This is because pkarr wraps ed25519-dalek internally. The `PKARR_SECRET_KEY` env var already holds the 32-byte hex-encoded seed, so the same secret key works for both pkarr identity and audit entry signing.

## Updated Cargo.toml Additions

```toml
# Add to [dependencies] — new for v2.0
ed25519-dalek = { version = "2.1", features = ["serde"] }
chrono = { version = "0.4", default-features = false, features = ["serde", "clock"] }
sha2 = "0.10"
```

**Note on ed25519-dalek version:** The project patches `curve25519-dalek` to use a git branch. ed25519-dalek 2.1 depends on curve25519-dalek 4.x, which should be compatible with the existing git patch. If there is a version conflict, pin to the same ed25519-dalek version that pkarr 5.0 uses transitively (check with `cargo tree -i ed25519-dalek`).

**Note on chrono features:** `default-features = false` avoids pulling in the `oldtime` feature and legacy APIs. `clock` provides `Utc::now()` for the CLI signing tool. `serde` enables `#[serde(with = "...")]` for DateTime fields.

## New Type Definitions (Informed by Stack)

The stack choices shape these types:

```rust
// Audit entry — signed offline, verified at load time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: u64,
    pub timestamp: String,              // ISO 8601 via chrono
    pub action: AuditAction,            // enum: add_source, update_source, etc.
    pub details: serde_json::Value,     // flexible payload
    pub content_hash: String,           // SHA-256 hex of attested content
    pub signature: String,              // ed25519 signature hex
    pub signer: String,                 // PKARR pubkey z32
}

// Identity — platform linking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub pkarr_pubkey: String,           // z32-encoded PKARR public key
    pub display_name: String,
    pub platforms: Vec<PlatformLink>,    // X, Nostr, GitHub links
    pub verified: bool,                 // curator-attested
}

// Proposal — community contribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: u64,
    pub status: ProposalStatus,         // enum: open, accepted, rejected
    pub proposer: String,               // PKARR pubkey or platform handle
    pub category: String,               // target category slug
    pub proposed_sources: Vec<Source>,   // reuse existing Source type
    pub votes: Vec<Vote>,
    pub created: String,                // ISO 8601
}
```

## axum Query Filtering

For filterable endpoints like `GET /audit?action=add_source&after=2026-01-01`, axum's built-in `Query` extractor handles this cleanly:

```rust
#[derive(Deserialize)]
pub struct AuditFilter {
    pub action: Option<String>,
    pub after: Option<String>,      // ISO 8601 date string
    pub before: Option<String>,
    pub limit: Option<usize>,
}

async fn audit_endpoint(
    State(state): State<Arc<AppState>>,
    Query(filter): Query<AuditFilter>,
) -> Json<Vec<AuditEntry>> {
    // Filter in-memory audit log
}
```

No additional crates needed -- axum 0.8's `Query` deserializes query strings into structs via serde.

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Signing | ed25519-dalek direct | Use pkarr Keypair for signing | pkarr's signing API is for DNS packets (SignedPacket), not arbitrary messages. Direct ed25519-dalek gives clean `sign(message)` semantics. |
| Timestamps | chrono 0.4 | time 0.3 | chrono has better serde integration and wider ecosystem adoption. For UTC-only ISO 8601 timestamps, both work, but chrono's `to_rfc3339()` is more ergonomic. |
| Timestamps | chrono 0.4 | Plain strings | Tempting for simplicity, but chrono gives validation at parse time and consistent formatting. Worth the small dependency. |
| Content hashing | sha2 (SHA-256) | blake3 | SHA-256 is the standard for content attestation. blake3 is faster but SHA-256 is more universally verifiable by external tools. |
| Content hashing | sha2 (SHA-256) | No hashing | Audit entries without content hashes can't prove what was attested. Hash is essential for integrity. |

## Sources

- [ed25519-dalek docs](https://docs.rs/ed25519-dalek/) — SigningKey and VerifyingKey API (HIGH confidence)
- [chrono crate](https://crates.io/crates/chrono) — DateTime, UTC, RFC 3339 formatting (HIGH confidence)
- [sha2 crate](https://docs.rs/sha2/) — SHA-256 hashing (HIGH confidence)
- [axum Query extractor](https://docs.rs/axum/latest/axum/extract/struct.Query.html) — Query parameter deserialization (HIGH confidence)
- [pkarr GitHub](https://github.com/pubky/pkarr) — PKARR protocol implementation (HIGH confidence)
- [State of the Crates 2025](https://ohadravid.github.io/posts/2024-12-state-of-the-crates/) — chrono vs time ecosystem assessment (MEDIUM confidence)

## Confidence Assessment

| Area | Level | Reason |
|------|-------|--------|
| ed25519-dalek compatibility | HIGH | Already a transitive dependency; same key format as pkarr |
| chrono suitability | HIGH | Well-established, serde support verified |
| sha2 suitability | HIGH | RustCrypto standard, already in dependency tree via dalek |
| No new crates beyond 3 | HIGH | Server is read-only JSON loader; all complexity is in schema design and offline tooling |
| Key interchangeability | MEDIUM | pkarr wraps ed25519-dalek but exact internal API not inspected (cargo registry access denied). Verify with `cargo tree` and a test. |
