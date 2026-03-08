# Phase 12: Audit Log - Research

**Researched:** 2026-03-07
**Domain:** Cryptographic audit logging with Ed25519 signatures and hash chains in Rust
**Confidence:** HIGH

## Summary

Phase 12 adds an append-only audit log to the 3GS server. The audit log is a JSON file (`audit_log.json`) containing signed, hash-chained entries that record every registry change. The server loads and validates the log at startup (fail-fast), serves it via GET /audit with query filtering, and exposes it through a `get_audit_log` MCP tool.

The implementation closely mirrors the existing `src/registry/` module pattern (4-file structure: mod.rs, types.rs, loader.rs, error.rs). The key cryptographic dependencies -- ed25519-dalek and sha2 -- are already in the dependency tree via pkarr. The main new dependencies are `uuid` (entry IDs) and `chrono` (timestamp parsing/validation). The pkarr Keypair's `secret_key()` method returns an ed25519-dalek `SecretKey` type directly, so offline signing tools can reuse the existing PKARR_SECRET_KEY for audit entry signatures.

**Primary recommendation:** Replicate the registry module pattern exactly. Use ed25519-dalek directly (already a transitive dependency) for signature verification at load time. Use the `hex` crate (already a dependency) for signature encoding. Keep the audit log as a single JSON array file for simplicity.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Signing format is field concatenation: `timestamp|action|category|sha256(data_json)|actor_pubkey`
- Signatures hex-encoded (consistent with PKARR pubkey display convention)
- SHA-256 hash of data JSON (sorted-key compact) covers variable-schema data
- Server verifies all signatures at load time -- invalid entries cause startup failure
- 40 retroactive entries: 10 category_added + 30 source_added
- All retroactive entries timestamped to 2026-02-03T00:00:00Z
- Standard action types for retroactive entries (not a backfill type)
- Ordering: 10 category_added first, then 30 source_added grouped by category
- Hash chain starts from entry #1, previous_hash is null for genesis
- GET /audit returns raw JSON array, no wrapper object
- Query params: since (ISO timestamp), category (slug), action (enum value)
- No pagination for v2.0
- MCP tool get_audit_log accepts same filter params as REST endpoint
- Action enum includes future types now (identity_registered, identity_updated, proposal_submitted, proposal_status_changed, vote_cast)
- Action type is strict Rust enum with serde rename (snake_case in JSON)
- category field is Option<String> -- null for non-registry actions
- Entry ID: UUID v4
- Use #[serde(default)] not deny_unknown_fields for audit types
- Hash chain validation deferred -- field exists but no runtime chain integrity check

### Claude's Discretion
- Exact uuid crate version and features (v4 generation)
- Audit log file format (single JSON array vs JSONL)
- Specific validation rules in loader beyond signature verification
- Test fixture design for invalid audit entries

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| AUDIT-01 | Every registry change creates an append-only audit log entry with timestamp, action, category, data, and actor | Entry schema defined with AuditEntry struct, action enum, canonical signing format |
| AUDIT-02 | Each audit entry is signed by the actor's Ed25519 key using a defined canonical format | ed25519-dalek already in tree via pkarr, canonical format locked: `timestamp\|action\|category\|sha256(data_json)\|actor_pubkey` |
| AUDIT-03 | Each audit entry includes a previous_hash field linking to the prior entry (hash chain) | SHA-256 hash chain with null genesis, sha2 already in dependency tree |
| AUDIT-04 | GET /audit endpoint returns audit entries filterable by since, category, and action | Axum Query extractor with serde Deserialize for optional filter params |
| AUDIT-05 | get_audit_log MCP tool returns audit entries with the same filtering as REST endpoint | Follows existing MCP tool pattern with schemars JsonSchema derive for params |
| AUDIT-06 | Retroactive audit entries exist for all 30 existing sources from v1.0 | 40 entries total (10 categories + 30 sources), timestamped 2026-02-03T00:00:00Z, script or manual creation |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| ed25519-dalek | 3.0.0-pre.6 | Ed25519 signature verification | Already in dependency tree via pkarr 5.0.2; provides SigningKey and VerifyingKey |
| sha2 | 0.11.0-rc.5 | SHA-256 hashing for canonical data and hash chain | Already in dependency tree via ed25519-dalek |
| uuid | 1.x | UUID v4 entry identifiers | Standard Rust UUID crate; features: `v4`, `serde` |
| chrono | 0.4.x | ISO 8601 timestamp parsing and validation | Standard Rust datetime; features: `serde` |

### Supporting (already in project)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| hex | 0.4 | Hex encoding of signatures and hashes | Already a dependency; used for PKARR key handling |
| serde / serde_json | 1.0 | Serialization of audit entries | Already a dependency; derive Serialize/Deserialize |
| schemars | 1.x | JSON Schema for MCP tool params | Already a dependency; derive JsonSchema |
| thiserror | 2.0 | Error type definitions | Already a dependency; matches registry error pattern |
| axum | 0.8 | GET /audit route with Query extractor | Already a dependency |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Single JSON array | JSONL (one entry per line) | JSON array is simpler to load/parse with serde_json; JSONL better for append-only writes but server is read-only |
| chrono | time crate | chrono has better serde integration for ISO 8601; time is lighter but less ecosystem support |

**Recommendation for Claude's discretion items:**
- **uuid crate:** Use `uuid = { version = "1", features = ["v4", "serde"] }` -- latest stable 1.x
- **File format:** Single JSON array (not JSONL). Rationale: server loads entire file at startup anyway, serde_json::from_str directly deserializes Vec<AuditEntry>, consistent with registry.json pattern
- **Loader validation:** Verify signatures, validate UUID format (serde handles this), check timestamp parses as valid ISO 8601, verify previous_hash matches SHA-256 of prior entry's canonical form (chain integrity at load time is cheap and catches corruption)
- **Test fixtures:** Create minimal valid and invalid audit log JSON files in tests/fixtures/; test invalid signature, broken chain, missing fields

**Installation:**
```bash
# uuid and chrono are new; ed25519-dalek and sha2 need explicit dependency for direct use
cargo add uuid --features v4,serde
cargo add chrono --features serde
cargo add ed25519-dalek --features serde
cargo add sha2
```

**Note on ed25519-dalek and sha2 versions:** These are pre-release versions (3.0.0-pre.6 and 0.11.0-rc.5) pulled in by pkarr's dependency chain and the curve25519-dalek git patch. When adding them as direct dependencies, Cargo will unify to the versions already in the tree. Specify exact version constraints matching what's in Cargo.lock to avoid resolution conflicts:
```toml
ed25519-dalek = { version = "=3.0.0-pre.6", features = ["serde"] }
sha2 = "=0.11.0-rc.5"
```

## Architecture Patterns

### Recommended Project Structure
```
src/
  audit/
    mod.rs          # Re-exports: AuditEntry, AuditAction, AuditError, load
    types.rs        # AuditEntry, AuditAction enum, AuditFilterParams
    loader.rs       # load() -> Result<Vec<AuditEntry>, AuditError>
    error.rs        # AuditError enum with thiserror
  server.rs         # Add audit_log: Arc<Vec<AuditEntry>> to AppState, add GET /audit route
  mcp/
    tools.rs        # Add get_audit_log tool definition and handler
    handler.rs      # Pass audit_log to tool handler
  main.rs           # Load audit_log.json at startup
  lib.rs            # Add pub mod audit
data/
  audit_log.json    # 40 retroactive entries (or root-level alongside registry.json)
tests/
  fixtures/
    audit_log.json          # Valid test audit log
    audit_log_invalid.json  # Invalid entries for error tests
  integration_audit.rs      # Integration tests for GET /audit
  common/mod.rs             # Update spawn_test_server with audit_log
```

### Pattern 1: Module Structure (mirror registry)
**What:** 4-file module matching existing registry pattern
**When to use:** Always for new data modules in this project
**Example:**
```rust
// src/audit/mod.rs
pub mod error;
pub mod loader;
pub mod types;

pub use error::AuditError;
pub use loader::load;
pub use types::{AuditEntry, AuditAction};
```

### Pattern 2: Entry Schema with Serde
**What:** AuditEntry struct with future-proof serde attributes
**When to use:** Defining the audit entry type
**Example:**
```rust
// src/audit/types.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]  // NOT deny_unknown_fields -- schema must evolve
pub struct AuditEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub action: AuditAction,
    pub category: Option<String>,
    pub data: serde_json::Value,
    pub actor: String,           // hex-encoded public key
    pub signature: String,       // hex-encoded Ed25519 signature
    pub previous_hash: Option<String>,  // hex-encoded SHA-256; null for genesis
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    SourceAdded,
    SourceUpdated,
    SourceRemoved,
    CategoryAdded,
    CategoryUpdated,
    CategoryRemoved,
    IdentityRegistered,
    IdentityUpdated,
    ProposalSubmitted,
    ProposalStatusChanged,
    VoteCast,
}
```

### Pattern 3: Canonical Signing Format
**What:** Deterministic string construction for signature verification
**When to use:** Both in offline signing tool and server verification
**Example:**
```rust
use sha2::{Sha256, Digest};

fn canonical_message(entry: &AuditEntry) -> String {
    // Sort-key compact JSON of data field
    let data_json = serde_json::to_string(&entry.data).unwrap();
    let data_hash = hex::encode(Sha256::digest(data_json.as_bytes()));

    let category_str = entry.category.as_deref().unwrap_or("");

    format!(
        "{}|{}|{}|{}|{}",
        entry.timestamp.to_rfc3339(),
        serde_json::to_value(&entry.action).unwrap().as_str().unwrap(),
        category_str,
        data_hash,
        entry.actor
    )
}
```

### Pattern 4: Signature Verification at Load Time
**What:** Verify every entry's Ed25519 signature during startup
**When to use:** In the audit loader
**Example:**
```rust
use ed25519_dalek::{VerifyingKey, Signature, Verifier};

fn verify_signature(entry: &AuditEntry) -> Result<(), AuditError> {
    let pubkey_bytes = hex::decode(&entry.actor)
        .map_err(|_| AuditError::InvalidActorKey { id: entry.id })?;
    let pubkey_array: [u8; 32] = pubkey_bytes.try_into()
        .map_err(|_| AuditError::InvalidActorKey { id: entry.id })?;
    let verifying_key = VerifyingKey::from_bytes(&pubkey_array)
        .map_err(|_| AuditError::InvalidActorKey { id: entry.id })?;

    let sig_bytes = hex::decode(&entry.signature)
        .map_err(|_| AuditError::InvalidSignature { id: entry.id })?;
    let sig_array: [u8; 64] = sig_bytes.try_into()
        .map_err(|_| AuditError::InvalidSignature { id: entry.id })?;
    let signature = Signature::from_bytes(&sig_array);

    let message = canonical_message(entry);
    verifying_key.verify(message.as_bytes(), &signature)
        .map_err(|_| AuditError::SignatureVerificationFailed { id: entry.id })
}
```

### Pattern 5: Axum Query Filtering
**What:** Optional query parameters for GET /audit
**When to use:** The audit endpoint
**Example:**
```rust
use axum::extract::Query;

#[derive(Debug, Deserialize)]
pub struct AuditFilterParams {
    pub since: Option<String>,     // ISO 8601 timestamp
    pub category: Option<String>,  // category slug
    pub action: Option<String>,    // action enum value (snake_case)
}

async fn audit_endpoint(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AuditFilterParams>,
) -> (StatusCode, [(HeaderName, &'static str); 1], String) {
    let entries = filter_audit_entries(&state.audit_log, &params);
    // serialize and return...
}
```

### Anti-Patterns to Avoid
- **deny_unknown_fields on audit types:** Decision is to use `#[serde(default)]` for schema evolution. Historical entries must remain valid as schema grows.
- **Storing audit log in a HashMap:** Audit entries are ordered (hash chain). Use `Vec<AuditEntry>` to preserve insertion order.
- **Wrapping response in an object:** Decision is raw JSON array response, matching the simplicity of the registry endpoint pattern.
- **Building a custom signing implementation:** Use ed25519-dalek directly. The crypto is standard Ed25519.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Ed25519 signatures | Custom signing code | ed25519-dalek VerifyingKey/Signature | Cryptographic correctness; already in dependency tree |
| SHA-256 hashing | Manual hash implementation | sha2::Sha256 with Digest trait | Already in dependency tree; battle-tested |
| UUID generation | Random ID generation | uuid::Uuid::new_v4() | Proper v4 UUID format with serde support |
| ISO 8601 parsing | String parsing with regex | chrono::DateTime<Utc> with serde | Handles timezone, validation, comparison |
| Hex encoding/decoding | Manual byte conversion | hex crate (already a dependency) | Consistent with existing PKARR key handling |
| Query parameter extraction | Manual URL parsing | axum::extract::Query<T> | Automatic deserialization, 400 on bad params |

**Key insight:** The cryptographic stack (ed25519-dalek, sha2, hex) is already in the project's dependency tree via pkarr. Adding them as direct dependencies costs zero additional compile time.

## Common Pitfalls

### Pitfall 1: Pre-release Dependency Version Conflicts
**What goes wrong:** Adding `ed25519-dalek = "2"` or `sha2 = "0.10"` creates version conflicts with pkarr's transitive dependencies
**Why it happens:** pkarr 5.0.2 depends on ed25519-dalek 3.0.0-pre.6 and sha2 0.11.0-rc.5 (pre-release versions via curve25519-dalek git patch)
**How to avoid:** Pin exact versions matching Cargo.lock: `ed25519-dalek = "=3.0.0-pre.6"` and `sha2 = "=0.11.0-rc.5"`
**Warning signs:** Cargo resolution errors mentioning incompatible versions of curve25519-dalek or digest

### Pitfall 2: Non-Deterministic Canonical Format
**What goes wrong:** Signature verification fails because canonical message differs between signing and verification
**Why it happens:** JSON key ordering, timestamp formatting, or category null handling differs
**How to avoid:** Use `serde_json::to_string()` (not `to_string_pretty()`) for data JSON hash; always use `to_rfc3339()` for timestamp; use empty string for null category
**Warning signs:** Signatures that verify locally but fail on server, or vice versa

### Pitfall 3: serde_json Key Ordering
**What goes wrong:** `serde_json::to_string(&entry.data)` produces different key ordering on different runs or platforms
**Why it happens:** serde_json::Value uses BTreeMap for objects, which IS deterministic (sorted by key). But if data is constructed differently, field order may vary.
**How to avoid:** The data field is `serde_json::Value` -- BTreeMap ordering is stable. Parse/re-serialize through Value to normalize. The canonical format hashes the data JSON string, so consistency is critical.
**Warning signs:** Hash chain breaks after re-serialization

### Pitfall 4: Actor Key Format Mismatch
**What goes wrong:** Audit entry actor field uses z32-encoded pubkey but verification expects hex-encoded bytes
**Why it happens:** PKARR uses z32 encoding for public keys, but the decision says hex-encoded for consistency
**How to avoid:** Store actor as hex-encoded 32-byte public key (64 hex chars). This is the raw Ed25519 public key bytes, NOT the z32 format. Document this clearly.
**Warning signs:** Key decoding failures in verify_signature

### Pitfall 5: Chrono Timestamp Precision
**What goes wrong:** `2026-02-03T00:00:00Z` serializes to `2026-02-03T00:00:00+00:00` with chrono, breaking canonical format
**Why it happens:** chrono's `to_rfc3339()` uses `+00:00` suffix, not `Z`
**How to avoid:** Use `to_rfc3339_opts(chrono::SecondsFormat::Secs, true)` to force `Z` suffix. Or use a custom serde format. The canonical signing format must specify which representation is authoritative.
**Warning signs:** Signatures fail because timestamp formatting differs

### Pitfall 6: Forgetting to Update MCP Handler Signature
**What goes wrong:** MCP handler doesn't have access to audit_log, can't serve get_audit_log tool
**Why it happens:** McpHandler::new() currently takes (registry, match_config, pubkey_z32) -- needs audit_log added
**How to avoid:** Add `audit_log: Arc<Vec<AuditEntry>>` to McpHandler struct; update new(), handle_tool_call() to pass it through
**Warning signs:** Compile errors when adding get_audit_log tool

## Code Examples

### Loading and Verifying Audit Log
```rust
// src/audit/loader.rs
use super::{AuditEntry, AuditError};
use ed25519_dalek::{Verifier, VerifyingKey, Signature};
use sha2::{Sha256, Digest};
use std::path::Path;
use tokio::fs;

pub async fn load(path: impl AsRef<Path>) -> Result<Vec<AuditEntry>, AuditError> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path).await
        .map_err(|e| AuditError::FileRead {
            path: path.display().to_string(),
            error: e.to_string(),
        })?;

    let entries: Vec<AuditEntry> = serde_json::from_str(&contents)
        .map_err(|e| AuditError::JsonParse {
            path: path.display().to_string(),
            error: e.to_string(),
        })?;

    // Verify all signatures
    for entry in &entries {
        verify_signature(entry)?;
    }

    tracing::info!(entries = entries.len(), "Audit log loaded successfully");
    Ok(entries)
}
```

### Filter Function for Endpoint and MCP Tool
```rust
fn filter_entries(
    entries: &[AuditEntry],
    since: Option<&DateTime<Utc>>,
    category: Option<&str>,
    action: Option<&AuditAction>,
) -> Vec<&AuditEntry> {
    entries.iter()
        .filter(|e| since.map_or(true, |s| e.timestamp >= *s))
        .filter(|e| category.map_or(true, |c| e.category.as_deref() == Some(c)))
        .filter(|e| action.map_or(true, |a| std::mem::discriminant(&e.action) == std::mem::discriminant(a)))
        .collect()
}
```

### MCP Tool Definition
```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetAuditLogParams {
    /// Optional ISO 8601 timestamp to filter entries after this time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
    /// Optional category slug to filter by
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Optional action type to filter by (e.g., "source_added", "category_added")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
}
```

### Offline Signing (for generating audit_log.json)
```rust
// This runs as a script/binary, NOT on the server
use ed25519_dalek::SigningKey;
use sha2::{Sha256, Digest};

fn sign_entry(entry: &mut AuditEntry, secret_key: &[u8; 32]) {
    let signing_key = SigningKey::from_bytes(secret_key);
    let message = canonical_message(entry);
    let signature = signing_key.try_sign(message.as_bytes()).unwrap();
    entry.signature = hex::encode(signature.to_bytes());
    entry.actor = hex::encode(signing_key.verifying_key().to_bytes());
}

fn hash_entry(entry: &AuditEntry) -> String {
    // Hash the canonical JSON representation of the entry
    let json = serde_json::to_string(entry).unwrap();
    hex::encode(Sha256::digest(json.as_bytes()))
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| ed25519-dalek 1.x Keypair struct | ed25519-dalek 2.x+ SigningKey/VerifyingKey | v2.0 (2023) | Type renames; same underlying crypto |
| sha2 0.10 stable | sha2 0.11.0-rc (via dalek ecosystem) | Pre-release | API same (Digest trait); version pinning required |
| pkarr independent keys | pkarr wraps ed25519-dalek internally | pkarr 5.0 | Keys interoperable; same 32-byte secret key format |

**Deprecated/outdated:**
- ed25519-dalek `Keypair` struct: renamed to `SigningKey` in 2.x+
- ed25519-dalek `PublicKey`: renamed to `VerifyingKey` in 2.x+

## Open Questions

1. **Audit log file location**
   - What we know: registry.json is at project root, loaded via REGISTRY_PATH env var
   - What's unclear: Should audit_log.json follow same pattern (AUDIT_LOG_PATH env var) or use a convention (same directory as registry)?
   - Recommendation: Add AUDIT_LOG_PATH to Config struct following REGISTRY_PATH pattern. Consistent, explicit.

2. **Hash chain entry hash scope**
   - What we know: previous_hash links entries. Need to define what gets hashed.
   - What's unclear: Hash the entire JSON entry? Just certain fields? The canonical signing message?
   - Recommendation: Hash the entire serialized JSON of the previous entry (compact, sorted keys via serde_json::to_string). This is the simplest and most tamper-evident approach.

3. **ed25519-dalek Signer trait import**
   - What we know: ed25519-dalek 3.0.0-pre.6 uses trait-based signing
   - What's unclear: Whether `try_sign` requires the `Signer` trait import or is a direct method
   - Recommendation: Import `use ed25519_dalek::Signer;` to bring the trait into scope. Verify with a unit test early.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test + cargo test |
| Config file | Cargo.toml (already configured) |
| Quick run command | `cargo test --lib audit` |
| Full suite command | `cargo test` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| AUDIT-01 | Audit entries have required fields (timestamp, action, category, data, actor) | unit | `cargo test --lib audit::loader::tests` | No - Wave 0 |
| AUDIT-02 | Ed25519 signature verifies against canonical format | unit | `cargo test --lib audit::loader::tests::test_signature_verification` | No - Wave 0 |
| AUDIT-03 | previous_hash links to prior entry forming chain | unit | `cargo test --lib audit::loader::tests::test_hash_chain` | No - Wave 0 |
| AUDIT-04 | GET /audit returns filtered entries | integration | `cargo test --test integration_audit` | No - Wave 0 |
| AUDIT-05 | get_audit_log MCP tool returns filtered entries | unit | `cargo test --lib mcp::handler::tests::test_get_audit_log` | No - Wave 0 |
| AUDIT-06 | 40 retroactive entries exist in audit_log.json | integration | `cargo test --test integration_audit::test_retroactive_entries` | No - Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --lib audit`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before /gsd:verify-work

### Wave 0 Gaps
- [ ] `src/audit/` module -- does not exist yet
- [ ] `tests/fixtures/audit_log.json` -- test fixture for valid audit log
- [ ] `tests/fixtures/audit_log_invalid.json` -- test fixture for error cases
- [ ] `tests/integration_audit.rs` -- integration tests for GET /audit
- [ ] uuid and chrono dependencies in Cargo.toml
- [ ] ed25519-dalek and sha2 as direct dependencies in Cargo.toml

## Sources

### Primary (HIGH confidence)
- [pkarr 5.0.2 docs](https://docs.rs/pkarr/5.0.2/pkarr/struct.Keypair.html) - Keypair struct methods, secret_key() returns ed25519-dalek SecretKey
- [ed25519-dalek 3.0.0-pre.6 docs](https://docs.rs/ed25519-dalek/3.0.0-pre.6/ed25519_dalek/struct.SigningKey.html) - SigningKey API, from_bytes, try_sign, verify
- [sha2 docs](https://docs.rs/sha2/latest/sha2/) - Sha256 hasher with Digest trait
- [axum Query extractor docs](https://docs.rs/axum/latest/axum/extract/struct.Query.html) - Query parameter deserialization pattern
- Cargo.lock in project - exact versions of transitive dependencies confirmed

### Secondary (MEDIUM confidence)
- [uuid crate](https://crates.io/crates/uuid) - v4 and serde features confirmed
- [chrono crate](https://docs.rs/chrono) - DateTime<Utc> serde integration, to_rfc3339_opts for Z suffix control

### Tertiary (LOW confidence)
- ed25519-dalek 3.0.0-pre.6 API stability - pre-release version, API could change but is pinned via Cargo.lock

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - dependencies already in tree, versions confirmed from Cargo.lock
- Architecture: HIGH - mirrors existing registry module pattern exactly
- Pitfalls: HIGH - identified from dependency analysis and format specification review
- Crypto interop: MEDIUM - pkarr/ed25519-dalek interop confirmed via docs but needs unit test validation

**Research date:** 2026-03-07
**Valid until:** 2026-04-07 (stable project, dependencies pinned)
