# Phase 5: Identity & Provenance - Research

**Researched:** 2026-02-02
**Domain:** Ed25519 keypair generation and PKARR identity in Rust
**Confidence:** HIGH

## Summary

PKARR (Public Key Addressable Resource Records) uses Ed25519 keypairs where the public key serves as a sovereign, censorship-resistant identifier. For 3GS, we need to generate or load an Ed25519 keypair at startup, display the public key in z-base-32 format, and integrate it into the existing AppState architecture.

The standard approach is to use `ed25519-dalek` (version 3.0.0-pre.5) for Ed25519 cryptography, which is the de facto standard in Rust and is already used by the official pkarr crate (v5.0.2). For PKARR-specific functionality like z-base-32 encoding of public keys, we can either use the `pkarr` crate directly (recommended) or implement our own encoding using `z-base-32` crate (v0.1.4).

**Primary recommendation:** Use `pkarr` crate v5.0 for `Keypair` and `PublicKey` types, which handles Ed25519 operations and z-base-32 encoding. This gives us PKARR-compatible key management without pulling in DHT or networking dependencies.

## Standard Stack

The established libraries for Ed25519 and PKARR in Rust:

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| pkarr | 5.0.2 | PKARR keypair types and z-base-32 encoding | Official PKARR implementation, provides `Keypair` and `PublicKey` with proper encoding |
| ed25519-dalek | 3.0.0-pre.5 | Ed25519 cryptography (transitive) | Industry standard, used by pkarr, constant-time operations, no unsafe code |
| hex | 0.4.3 | Hex encoding/decoding for secret keys | Standard encoding library, simple API, no_std compatible |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| z-base-32 | 0.1.4 | Human-oriented base-32 encoding | If NOT using pkarr crate (not recommended) |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| pkarr crate | ed25519-dalek + z-base-32 | More control but need to implement PKARR conventions manually; lose compatibility |
| ed25519-dalek | ed25519-compact | Smaller but less ecosystem adoption; ed25519-dalek is industry standard |

**Installation:**

```toml
[dependencies]
pkarr = { version = "5.0", default-features = false, features = [] }
hex = "0.4"
```

**Note:** pkarr v5.0 has many optional features for DHT, relays, HTTP clients, etc. We disable all default features and add only what we need. The core `Keypair` and `PublicKey` types are always available.

## Architecture Patterns

### Recommended Integration into Existing Code

Current structure:
```
src/
├── config.rs           # Config uses envy for env vars
├── server.rs           # AppState holds Arc<Registry> + McpHandler
├── mcp/                # McpHandler uses Arc<Registry>
└── pubky/              # Currently stub (error.rs + mod.rs)
```

After Phase 5:
```
src/
├── config.rs           # Add optional PKARR_SECRET_KEY: Option<String>
├── server.rs           # AppState add pubkey: PublicKey field
├── pubky/
│   ├── mod.rs          # Re-export Keypair, PublicKey from pkarr
│   ├── error.rs        # Update PubkyError variants
│   └── identity.rs     # NEW: generate_or_load_keypair() function
```

### Pattern 1: Keypair Loading at Startup

**What:** Load secret key from env var or generate new keypair at startup, store PublicKey in AppState.

**When to use:** Server initialization, before building axum router.

**Example:**

```rust
// Source: https://docs.rs/pkarr/latest/pkarr/struct.Keypair.html
use pkarr::{Keypair, PublicKey};

// In src/pubky/identity.rs
pub fn generate_or_load_keypair(secret_key_hex: Option<&str>) -> Result<Keypair, PubkyError> {
    match secret_key_hex {
        Some(hex_str) => {
            // Load from hex-encoded secret key
            let bytes = hex::decode(hex_str)
                .map_err(|_| PubkyError::InvalidSecretKey("invalid hex encoding"))?;

            if bytes.len() != 32 {
                return Err(PubkyError::InvalidSecretKey("secret key must be 32 bytes"));
            }

            let mut key_bytes = [0u8; 32];
            key_bytes.copy_from_slice(&bytes);

            Ok(Keypair::from_secret_key(&key_bytes))
        }
        None => {
            // Generate new random keypair
            tracing::warn!("PKARR_SECRET_KEY not set, generating new keypair. This identity will not persist across restarts.");
            Ok(Keypair::random())
        }
    }
}
```

### Pattern 2: Public Key Display

**What:** Convert public key to z-base-32 string for display in health endpoint and get_provenance tool.

**When to use:** Any time public key needs to be shown to users or AI agents.

**Example:**

```rust
// Source: https://docs.rs/pkarr/latest/pkarr/struct.PublicKey.html
use pkarr::PublicKey;

// Get z-base-32 encoded public key string
let pubkey_str = public_key.to_z32();
// Example output: "o4dksfbqk85ogzdb5osziw6befigbuxmuxkuxq8434q89uj56uyy"

// Or with pk: URI scheme
let pubkey_uri = public_key.to_uri_string();
// Example output: "pk:o4dksfbqk85ogzdb5osziw6befigbuxmuxkuxq8434q89uj56uyy"
```

### Pattern 3: Thread-Safe Storage in AppState

**What:** Store PublicKey in Arc-wrapped AppState for access across async handlers.

**When to use:** AppState initialization, handler access.

**Example:**

```rust
// Source: https://doc.rust-lang.org/std/sync/struct.Arc.html
use std::sync::Arc;
use pkarr::PublicKey;

pub struct AppState {
    pub mcp_handler: McpHandler,
    pub registry: Arc<Registry>,
    pub pubkey: PublicKey,  // PublicKey is Copy, no Arc needed
}

// In health_endpoint
async fn health_endpoint(State(state): State<Arc<AppState>>) -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "pubkey": state.pubkey.to_z32()
    }))
}
```

**Thread safety note:** `PublicKey` from pkarr implements `Copy`, so no Arc/Mutex needed. Just store it directly in AppState.

### Pattern 4: Secret Key Environment Variable Format

**What:** Store secret key as 64-character hex string in PKARR_SECRET_KEY env var.

**When to use:** Production deployments, persistent identity across restarts.

**Example:**

```bash
# Generate new keypair and extract secret key (do this once, manually)
# Use pkarr example or write simple utility

# In .env or environment
PKARR_SECRET_KEY=a1b2c3d4e5f6789... (64 hex chars = 32 bytes)
```

```rust
// In config.rs
#[derive(Debug, Deserialize)]
pub struct Config {
    pub registry_path: PathBuf,
    #[serde(default = "default_log_format")]
    pub log_format: String,
    #[serde(default = "default_port")]
    pub port: u16,

    // NEW: Optional secret key for persistent identity
    pub pkarr_secret_key: Option<String>,
}
```

### Anti-Patterns to Avoid

- **Storing Keypair in AppState**: Only store `PublicKey`. Keypair contains secret key; don't expose it across handlers unnecessarily.
- **Using base64 for secret keys**: Use hex encoding for consistency with common Ed25519 tooling and readability.
- **Generating keypair per request**: Generate once at startup, not per request.
- **Panic on missing env var**: Log warning and generate temporary keypair; allow server to start.

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| z-base-32 encoding | Custom base32 with alphabet | pkarr::PublicKey::to_z32() | PKARR-specific format, handles padding correctly |
| Ed25519 key generation | Custom crypto from curves | pkarr::Keypair::random() or ed25519-dalek | Constant-time operations, audited, no timing side-channels |
| Hex encoding/decoding | Manual byte conversion | hex crate | Handles error cases, optimized, widely tested |
| Secret key validation | Manual length check | Try-from pattern on fixed arrays | Type safety, compiler-enforced |

**Key insight:** Ed25519 cryptography has subtle security requirements (constant-time operations to prevent timing attacks, proper randomness sources). Always use audited libraries, never implement crypto primitives yourself.

## Common Pitfalls

### Pitfall 1: Secret Key Length Mismatch

**What goes wrong:** Loading secret key from env var with wrong byte length (not 32 bytes) causes runtime panic or invalid keys.

**Why it happens:** Hex encoding is 64 chars for 32 bytes; easy to accidentally include/exclude characters.

**How to avoid:**
- Validate hex string is exactly 64 characters before decoding
- Return descriptive error (don't unwrap)
- Document format clearly in config/docs

**Warning signs:**
- "invalid key length" errors
- Hex decode succeeds but keypair creation fails

### Pitfall 2: Public Key Encoding Confusion

**What goes wrong:** Using wrong base32 variant (RFC4648 instead of z-base-32), or using hex/base64 for public key display.

**Why it happens:** Multiple base32 standards exist; z-base-32 is human-oriented with specific alphabet.

**How to avoid:**
- Always use pkarr::PublicKey::to_z32() for PKARR keys
- Don't manually encode public key bytes
- Document that public keys are 52 characters z-base-32

**Warning signs:**
- Public key doesn't match expected format
- Keys contain characters not in z-base-32 alphabet (ybndrfg8ejkmcpqxot1uwisza345h769)

### Pitfall 3: Keypair Not Zeroized on Drop

**What goes wrong:** Secret keys remain in memory after drop, potential security issue.

**Why it happens:** Rust doesn't zero memory by default for performance.

**How to avoid:**
- Use ed25519-dalek or pkarr with default features (includes zeroize)
- Don't disable zeroize feature unless absolutely necessary
- Keep keypair lifetime short, generate once at startup

**Warning signs:**
- Security audit flags secret keys in memory dumps
- Using `default-features = false` without understanding implications

### Pitfall 4: Thread Safety Confusion

**What goes wrong:** Wrapping `PublicKey` in `Arc<Mutex<>>` unnecessarily, or trying to mutate it.

**Why it happens:** Assumption that all types need Arc for multi-threaded access.

**How to avoid:**
- Check if type implements `Copy` (PublicKey does)
- Copy types can be stored directly in Arc<AppState>
- Only use Mutex for mutable shared state

**Warning signs:**
- `Arc<PublicKey>` or `Arc<Mutex<PublicKey>>` in AppState
- Attempting to call .lock() on public key

### Pitfall 5: Generating Keypair Without CSPRNG

**What goes wrong:** Using predictable randomness source, generating weak keys.

**Why it happens:** Using non-cryptographic RNG (like `rand::thread_rng()` without proper features).

**How to avoid:**
- Use pkarr::Keypair::random() which uses proper CSPRNG
- Or use ed25519-dalek::SigningKey::generate(&mut OsRng)
- Never use non-crypto RNG for key generation

**Warning signs:**
- Keys are predictable or repeatable
- Not using OsRng or platform CSPRNG

## Code Examples

Verified patterns from official sources:

### Complete Integration Example

```rust
// Source: https://docs.rs/pkarr/5.0.2/pkarr/
// In src/pubky/identity.rs

use pkarr::Keypair;
use hex;

#[derive(Debug, thiserror::Error)]
pub enum IdentityError {
    #[error("Invalid secret key: {0}")]
    InvalidSecretKey(&'static str),

    #[error("Hex decode error: {0}")]
    HexDecode(#[from] hex::FromHexError),
}

/// Generate new keypair or load from hex-encoded secret key
pub fn generate_or_load_keypair(
    secret_key_hex: Option<&str>
) -> Result<Keypair, IdentityError> {
    match secret_key_hex {
        Some(hex_str) => {
            // Validate length before decode
            if hex_str.len() != 64 {
                return Err(IdentityError::InvalidSecretKey(
                    "hex string must be 64 characters (32 bytes)"
                ));
            }

            // Decode hex to bytes
            let bytes = hex::decode(hex_str)?;

            // Convert to fixed-size array
            let mut key_bytes = [0u8; 32];
            key_bytes.copy_from_slice(&bytes);

            // Create keypair from secret key
            Ok(Keypair::from_secret_key(&key_bytes))
        }
        None => {
            // Warn about ephemeral identity
            tracing::warn!(
                "PKARR_SECRET_KEY not set, generating ephemeral keypair. \
                 Identity will change on restart."
            );

            // Generate using OS CSPRNG
            Ok(Keypair::random())
        }
    }
}
```

### AppState Integration Example

```rust
// Source: Inferred from https://docs.rs/axum/0.8/axum/ + pkarr docs
// In src/server.rs

use pkarr::PublicKey;
use std::sync::Arc;

pub struct AppState {
    pub mcp_handler: McpHandler,
    pub registry: Arc<Registry>,
    pub pubkey: PublicKey,  // Copy type, no Arc needed
}

// In main.rs initialization
let keypair = crate::pubky::identity::generate_or_load_keypair(
    config.pkarr_secret_key.as_deref()
)?;

let public_key = keypair.public_key();

// Log public key at startup
tracing::info!(
    pubkey = %public_key.to_z32(),
    "Server identity initialized"
);

let state = Arc::new(AppState {
    mcp_handler,
    registry,
    pubkey: public_key,
});
```

### Health Endpoint with Public Key

```rust
// Source: https://docs.rs/pkarr/latest/pkarr/struct.PublicKey.html
// In src/server.rs

use axum::{extract::State, Json};
use serde_json::json;
use std::sync::Arc;

async fn health_endpoint(
    State(state): State<Arc<AppState>>
) -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "pubkey": state.pubkey.to_z32()
    }))
}
```

### get_provenance Tool Response

```rust
// Source: Phase 5 requirements
// In src/mcp/tools/get_provenance.rs (or wherever tool is implemented)

use serde_json::json;

// In tool execution
let response = json!({
    "curator": {
        "name": registry.curator.name,
        "bio": registry.curator.bio,
        "pubkey": state.pubkey.to_z32()  // z-base-32 encoded public key
    },
    "version": registry.version,
    "last_updated": registry.last_updated
});
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| ed25519-dalek v1.x with Keypair | ed25519-dalek v2/v3 with SigningKey | v2.0 (2023) | API changed from Keypair to SigningKey/VerifyingKey |
| Manual base32 encoding | pkarr crate with built-in z-base-32 | pkarr v5.0 (2026) | PKARR-specific conventions handled automatically |
| RFC4648 base32 | z-base-32 for PKARR | PKARR spec | Human-readable alphabet, URL-safe |

**Deprecated/outdated:**
- ed25519-dalek v1.x Keypair API: Use v2+ SigningKey (or pkarr::Keypair wrapper)
- Standalone z-base-32 encoding: Use pkarr::PublicKey::to_z32() for PKARR keys
- PEM/DER formats for storage: Use hex encoding for simplicity and ed25519 compatibility

## Open Questions

1. **Should we use pkarr crate or just ed25519-dalek + z-base-32?**
   - What we know: pkarr crate provides exactly what we need (Keypair, PublicKey, z-base-32)
   - What's unclear: Dependency weight - does pkarr pull in DHT/networking even with default-features = false?
   - Recommendation: Use pkarr with minimal features. It's the canonical implementation and ensures PKARR compatibility. If dependency size becomes issue, can refactor later.

2. **Should we store keypair in AppState or just PublicKey?**
   - What we know: Only PublicKey is needed for display; Keypair contains secret key
   - What's unclear: Future signing requirements (Phase 6+)
   - Recommendation: Store only PublicKey in AppState initially. If future phases need signing, add keypair to separate signing service with limited access.

3. **What format for PKARR_SECRET_KEY: hex, base64, or something else?**
   - What we know: Hex is common for ed25519 tools, base64 is more compact
   - What's unclear: pkarr ecosystem convention
   - Recommendation: Use hex (64 chars). It's human-readable, widely supported, and consistent with pkarr::Keypair::from_secret_key API which expects bytes.

4. **Should we validate public key derivation matches stored value?**
   - What we know: Public key is deterministically derived from secret key
   - What's unclear: Whether to separately store/validate public key
   - Recommendation: No. Always derive from secret key. Don't store public key separately in env; it's redundant and creates risk of mismatch.

## Sources

### Primary (HIGH confidence)

- [pkarr crate v5.0.2](https://docs.rs/pkarr/latest/pkarr/) - Keypair and PublicKey API documentation
- [pkarr::PublicKey docs](https://docs.rs/pkarr/latest/pkarr/struct.PublicKey.html) - to_z32(), to_uri_string() methods verified
- [pkarr::Keypair docs](https://docs.rs/pkarr/latest/pkarr/struct.Keypair.html) - random(), from_secret_key(), public_key() methods verified
- [ed25519-dalek v3.0.0-pre.5](https://docs.rs/ed25519-dalek/latest/ed25519_dalek/) - Current version and API patterns
- [ed25519-dalek crates.io](https://crates.io/crates/ed25519-dalek) - Version history and features
- [PKARR GitHub repository](https://github.com/pubky/pkarr) - Official specification and examples
- [hex crate v0.4](https://crates.io/crates/hex) - Hex encoding/decoding API
- [z-base-32 crate v0.1.4](https://lib.rs/crates/z-base-32) - Alternative encoding library
- [Rust std::sync::Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html) - Thread-safe reference counting

### Secondary (MEDIUM confidence)

- [PKARR documentation site](https://pubky.github.io/pkarr/) - Public key encoding overview
- [pkarr publish.rs example](https://github.com/pubky/pkarr/blob/main/pkarr/examples/publish.rs) - Keypair::random() usage verified
- [Rust Cryptography Ecosystem 2026](https://kerkour.com/rust-cryptography-ecosystem-2026) - Current state of crypto in Rust
- [Ed25519 Signatures in Rust with Dalek](https://asecuritysite.com/rust/rust_ed25519_dalek) - Hex encoding examples
- [Awesome Rust Cryptography](https://cryptography.rs/) - Library landscape

### Tertiary (LOW confidence)

- WebSearch results on z-base-32 alphabet specification - Confirmed alphabet: ybndrfg8ejkmcpqxot1uwisza345h769
- WebSearch results on PKARR public key format - Confirmed 52-character z-base-32 for 32-byte keys
- Community discussions on ed25519-dalek best practices - verify_strict() recommendation

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - pkarr and ed25519-dalek are well-documented and verified through official sources
- Architecture: HIGH - Patterns verified through official docs and similar to existing AppState pattern
- Pitfalls: MEDIUM - Based on general cryptography best practices and common Rust patterns
- z-base-32 encoding: HIGH - Verified through pkarr crate documentation and examples

**Research date:** 2026-02-02
**Valid until:** ~2026-04-02 (60 days - stable domain, but pkarr is under active development)

**Key assumptions:**
- pkarr v5.0 is stable enough for production use (released Jan 2026)
- ed25519-dalek v3.0 pre-release is acceptable (or can use v2.2 stable)
- z-base-32 encoding via pkarr is PKARR-canonical (no separate spec needed)
- Thread-safe public key storage requires only Copy, not Arc/Mutex
