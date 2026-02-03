# Phase 5 Plan 1: Identity Module Foundation Summary

**One-liner:** PKARR identity module with Ed25519 keypair generation/loading via pkarr crate (with curve25519-dalek patch for compilation).

---
phase: 05-identity-and-provenance
plan: 01
subsystem: identity
tags: [pkarr, ed25519, keypair, cryptography, identity, environment-config]
status: complete
completed: 2026-02-02

requires: [04-02]
provides: [generate_or_load_keypair, pkarr_secret_key_config]
affects: [05-02]

tech-stack.added: [pkarr 5.0, hex 0.4]
tech-stack.patterns: [keypair-generation, hex-encoding, optional-env-config]

key-files.created: [src/pubky/identity.rs]
key-files.modified: [Cargo.toml, src/pubky/error.rs, src/pubky/mod.rs, src/config.rs]

decisions: [use-pkarr-with-patch, hex-encoding-for-secret-keys, optional-config-field]

metrics:
  duration: 2 min 44 sec
  tasks: 2
  commits: 2
---

## Objective Completed

Created PKARR identity module with `generate_or_load_keypair()` function that loads Ed25519 keypairs from hex-encoded environment variable or generates random keypair. Added `pkarr_secret_key` optional field to Config struct. Established cryptographic foundation for server identity that Plan 02 will integrate into AppState and health endpoint.

## Tasks Completed

| Task | Description | Commit | Files |
|------|-------------|--------|-------|
| 1 | Add pkarr and hex dependencies and create identity module | 59bd5f5 | Cargo.toml, src/pubky/identity.rs, src/pubky/mod.rs, src/pubky/error.rs |
| 2 | Add PKARR_SECRET_KEY to Config | 61772d3 | src/config.rs |

## Key Deliverables

### 1. Identity Module (src/pubky/identity.rs)

**What it does:** Provides `generate_or_load_keypair()` function that accepts optional hex-encoded secret key string and returns `pkarr::Keypair`. If secret key provided, validates hex format (64 chars) and decodes to 32-byte array. If not provided, generates random keypair using OS CSPRNG and logs warning about ephemeral identity.

**How to use:**
```rust
use crate::pubky::identity::generate_or_load_keypair;

// Load from env var or generate random
let keypair = generate_or_load_keypair(config.pkarr_secret_key.as_deref())?;
let public_key = keypair.public_key();
```

**Key patterns:**
- Hex validation before decode (exactly 64 chars)
- Descriptive error messages via PubkyError variants
- Warning log for ephemeral keypairs
- Uses pkarr::Keypair types directly

### 2. Error Variants (src/pubky/error.rs)

**What changed:** Replaced `NotImplemented` placeholder with real identity error variants: `InvalidSecretKey(&'static str)` for validation errors and `HexDecode(#[from] hex::FromHexError)` for hex decoding errors.

**Error handling pattern:**
```rust
Err(PubkyError::InvalidSecretKey("hex string must be 64 characters (32 bytes)"))
// or
Err(PubkyError::HexDecode(hex_error)) // automatically via ? operator
```

### 3. Config Field (src/config.rs)

**What it does:** Added `pkarr_secret_key: Option<String>` field to Config struct. Field is optional and defaults to None if PKARR_SECRET_KEY env var not set. No validation at config load time - validation happens in generate_or_load_keypair.

**How to use:**
```bash
# In .env or environment
PKARR_SECRET_KEY=a1b2c3d4e5f6789... (64 hex chars)
```

### 4. Dependencies (Cargo.toml)

**What was added:**
- `pkarr = { version = "5.0", default-features = false, features = ["keys"] }` - Minimal pkarr for Keypair/PublicKey types
- `hex = "0.4"` - Hex encoding/decoding for secret keys
- `[patch.crates-io]` section with curve25519-dalek git patch for compilation fix

**Why the patch:** pkarr 5.0.2 depends on pre-release ed25519-dalek v3.0.0-pre.5 which uses curve25519-dalek v5.0.0-pre.5. The pre-release version has a compilation error (missing crypto_common module in digest crate). Patching to use git main branch of curve25519-dalek fixes the issue.

## Decisions Made

### 1. Use pkarr crate with curve25519-dalek patch

**Context:** pkarr 5.0.2 depends on pre-release versions of ed25519-dalek and curve25519-dalek that have compilation issues.

**Decision:** Use pkarr crate as planned but add `[patch.crates-io]` section to override curve25519-dalek with git main branch.

**Rationale:**
- pkarr provides PKARR-specific functionality (z-base-32 encoding, Keypair/PublicKey types)
- Patch is minimal and non-invasive
- Alternative (using ed25519-dalek directly) would require implementing PKARR conventions manually
- Git patch uses official repository main branch (not a fork)

**Impact:** Server compiles successfully with pkarr. Patch will be removed once pkarr updates to stable ed25519-dalek or curve25519-dalek fixes the issue.

### 2. Hex encoding for secret keys

**Context:** Secret keys need to be stored in environment variables as strings.

**Decision:** Use 64-character hex encoding (32 bytes = 64 hex chars) for PKARR_SECRET_KEY.

**Rationale:**
- Hex is human-readable and widely supported by ed25519 tools
- 64 chars is easy to validate (exactly 2 chars per byte)
- Consistent with pkarr::Keypair::from_secret_key API which expects byte array
- Avoids base64 padding ambiguity

**Impact:** Documentation and validation logic enforce 64-character hex strings.

### 3. Optional Config field with no validation

**Context:** Config struct needs pkarr_secret_key field that may or may not be set.

**Decision:** Make field `Option<String>` with no validation at config load time. Validation happens later in generate_or_load_keypair.

**Rationale:**
- Server should start even if PKARR_SECRET_KEY not set (generates ephemeral keypair)
- Validation requires hex decoding which belongs in identity module, not config module
- Separation of concerns: Config loads env vars, identity module validates and uses them
- Consistent with existing optional Config fields

**Impact:** Invalid secret keys are detected at startup (main.rs) not at config load, with clear error messages.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added curve25519-dalek patch to fix compilation**

- **Found during:** Task 1, running cargo check
- **Issue:** pkarr 5.0.2 transitively depends on curve25519-dalek v5.0.0-pre.5 which has compilation error: `unresolved import 'digest::crypto_common'`. The pre-release version expects crypto_common to be re-exported from digest crate but it's not in the version being used.
- **Fix:** Added `[patch.crates-io]` section to Cargo.toml with `curve25519-dalek = { git = "https://github.com/dalek-cryptography/curve25519-dalek", branch = "main" }` to use git main branch which fixes the module visibility issue.
- **Files modified:** Cargo.toml
- **Commit:** 59bd5f5 (part of Task 1)
- **Why blocking:** Project would not compile without this fix. Cannot proceed with identity module development if dependencies don't compile.

## Integration Points

### For Plan 05-02 (Server Integration)

Plan 02 will:
1. Call `generate_or_load_keypair(config.pkarr_secret_key.as_deref())` in main.rs after config load
2. Extract public key from keypair: `keypair.public_key()`
3. Add `pubkey: PublicKey` field to AppState struct
4. Update health endpoint to return `state.pubkey.to_z32()`
5. Update get_provenance tool to include public key in curator object

**What this plan provides:**
- `generate_or_load_keypair()` function ready to call
- `PubkyError` variants for error handling
- `config.pkarr_secret_key` field ready to pass through
- pkarr dependency with Keypair and PublicKey types available

**Expected flow:**
```rust
// In main.rs (Plan 02)
let config = Config::load()?;
let keypair = crate::pubky::identity::generate_or_load_keypair(
    config.pkarr_secret_key.as_deref()
)?;
let pubkey = keypair.public_key();
tracing::info!(pubkey = %pubkey.to_z32(), "Server identity initialized");

let state = Arc::new(AppState {
    mcp_handler,
    registry,
    pubkey, // PublicKey is Copy, no Arc needed
});
```

## Verification Results

All verification checks passed:

1. ✅ `cargo check` compiles cleanly with pkarr and hex dependencies
2. ✅ `cargo test` - all 43 existing tests pass (no regressions)
3. ✅ `grep "generate_or_load_keypair" src/pubky/identity.rs` finds the function
4. ✅ `grep "pkarr_secret_key" src/config.rs` finds the config field
5. ✅ `grep "pkarr" Cargo.toml` confirms dependency

## Next Phase Readiness

**Phase 5 Plan 02 can proceed immediately.**

Blockers: None

Prerequisites met:
- ✅ Identity module created and compiling
- ✅ Config field added
- ✅ Error types defined
- ✅ Dependencies resolved (with patch)

**Note for Plan 02:** The curve25519-dalek patch is a workaround for pre-release dependency issues. When calling `generate_or_load_keypair()`, handle errors gracefully and provide clear startup messages for invalid secret keys.

## Technical Notes

### pkarr Crate Features

Used `default-features = false, features = ["keys"]` to get minimal pkarr functionality. This includes:
- `Keypair::random()` - Generate random keypair using OS CSPRNG
- `Keypair::from_secret_key()` - Load keypair from 32-byte secret key
- `PublicKey::to_z32()` - Convert public key to z-base-32 string (Plan 02 will use this)
- `PublicKey` implements `Copy` - Can be stored directly in AppState without Arc

Does NOT include:
- DHT networking
- Relay support
- HTTP clients
- LMDB cache

### Curve25519-Dalek Patch Details

The patch points to the official dalek-cryptography/curve25519-dalek repository main branch. This is a temporary workaround until either:
1. pkarr updates to use stable ed25519-dalek 2.x, or
2. curve25519-dalek 5.0 releases with the crypto_common fix

The git dependency adds minimal build time overhead (branch is cached by cargo).

### Secret Key Format

Valid PKARR_SECRET_KEY format:
- Exactly 64 hexadecimal characters (0-9, a-f, case insensitive)
- Represents 32 bytes (64 chars / 2 chars per byte)
- Example: `a1b2c3d4e5f67890a1b2c3d4e5f67890a1b2c3d4e5f67890a1b2c3d4e5f67890`

Invalid formats trigger `PubkyError::InvalidSecretKey` or `PubkyError::HexDecode`.

## Commits

- 59bd5f5: feat(05-01): add pkarr identity module with keypair generation
- 61772d3: feat(05-01): add PKARR_SECRET_KEY config field

## Test Results

All 43 existing tests pass:
- matcher::normalize tests (13 tests)
- matcher::scorer tests (9 tests)
- mcp::handler tests (21 tests)
- No new tests added (identity module tested in Plan 02 integration)

No test failures or regressions.
