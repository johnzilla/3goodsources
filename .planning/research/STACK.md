# Technology Stack

**Project:** Three Good Sources (3GS) MCP Server
**Researched:** 2026-02-01
**Overall Confidence:** MEDIUM (limited by tool access - verification needed)

## Executive Summary

Building a Rust MCP server with Pubky integration requires careful stack selection. Based on available knowledge (training data from January 2025), this document provides recommendations with honest confidence levels. **Critical finding**: Pubky SDK maturity is unknown without crates.io verification - this fundamentally affects architecture decisions and should be validated immediately.

## Recommended Stack

### Core Framework

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| axum | 0.7.x (0.8 unverified) | HTTP server framework | Most actively maintained Rust web framework, excellent tower ecosystem integration, strong JSON support | MEDIUM |
| tokio | 1.x (latest) | Async runtime | Industry standard, required by axum, mature ecosystem | HIGH |
| tower | 0.4.x | Middleware layer | Powers axum, provides composable service abstractions | HIGH |
| tower-http | 0.5.x | HTTP middleware | CORS, tracing, compression middleware | HIGH |

**Axum 0.8 Status:** UNVERIFIED - As of January 2025 training cutoff, axum 0.7.x was current. Version 0.8 may exist now (February 2026) but could not be verified. Recommendation: Use latest stable version available on crates.io.

### MCP Protocol Implementation

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| **Custom JSON-RPC** | N/A | MCP protocol | No mature Rust MCP server library found in training data | MEDIUM |
| serde | 1.x | JSON serialization | Industry standard, zero-copy deserialization | HIGH |
| serde_json | 1.x | JSON parsing | De facto standard for JSON in Rust | HIGH |

**Critical Assessment - MCP in Rust:**

As of January 2025 training cutoff, no mature Rust MCP server libraries were identified. The MCP protocol is JSON-RPC based, which suggests:

**RECOMMENDED APPROACH:** Implement MCP JSON-RPC manually using:
- `serde` + `serde_json` for message parsing
- Custom handler for JSON-RPC 2.0 protocol
- Axum route handler for POST /mcp endpoint

**Rationale:**
1. MCP protocol is relatively simple (JSON-RPC 2.0)
2. Full control over protocol implementation
3. No dependency on potentially immature MCP libraries
4. Direct integration with axum routing

**Alternative (if available):** Check crates.io for `mcp-rs`, `mcp-server`, or similar. If mature library exists (>1000 downloads, recent updates), consider using it. Otherwise, custom implementation is more reliable.

### Pubky/PKARR Integration

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| **pkarr** | UNKNOWN | PKARR key operations | Core PKARR protocol implementation | LOW |
| **pubky** or **pubky-core** | UNKNOWN | Pubky protocol | High-level Pubky operations | LOW |

**CRITICAL UNKNOWNS - Pubky SDK:**

Cannot verify without crates.io access:
- Does `pubky` or `pubky-core` crate exist?
- Does `pkarr` crate exist?
- What is maturity level (version, downloads, last update)?
- What functionality is available?
- Is there a homeserver SDK?

**RECOMMENDED INVESTIGATION (HIGH PRIORITY):**

1. Search crates.io for: `pubky`, `pubky-core`, `pkarr`, `pubky-homeserver`
2. Check GitHub: https://github.com/pubky for official Rust implementations
3. Assess maturity:
   - Version >= 0.3.0 (shows stability)
   - Recent updates (within 3 months)
   - Downloads > 500 (shows adoption)
   - Documentation quality

**ARCHITECTURE DECISION TREE:**

```
IF pubky-core crate exists AND mature (v0.3+, documented):
  ├─ Use pubky-core for identity + trust graph
  ├─ Store registry on Pubky homeserver
  └─ Fallback to local registry.json on error

ELSE IF pkarr crate exists AND mature:
  ├─ Use pkarr for keypair generation only
  ├─ Implement Pubky protocol manually (if spec available)
  └─ Fallback to local registry.json

ELSE (no Rust SDK):
  ├─ Phase 1: Build with local registry.json only
  ├─ Phase 2: Wait for Rust SDK maturity
  └─ Phase 3: Integrate Pubky when available
```

### PKARR Key Management (Fallback Approach)

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| ed25519-dalek | 2.x | Ed25519 signatures | PKARR uses Ed25519 keys | HIGH |
| bs58 | 0.5.x | Base58 encoding | For PKARR key representation | HIGH |
| blake3 | 1.x | Hashing | Fast, secure hashing | HIGH |

**If no pkarr crate exists**, implement PKARR keypair operations using:
- `ed25519-dalek` for key generation and signing
- `bs58` for base58 encoding of public keys
- Standard PKARR format (52-byte public key)

### String Similarity

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| strsim | 0.11.x | Levenshtein distance | Lightweight, well-maintained, provides normalized Levenshtein | HIGH |

**API:** `strsim::normalized_levenshtein(a, b)` returns f64 in [0.0, 1.0] where 1.0 is identical.

**Alternative:** `rapidfuzz` (faster for large-scale operations, but heavier dependency)

### Data Formats

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| serde | 1.x | Serialization framework | Zero-copy, derive macros, ecosystem standard | HIGH |
| serde_json | 1.x | JSON format | Registry storage, MCP messages | HIGH |
| toml | 0.8.x | Config format | Optional: for config files | HIGH |

### Error Handling

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| anyhow | 1.x | Error propagation | Ergonomic `?` operator, context chains | HIGH |
| thiserror | 1.x | Error types | Derive macros for custom error types | HIGH |

**Pattern:**
- `thiserror` for library error types (precise, typed errors)
- `anyhow` for application error handling (ergonomic, contextual)

### Observability

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| tracing | 0.1.x | Structured logging | Industry standard, async-aware, structured data | HIGH |
| tracing-subscriber | 0.3.x | Log formatting | Configurable output, env filter | HIGH |

### Configuration

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| dotenvy | 0.15.x | .env file loading | Development convenience, 12-factor apps | HIGH |

### HTTP Client (for Pubky homeserver)

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| reqwest | 0.12.x | HTTP client | Most popular, async, JSON support | HIGH |

**Needed for:** Pubky homeserver API calls (if using HTTP-based homeserver)

### Testing

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| tokio-test | (built-in) | Async test utilities | Test async code | HIGH |
| serde_json | 1.x | Test fixtures | JSON test data | HIGH |
| mockito | 1.x | HTTP mocking | Mock Pubky homeserver | MEDIUM |

## Alternatives Considered

| Category | Recommended | Alternative | Why Not | Confidence |
|----------|-------------|-------------|---------|------------|
| Web Framework | axum | actix-web | Axum has cleaner API, better tower integration, more active development | HIGH |
| Web Framework | axum | warp | Warp is less actively maintained, axum is spiritual successor | HIGH |
| Async Runtime | tokio | async-std | Tokio has larger ecosystem, better maintained | HIGH |
| JSON | serde_json | json | serde_json is more mature, better error handling | HIGH |
| String Similarity | strsim | rapidfuzz | strsim is simpler, sufficient for use case | MEDIUM |
| HTTP Client | reqwest | hyper | reqwest is higher-level, more ergonomic | HIGH |

## Version Pinning Strategy

**Cargo.toml recommendations:**

```toml
[dependencies]
# Core framework - allow patch updates
axum = "0.7"  # Verify latest: could be 0.8 in Feb 2026
tokio = { version = "1", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }

# Serialization - stable APIs, allow patch
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Error handling - stable
anyhow = "1"
thiserror = "1"

# Observability - stable
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Utilities - stable
dotenvy = "0.15"
strsim = "0.11"

# HTTP client - allow patch
reqwest = { version = "0.12", features = ["json"] }

# Pubky/PKARR - VERIFY THESE EXIST
# pkarr = "0.x"  # Check crates.io
# pubky-core = "0.x"  # Check crates.io

# Fallback if no Pubky SDK:
ed25519-dalek = "2"
bs58 = "0.5"
blake3 = "1"

[dev-dependencies]
tokio-test = "0.4"
mockito = "1"
```

## Installation Commands

```bash
# Create new project (if not exists)
cargo new three-good-sources --bin
cd three-good-sources

# Add dependencies (AFTER verifying versions on crates.io)
cargo add axum tokio --features tokio/full
cargo add tower tower-http --features tower-http/cors,tower-http/trace
cargo add serde serde_json --features serde/derive
cargo add anyhow thiserror
cargo add tracing tracing-subscriber --features tracing-subscriber/env-filter
cargo add dotenvy strsim
cargo add reqwest --features reqwest/json

# Verify Pubky crates exist first:
# cargo search pubky
# cargo search pkarr

# If Pubky crates exist:
# cargo add pubky-core  # or whatever the crate name is
# cargo add pkarr

# If no Pubky crates (fallback):
cargo add ed25519-dalek bs58 blake3

# Dev dependencies
cargo add --dev tokio-test mockito
```

## Docker Configuration for Render

**Render Free Tier Constraints (as of 2025):**
- Memory: 512 MB RAM
- CPU: Shared CPU (not guaranteed)
- Cold starts: After 15 minutes of inactivity
- Monthly hours: 750 free hours

**Dockerfile Strategy:**

```dockerfile
# Multi-stage build for minimal image size
FROM rust:1.75-slim as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build release binary with optimizations
RUN cargo build --release

# Runtime stage - minimal image
FROM debian:bookworm-slim

# Install SSL certificates (needed for HTTPS requests)
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/three-good-sources .

# Render provides PORT environment variable
ENV PORT=10000
EXPOSE 10000

CMD ["./three-good-sources"]
```

**Optimizations for Render Free Tier:**

1. **Fast cold starts:**
   - Minimal Docker image (debian-slim, not full)
   - Static binary copying (not rebuild)
   - Pre-compiled dependencies

2. **Memory efficiency:**
   - Release build with optimizations
   - Lazy-load registry data
   - Limit in-memory caching

3. **Binary size reduction:**
   ```toml
   [profile.release]
   opt-level = "z"     # Optimize for size
   lto = true          # Link-time optimization
   codegen-units = 1   # Single codegen unit
   strip = true        # Strip symbols
   ```

**Expected Performance:**
- Cold start: 2-5 seconds (Rust binary + data load)
- Memory footprint: <100 MB (leaves headroom)
- Binary size: ~5-15 MB (optimized)

## Deployment Checklist

- [ ] Verify axum version on crates.io (0.7 or 0.8?)
- [ ] Search crates.io for `pubky`, `pubky-core`, `pkarr`
- [ ] Assess Pubky SDK maturity (version, docs, downloads)
- [ ] Choose architecture: Pubky-first or local-first
- [ ] Configure Cargo.toml with verified versions
- [ ] Set up Dockerfile with multi-stage build
- [ ] Configure Render environment variables
- [ ] Test cold start time (<5s target)
- [ ] Verify memory usage (<400 MB target)

## Critical Path Items

**BEFORE STARTING DEVELOPMENT:**

1. **Verify Pubky SDK existence and maturity** (CRITICAL)
   - This determines entire architecture
   - Affects roadmap phase structure
   - Impacts MVP scope

2. **Verify axum 0.8 availability**
   - If 0.8 exists, check for breaking changes from 0.7
   - Read migration guide if available

3. **Verify MCP Rust library existence**
   - Could save significant implementation time
   - Check GitHub: https://github.com/modelcontextprotocol
   - Check crates.io: search "mcp", "model context protocol"

## Sources

**Limitation Notice:** Research conducted without crates.io or GitHub access. All recommendations based on training data from January 2025. **MUST VERIFY** all crate versions and availability before proceeding.

**Verification URLs (manual check required):**
- Axum: https://crates.io/crates/axum
- Pubky search: https://crates.io/search?q=pubky
- PKARR search: https://crates.io/search?q=pkarr
- MCP search: https://crates.io/search?q=mcp
- strsim: https://crates.io/crates/strsim
- Render docs: https://render.com/docs/free

**Confidence Legend:**
- HIGH: Well-established technology, API unlikely to change
- MEDIUM: Technology exists but version/availability unverified
- LOW: Existence/maturity unknown, requires verification

## Next Steps for Roadmap

**Phase structure implications:**

1. **Phase 0 (Discovery):** Verify Pubky SDK availability - determines architecture
2. **Phase 1 (Foundation):**
   - If Pubky SDK mature: Build with Pubky-first architecture
   - If Pubky SDK immature: Build with local-first, Pubky integration in Phase 3
3. **Phase 2 (Core MCP):** Implement JSON-RPC manually (no mature Rust MCP lib expected)
4. **Phase 3+ (Advanced):** Pubky integration if deferred from Phase 1

**Research flags:**
- Pubky SDK research is CRITICAL and BLOCKING for Phase 1 planning
- MCP protocol implementation research needed for Phase 2
- Render deployment research needed for final phase
