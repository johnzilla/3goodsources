# Domain Pitfalls: v2.0 Community Curation Features

**Domain:** Adding audit log, identity linking, and community contributions to existing Rust MCP server
**Researched:** 2026-03-07
**Confidence:** MEDIUM-HIGH (based on codebase analysis, cryptographic signing patterns, JSON storage pitfalls, and community system design)

## Critical Pitfalls

Mistakes that cause data corruption, security breaches, or require rewrites.

### Pitfall 1: `deny_unknown_fields` Breaking Schema Evolution

**What goes wrong:** The existing codebase uses `#[serde(deny_unknown_fields)]` on every struct in `registry/types.rs`. Any new JSON data files (audit_log.json, identities.json, contributions.json) using the same pattern will be unable to evolve their schemas without breaking deserialization of existing data.

**Why it happens:**
- Developer copies the pattern from existing types without considering forward compatibility
- New fields added to audit log entries cause older entries (without those fields) to fail validation -- or worse, newer files with additional fields fail on older server versions
- `deny_unknown_fields` is correct for the existing registry (strict contract), but wrong for append-only logs that accumulate entries over time

**Consequences:**
- Adding a field to an audit log entry type means rewriting all historical entries or the server refuses to start
- Cannot deploy a new server version until all JSON files are manually migrated
- Append-only property violated (you must modify old entries to add new fields)

**Prevention:**
```rust
// WRONG for evolving data: strict rejection
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditEntry {
    pub timestamp: String,
    pub action: String,
}

// RIGHT for append-only data: use defaults for new optional fields
#[derive(Deserialize)]
pub struct AuditEntry {
    pub timestamp: String,
    pub action: String,
    #[serde(default)]  // Old entries without this field still load
    pub metadata: Option<serde_json::Value>,
}

// Keep deny_unknown_fields ONLY on registry types (stable schema)
```

**Detection:** Add integration tests that load audit log fixtures from "v1" and "v2" schema versions to verify backward compatibility.

**Phase mapping:** Address in the first phase that defines new data types. Decision must be made before any JSON files are written to disk.

---

### Pitfall 2: Audit Log Signatures Not Actually Verifiable

**What goes wrong:** The server claims audit entries are "signed with PKARR" but the signature doesn't cover a well-defined, canonicalized payload, making verification impossible or trivially forgeable.

**Why it happens:**
- Signing the pretty-printed JSON string (whitespace changes = different signature)
- Not defining a canonical serialization order (serde_json HashMap iteration order is non-deterministic)
- Signing only part of the entry (timestamp but not content, or content but not timestamp)
- Using the existing `pkarr` crate which currently only has the `keys` feature enabled -- it provides `Keypair` and `PublicKey` but signing arbitrary messages requires `ed25519-dalek` directly or enabling additional features
- The current codebase claims "cryptographically signed to prevent tampering" in MCP tool responses but no actual signing is implemented

**Consequences:**
- Signatures verify on the server that created them but fail on any other implementation
- Third parties cannot independently verify the audit log
- The "signed" claim becomes security theater, undermining the entire trust model
- If JSON field ordering changes between serde versions, all existing signatures break

**Prevention:**
```rust
// Define a canonical format for signing
// Option A: Sort keys, compact JSON (no whitespace)
fn canonicalize(entry: &AuditEntry) -> Vec<u8> {
    // Use serde_json with sorted keys
    let value = serde_json::to_value(entry).unwrap();
    let sorted = sort_json_keys(&value);
    serde_json::to_vec(&sorted).unwrap()  // Compact, no pretty print
}

// Option B: Sign a specific concatenation of fields (simpler, more robust)
fn sign_payload(entry: &AuditEntry) -> Vec<u8> {
    format!("{}|{}|{}|{}", entry.version, entry.timestamp, entry.action, entry.target)
        .into_bytes()
}

// Use ed25519-dalek directly for signing (pkarr wraps it)
use ed25519_dalek::{Signer, SigningKey};
let signature = signing_key.sign(&canonicalize(&entry));
```

**Detection:** Write a verification test that serializes, signs, deserializes, re-serializes, and verifies. If this round-trip test fails, canonicalization is broken.

**Phase mapping:** Must be addressed in the audit log phase. Define the signing format specification BEFORE writing any entries. Document the canonical format so third parties can verify.

---

### Pitfall 3: Hash Chain Gaps Destroying Audit Log Integrity

**What goes wrong:** The append-only audit log uses a hash chain (each entry references the hash of the previous entry) but gaps or out-of-order entries silently corrupt the chain.

**Why it happens:**
- Curator edits the JSON file manually and accidentally deletes an entry or reorders them
- JSON array ordering not preserved during manual editing (copy-paste errors)
- No validation that the hash chain is contiguous on startup
- Server starts with a broken chain and serves corrupted audit data without warning

**Consequences:**
- Anyone verifying the hash chain detects tampering (but it was actually an accident)
- Loss of trust in the audit log (even if the data is actually correct)
- No way to repair the chain without re-signing everything from the break point forward
- If not caught early, the break propagates and requires full chain reconstruction

**Prevention:**
```rust
fn validate_hash_chain(entries: &[AuditEntry]) -> Result<(), AuditError> {
    for i in 1..entries.len() {
        let expected_prev_hash = hash_entry(&entries[i - 1]);
        if entries[i].prev_hash != expected_prev_hash {
            return Err(AuditError::BrokenChain {
                entry_index: i,
                expected: expected_prev_hash,
                actual: entries[i].prev_hash.clone(),
            });
        }
    }
    Ok(())
}

// Validate on EVERY startup, fail loud
let audit_log = load_audit_log(&path).await?;
validate_hash_chain(&audit_log.entries)?;
```

**Detection:** Startup validation that checks the full chain and refuses to start if broken, with a clear error message identifying the exact break point.

**Phase mapping:** Implement hash chain validation in the same phase as the audit log. Never ship the audit log endpoint without chain validation.

---

### Pitfall 4: Identity Linking Without Proof Creates Impersonation Vectors

**What goes wrong:** The identities.json file maps PKARR pubkeys to X/Nostr/GitHub handles, but without any verification proof stored alongside the claim, anyone who edits the file can claim any identity.

**Why it happens:**
- v2.0 explicitly defers automated verification (OAuth, signature challenges) to later
- Manual verification by curator is the intended workflow, but the JSON file doesn't record HOW the identity was verified
- No proof artifact stored (e.g., a signed message from the linked account, a specific tweet URL, a Nostr event ID)
- Anyone with file access can add arbitrary identity claims

**Consequences:**
- A compromised or careless file edit links a PKARR key to the wrong X/GitHub account
- No way to audit whether identities were actually verified or just added
- If the curator changes, the new curator has no way to re-verify past claims
- Trust model collapses: "The file says so" is not verification

**Prevention:**
```rust
#[derive(Serialize, Deserialize)]
pub struct IdentityLink {
    pub pkarr_pubkey: String,
    pub platform: Platform,
    pub handle: String,
    pub verified_at: String,  // ISO 8601
    pub verification_method: VerificationMethod,
    pub proof_url: Option<String>,  // Tweet URL, Gist URL, Nostr event ID
    pub verified_by: String,  // Curator pubkey who verified
}

#[derive(Serialize, Deserialize)]
pub enum VerificationMethod {
    TweetVerification,   // User posted their pubkey in a tweet
    GistVerification,    // User created a gist with their pubkey
    NostrEventProof,     // User published a Nostr event with their pubkey
    ManualVerification,  // Curator verified out-of-band (document how)
}
```

**Detection:** Every identity link must have a non-empty `verification_method` and ideally a `proof_url`. Validation on startup should warn about entries with `ManualVerification` and no `proof_url`.

**Phase mapping:** Address in the identity linking phase. Design the schema with proof fields from day one, even if automated verification comes later.

---

## Moderate Pitfalls

Mistakes that cause technical debt, performance issues, or degraded trust.

### Pitfall 5: AppState Bloat from Adding Multiple Data Stores

**What goes wrong:** The existing `AppState` holds `McpHandler`, `Registry`, and `PublicKey`. Adding audit log, identities, and contributions means stuffing more `Arc<T>` fields into AppState, making the struct unwieldy and creating coupling between unrelated data.

**Why it happens:**
- The current pattern is straightforward for one data store (registry)
- Adding 3 more data stores by copy-pasting the pattern creates a god struct
- All handlers get access to all data whether they need it or not
- Testing requires constructing the full AppState even for single-endpoint tests

**Consequences:**
- Every new endpoint handler depends on the full application state
- Compile times increase as changes to any data type recompile all handlers
- Integration tests become brittle (must construct full state for any test)
- No clear ownership boundaries

**Prevention:**
```rust
// WRONG: God struct
pub struct AppState {
    pub mcp_handler: McpHandler,
    pub registry: Arc<Registry>,
    pub audit_log: Arc<AuditLog>,
    pub identities: Arc<Identities>,
    pub contributions: Arc<Contributions>,
    pub pubkey: PublicKey,
}

// RIGHT: Compose from focused state objects
pub struct AppState {
    pub mcp_handler: McpHandler,
    pub registry: Arc<Registry>,
    pub community: Arc<CommunityState>,  // Groups new v2.0 data
    pub pubkey: PublicKey,
}

pub struct CommunityState {
    pub audit_log: AuditLog,
    pub identities: Identities,
    pub contributions: Contributions,
}
```

**Detection:** If adding a new JSON file requires modifying `AppState`, `main.rs`, `McpHandler::new()`, and test helpers simultaneously, the coupling is too tight.

**Phase mapping:** Refactor AppState in the first v2.0 phase, before adding any new data stores.

---

### Pitfall 6: Human vs Bot Vote Separation Becoming an Arms Race

**What goes wrong:** The system tries to classify votes as "human" or "bot" but the classification logic becomes increasingly complex and never fully reliable.

**Why it happens:**
- Simple heuristics (user-agent, rate limiting) are trivially bypassed
- Sophisticated bots mimic human voting patterns (timing jitter, varied user agents)
- No ground truth for training detection algorithms
- The harder you make detection, the more you penalize legitimate users

**Consequences:**
- Bot votes leak into human vote counts, corrupting signal
- False positives flag real users as bots, damaging trust
- Ongoing maintenance burden as bot techniques evolve
- Classification code becomes the most complex part of the system

**Prevention:** Do NOT try to detect bots automatically. Instead, design the system so bot votes are useful but distinct:

```rust
#[derive(Serialize, Deserialize)]
pub struct Vote {
    pub voter_pubkey: String,
    pub voter_type: VoterType,  // Self-declared
    pub proposal_id: String,
    pub vote: VoteValue,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize)]
pub enum VoterType {
    Human,      // Voter with verified identity link
    Agent,      // MCP client/bot (self-identified)
    Unknown,    // No identity verification
}
```

Classification by identity status (verified identity = human signal, no identity = unknown signal, self-declared agent = bot signal) rather than behavioral detection. This is the approach PROJECT.md implies -- votes are separated by type, not detected.

**Detection:** If you find yourself writing regex to parse user-agent strings or implementing rate-based heuristics, you are going down the wrong path.

**Phase mapping:** Address in the contributions/voting phase. Define vote types by identity verification status, not behavioral analysis.

---

### Pitfall 7: Startup Time Regression from Loading Multiple JSON Files

**What goes wrong:** v1.0 loads one file (registry.json) on startup. v2.0 adds audit_log.json, identities.json, and contributions.json. Startup time increases linearly, and a validation failure in any file prevents the server from starting.

**Why it happens:**
- Sequential loading: load registry, then audit log, then identities, then contributions
- Full hash chain validation on the audit log at startup
- Signature verification on all audit entries at startup
- Any single file being malformed crashes the entire server

**Consequences:**
- Startup time grows from ~1 second to 5-10+ seconds as data accumulates
- A typo in contributions.json prevents the registry from being served
- Health check timeouts on DigitalOcean during startup
- Deployments become fragile (any file issue = complete outage)

**Prevention:**
```rust
// Load files in parallel
let (registry, audit_log, identities, contributions) = tokio::try_join!(
    registry::load(&config.registry_path),
    audit::load(&config.audit_log_path),
    identity::load(&config.identities_path),
    contribution::load(&config.contributions_path),
)?;

// Graceful degradation: core registry always loads, new features are optional
let registry = registry::load(&config.registry_path).await?;  // Required
let audit_log = audit::load(&config.audit_log_path).await
    .unwrap_or_else(|e| {
        tracing::warn!("Audit log failed to load: {}, serving without audit", e);
        AuditLog::empty()
    });
// Same pattern for identities and contributions
```

**Detection:** Measure startup time in CI with realistic data sizes. Alert if startup exceeds 5 seconds.

**Phase mapping:** Implement parallel loading and graceful degradation in the first phase that adds a second data file.

---

### Pitfall 8: New MCP Tools Breaking Existing Tool Discovery

**What goes wrong:** Adding `get_identity`, `list_proposals`, and `get_proposal` MCP tools causes existing clients to break because the `tools/list` response changes.

**Why it happens:**
- MCP clients may cache the tool list and not expect it to change
- New tools with required parameters cause `tools/list` response to exceed size limits in some clients
- Tool names or parameter schemas conflict with existing tools
- Capability negotiation doesn't account for new features

**Consequences:**
- Existing MCP clients that work with v1.0 break on v2.0
- Agents that parse `tools/list` responses fail on unexpected new tools
- No way for clients to opt into v2.0 features gradually

**Prevention:**
```rust
// Ensure new tools follow the same naming convention
// Existing: get_sources, list_categories, get_provenance, get_endorsements
// New:      get_identity, list_proposals, get_proposal (consistent pattern)

// Test that existing tool calls still produce identical responses
#[test]
fn v1_tool_responses_unchanged() {
    let handler = create_handler_with_v2_data();
    let response = handler.handle("get_sources", params);
    // Response format must be identical to v1.0
    assert_eq!(response.format, v1_expected_format);
}
```

**Detection:** Run the full v1.0 integration test suite against the v2.0 server without modification. All 78 existing tests must pass unchanged.

**Phase mapping:** Every phase that adds a new MCP tool must run the existing test suite as a regression gate.

---

### Pitfall 9: Audit Log JSON File Unbounded Growth

**What goes wrong:** The audit log is append-only and loaded entirely into memory on startup. Over months/years, this file grows without bound, eventually causing the memory and startup-time problems from Pitfall 3 in the v1.0 research.

**Why it happens:**
- Every registry change, identity verification, and contribution action creates an audit entry
- Append-only means entries are never removed
- JSON arrays with thousands of entries become slow to parse (serde_json `from_str` parses the entire string before returning)
- Memory usage for `serde_json::Value` is roughly 2-4x the file size on disk

**Consequences:**
- After 1,000+ entries, startup adds noticeable delay
- After 10,000+ entries, memory usage becomes a concern on DigitalOcean (512MB-1GB instances)
- JSON file becomes unwieldy to manually edit (curator workflow breaks)
- `serde_json::from_str` on a 20MB file is measurably slow (100ms+)

**Prevention:**
```rust
// Set a size limit and plan for rotation
const MAX_AUDIT_FILE_SIZE: usize = 5 * 1024 * 1024; // 5MB

// For v2.0 scale (manual curation), this is probably fine:
// - 100 audit entries/month * 500 bytes/entry = 50KB/month
// - 5MB limit = ~8 years of data at this rate

// But document the plan for when it matters:
// Future: audit_log_2026.json, audit_log_2027.json (yearly rotation)
// Future: summary endpoint with pagination, not full dump
```

At the current scale (manual curation, ~100 entries/month), this is not an immediate problem. But the design should not preclude rotation later.

**Detection:** Log the audit file size and entry count on startup. Alert if approaching 1MB (years away at current scale).

**Phase mapping:** Implement size monitoring in the audit log phase. Defer rotation to a future milestone but do not design the schema in a way that prevents it.

---

### Pitfall 10: Contribution Proposals Without Lifecycle State Machine

**What goes wrong:** Proposals exist as entries in a JSON file but have no defined state transitions, leading to proposals stuck in limbo, duplicate approvals, or rejected proposals being re-submitted.

**Why it happens:**
- Starting with just "proposed" and "accepted/rejected" states seems simple enough
- No enforcement of valid transitions (can a rejected proposal be re-opened?)
- No tracking of who changed the state and when
- Multiple proposals for the same change with no deduplication

**Consequences:**
- Proposals in undefined states ("approved but not applied", "rejected but re-submitted")
- No audit trail of proposal lifecycle (ironic given the audit log exists)
- Curator confusion about which proposals need attention
- API returns proposals in inconsistent states

**Prevention:**
```rust
#[derive(Serialize, Deserialize)]
pub enum ProposalStatus {
    Submitted,   // Initial state
    UnderReview,  // Curator acknowledged
    Approved,    // Curator approved, pending application
    Applied,     // Changes applied to registry
    Rejected,    // Curator rejected with reason
    Withdrawn,   // Proposer withdrew
}

// Valid transitions (enforce in validation)
// Submitted -> UnderReview | Rejected | Withdrawn
// UnderReview -> Approved | Rejected | Withdrawn
// Approved -> Applied | Withdrawn
// Applied -> (terminal)
// Rejected -> (terminal, but can submit NEW proposal)
// Withdrawn -> (terminal)

fn validate_transition(from: &ProposalStatus, to: &ProposalStatus) -> bool {
    matches!((from, to),
        (ProposalStatus::Submitted, ProposalStatus::UnderReview) |
        (ProposalStatus::Submitted, ProposalStatus::Rejected) |
        (ProposalStatus::Submitted, ProposalStatus::Withdrawn) |
        (ProposalStatus::UnderReview, ProposalStatus::Approved) |
        (ProposalStatus::UnderReview, ProposalStatus::Rejected) |
        (ProposalStatus::UnderReview, ProposalStatus::Withdrawn) |
        (ProposalStatus::Approved, ProposalStatus::Applied) |
        (ProposalStatus::Approved, ProposalStatus::Withdrawn)
    )
}
```

**Detection:** Write tests for every valid transition and every invalid transition. Verify that JSON fixtures with invalid state sequences fail validation.

**Phase mapping:** Address in the contributions phase. Define the state machine before implementing any proposal endpoints.

---

## Minor Pitfalls

Mistakes that cause annoyance or minor issues but are easily fixed.

### Pitfall 11: CORS Not Updated for New Endpoints

**What goes wrong:** New endpoints (`/audit`, `/identities`, `/proposals`) work via curl but fail from browser clients because the CORS allowlist only covers `/mcp`, `/health`, `/registry`, and `/`.

**Why it happens:** The CORS middleware is applied at the router level (which covers all routes), but if new routes are added via nested routers or separate middleware stacks, they may not inherit the CORS layer.

**Prevention:** Verify that `build_router` applies the CORS layer after all routes are defined. The current implementation applies CORS to the full router, which is correct. Maintain this pattern -- do not nest sub-routers with separate middleware.

**Phase mapping:** Verify in every phase that adds new routes.

---

### Pitfall 12: Inconsistent Timestamp Formats Across JSON Files

**What goes wrong:** Registry uses `"2026-02-01"` (date only), audit log uses `"2026-03-07T14:30:00Z"` (ISO 8601 with time), identities use `"March 7, 2026"` (human readable). Filtering by date range across files becomes impossible.

**Prevention:**
```rust
// Standardize on RFC 3339 (ISO 8601 with timezone) for all new files
// Format: "2026-03-07T14:30:00Z"
use chrono::{DateTime, Utc};

pub fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

// The existing registry.updated field uses "2026-02-01" format
// Do NOT change this (backward compatibility), but all new files use RFC 3339
```

**Phase mapping:** Establish the timestamp convention in the first v2.0 phase and document it.

---

### Pitfall 13: Serving Stale Data After JSON File Updates

**What goes wrong:** The curator updates a JSON file on disk (e.g., adds a new audit entry), but the server continues serving the old in-memory data until restarted.

**Why it happens:** All JSON files are loaded once on startup into `Arc<T>` and served from memory. There is no file watcher or reload mechanism.

**Prevention:** This is actually the correct design for v2.0 (curator deploys new files via git push, which triggers a redeploy on DigitalOcean App Platform). But it must be documented clearly:

```
IMPORTANT: After editing any JSON data file, you must redeploy the server.
The server loads all data at startup and does not watch for file changes.

Workflow:
1. Edit registry.json / audit_log.json / identities.json / contributions.json
2. Commit and push to main
3. DigitalOcean auto-deploys from main branch
4. New server instance loads updated files
```

**Detection:** Document this in the curator workflow. Do not add file watching complexity for v2.0.

**Phase mapping:** Document in the first v2.0 phase.

---

### Pitfall 14: Missing Pagination on New List Endpoints

**What goes wrong:** `/audit` returns the entire audit log as a single JSON response. At 1,000+ entries, this becomes a 500KB+ response that is slow to transfer and parse.

**Prevention:**
```rust
// Support optional query parameters from day one
// GET /audit?limit=50&offset=0&action=registry_update
// GET /proposals?status=submitted&limit=20

// Default limit prevents unbounded responses
const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 200;
```

**Phase mapping:** Implement pagination parameters when adding list endpoints. Even if the data is small now, the API contract should support pagination from the start.

---

### Pitfall 15: ed25519 Signature Malleability

**What goes wrong:** Ed25519 signatures can be trivially malleable unless the scalar component `s` is constrained to `0 <= s < L`. An attacker can create valid alternate signatures from existing ones.

**Why it happens:** Different Ed25519 implementations have different validation strictness. The `ed25519-dalek` crate (which pkarr depends on) defaults to strict validation, but if signatures are verified by other implementations (JavaScript clients, Nostr relays), they may accept or reject differently.

**Prevention:** Use `ed25519-dalek`'s strict verification (the default) and document which verification rules the system uses. If third parties will verify signatures, specify: "Signatures conform to ed25519-dalek strict mode (RFC 8032 with cofactored verification disabled, s < L enforced)."

**Phase mapping:** Document verification semantics when implementing the audit log signing. This matters most if external parties verify signatures.

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Data type definitions | `deny_unknown_fields` breaking evolution (Pitfall 1) | Use `#[serde(default)]` for new types, keep `deny_unknown_fields` only on registry types |
| Audit log implementation | Signature canonicalization (Pitfall 2) | Define and document canonical format before writing first entry |
| Audit log implementation | Hash chain integrity (Pitfall 3) | Validate full chain on startup, fail loud on breaks |
| Audit log implementation | Unbounded growth (Pitfall 9) | Monitor size, plan for rotation, set size limits |
| Identity linking | Missing verification proofs (Pitfall 4) | Store proof artifacts (URLs, event IDs) alongside claims |
| Identity linking | Impersonation without proof (Pitfall 4) | Require verification_method field on every link |
| MCP tool additions | Breaking existing clients (Pitfall 8) | Run full v1.0 test suite as regression gate |
| Community contributions | Bot vote arms race (Pitfall 6) | Classify by identity status, not behavioral detection |
| Community contributions | Proposal state machine (Pitfall 10) | Define valid transitions before implementing endpoints |
| All new endpoints | Startup time regression (Pitfall 7) | Parallel loading, graceful degradation for non-core data |
| All new endpoints | Stale data confusion (Pitfall 13) | Document reload-requires-redeploy in curator workflow |
| All new routes | CORS inheritance (Pitfall 11) | Verify CORS applies to all routes after adding new ones |
| All new data files | Timestamp inconsistency (Pitfall 12) | Standardize on RFC 3339 for all new files |
| All list endpoints | Missing pagination (Pitfall 14) | Add limit/offset parameters from the start |
| Server architecture | AppState bloat (Pitfall 5) | Group v2.0 data under CommunityState, not flat fields |

## Domain-Specific Testing Checklist

Before deploying v2.0, verify:

**Backward Compatibility:**
- [ ] All 78 existing tests pass without modification
- [ ] Existing MCP tool responses are byte-identical to v1.0
- [ ] `/health`, `/registry`, `/mcp` endpoints unchanged
- [ ] Registry with `deny_unknown_fields` still loads correctly

**Audit Log Integrity:**
- [ ] Signatures verify after serialize-deserialize round-trip
- [ ] Hash chain validates on startup
- [ ] Broken chain detected and reported with entry index
- [ ] Canonical format documented for third-party verification

**Identity Linking:**
- [ ] Every identity link has a verification_method
- [ ] Proof URLs are valid and documented
- [ ] No identity claims without proof artifacts

**Contribution System:**
- [ ] All state transitions tested (valid and invalid)
- [ ] Terminal states cannot be re-opened
- [ ] Vote types classified by identity status

**Performance:**
- [ ] Startup time with all JSON files < 3 seconds
- [ ] Memory usage with realistic data < 300MB
- [ ] No unbounded JSON responses (all list endpoints paginated)

## Confidence Assessment

**Overall confidence:** MEDIUM-HIGH

| Area | Confidence | Reason |
|------|------------|--------|
| `deny_unknown_fields` pitfall | HIGH | Verified in codebase -- all types use it, serde behavior is well-documented |
| Signature canonicalization | HIGH | Well-known problem in cryptographic signing, documented extensively |
| Hash chain integrity | HIGH | Standard append-only log pattern, well-understood failure modes |
| Identity verification proofs | MEDIUM | Design recommendation based on decentralized identity patterns; specific PKARR identity linking is novel territory |
| Bot vote separation | MEDIUM | Based on community voting system research; v2.0 manual curation context reduces risk |
| AppState architecture | HIGH | Direct codebase analysis of current structure |
| Startup time regression | MEDIUM | Estimated based on serde_json benchmarks; actual impact depends on data sizes |
| MCP backward compatibility | HIGH | Existing test suite provides concrete regression gate |

## Sources

- [serde deny_unknown_fields backward compatibility issues](https://github.com/serde-rs/serde/issues/2634)
- [serde_json memory usage with large files](https://github.com/serde-rs/json/issues/635)
- [serde_json slow parsing of 20MB files](https://github.com/serde-rs/json/issues/160)
- [Ed25519 validation criteria inconsistencies](https://hdevalence.ca/blog/2020-10-04-its-25519am/)
- [Ed25519 signature quirks and verification](https://slowli.github.io/ed25519-quirks/basics/)
- [Ed25519 deep dive on signatures](https://cendyne.dev/posts/2022-03-06-ed25519-signatures.html)
- [Immutable audit log architecture patterns](https://www.emergentmind.com/topics/immutable-audit-log)
- [Immutable audit trail best practices](https://www.hubifi.com/blog/immutable-audit-log-basics)
- [Append-only log enforcement patterns](https://www.designgurus.io/answers/detail/how-do-you-enforce-immutability-and-appendonly-audit-trails)
- [Votebots and automated voting detection challenges](https://fraudblocker.com/articles/bots/votebots-how-to-make-em-and-how-to-stop-em)
- [Decentralized identity verification patterns](https://www.dock.io/post/decentralized-identity)
- [pkarr crate on crates.io](https://crates.io/crates/pkarr) (v5.0.3, keys feature)
- Codebase analysis: `src/registry/types.rs`, `src/server.rs`, `src/main.rs`, `src/pubky/identity.rs`, `src/mcp/tools.rs`, `Cargo.toml`
