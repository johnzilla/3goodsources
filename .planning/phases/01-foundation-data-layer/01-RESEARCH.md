# Phase 1: Foundation & Data Layer - Research

**Researched:** 2026-02-01
**Domain:** Rust project structure, registry schema validation, structured logging, startup behavior
**Confidence:** HIGH

## Summary

Phase 1 establishes the foundational Rust project with strict registry schema validation, immutable in-memory data loading, and environment-switchable structured logging. The phase focuses on "fail-fast" validation where malformed data or missing configuration crashes at startup with clear error messages. This research validates that all locked decisions from CONTEXT.md are well-supported by current Rust ecosystem patterns as of 2026.

**Key findings:**
- Rust Edition 2024 is stable as of Rust 1.85.0 (released 2025-02-20)
- `serde(deny_unknown_fields)` is the correct approach for strict schema validation
- `thiserror` for module-specific errors + `anyhow` in main.rs is the 2026 consensus pattern
- `tracing-subscriber` supports environment-switchable formats (pretty/JSON) with custom logic
- `tokio::fs::read_to_string` + `serde_json::from_str` is faster than `from_reader` for async file loading
- Fail-fast startup validation with `.expect()` is idiomatic for required configuration

**Primary recommendation:** Use type-safe environment variable loading with `envy` crate, strict serde validation with custom validators for business rules (exactly 3 sources per category, at least 3 query patterns), and structured logging configured at startup based on `LOG_FORMAT` environment variable.

## Standard Stack

The established libraries/tools for this domain:

### Core Dependencies

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde | 1.x | Serialization framework | Industry standard, zero-copy deserialization, derive macros |
| serde_json | 1.x | JSON parsing | De facto standard for JSON in Rust, well-maintained |
| tokio | 1.x | Async runtime | Required for async file I/O, industry standard |
| thiserror | 1.x | Error types | Derive macros for module-specific error enums |
| anyhow | 1.x | Error propagation | Ergonomic error handling in main.rs |
| tracing | 0.1.x | Structured logging | Industry standard, async-aware, structured data |
| tracing-subscriber | 0.3.x | Log formatting | Configurable output, env filter support |

### Supporting Dependencies

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| dotenvy | 0.15.x | .env file loading | Development convenience, 12-factor apps |
| envy | 0.4.x | Type-safe env vars | Deserialize env vars into structs with validation |
| regex | 1.x | Pattern validation | Kebab-case slug validation |
| serde_valid | 0.15.x | Custom validation | Business rule validation (exactly 3 sources, etc.) |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| serde_valid | validator | serde_valid integrates directly with serde derive, validator requires separate validation step |
| envy | std::env::var | envy provides type-safe deserialization, std::env is more manual |
| thiserror | manual impl Error | thiserror eliminates boilerplate, same performance |
| tracing | log + env_logger | tracing provides structured logging and spans, log is simpler but less powerful |

**Installation:**
```bash
cargo add serde serde_json --features serde/derive
cargo add tokio --features tokio/full
cargo add thiserror anyhow
cargo add tracing tracing-subscriber --features tracing-subscriber/env-filter,tracing-subscriber/json
cargo add dotenvy envy
cargo add regex
cargo add serde_valid --features serde_valid/derive
```

## Architecture Patterns

### Recommended Project Structure

Based on locked decisions from CONTEXT.md:

```
src/
├── main.rs              # Entry point, anyhow only, startup orchestration
├── error.rs             # Root-level error conversion utilities
├── mcp/
│   ├── mod.rs           # Public interface, re-exports
│   └── error.rs         # McpError enum (thiserror)
├── registry/
│   ├── mod.rs           # Registry struct, public interface
│   ├── types.rs         # Category, Source, SourceType enums
│   ├── loader.rs        # Load from file, validation
│   └── error.rs         # RegistryError enum (thiserror)
└── pubky/
    ├── mod.rs           # Public interface (Phase 5)
    └── error.rs         # PubkyError enum (thiserror)
```

**Rationale:**
- Each module has its own error enum (follows thiserror pattern)
- `anyhow` confined to main.rs (application error handling)
- `mod.rs` files are module entry points (Rust convention)
- `types.rs` separates data structures from behavior
- Nested modules under `src/` (not separate crates yet)

### Pattern 1: Strict Schema Validation with Serde

**What:** Use `#[serde(deny_unknown_fields)]` to reject JSON with unexpected fields at deserialization time.

**When to use:** When schema correctness is critical and you want to fail-fast on data format errors.

**Example:**
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Registry {
    pub version: String,
    pub updated: String,
    pub curator: Curator,
    pub endorsements: Vec<String>,  // Required but can be empty
    pub categories: Vec<Category>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Category {
    pub slug: String,
    pub name: String,
    pub description: String,
    pub query_patterns: Vec<String>,
    pub sources: Vec<Source>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub rank: u8,
    pub name: String,
    pub url: String,
    #[serde(rename = "type")]
    pub source_type: SourceType,
    pub why: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    Documentation,
    Tutorial,
    Video,
    Article,
    Tool,
    Repo,
    Forum,
    Book,
    Course,
    Api,
}
```

**Source:** [Serde attributes documentation](https://serde.rs/attributes.html)

**Caveat:** `deny_unknown_fields` doesn't work well with `#[serde(flatten)]` - avoid flattening if using strict validation.

### Pattern 2: Custom Business Rule Validation

**What:** Add validation beyond serde's deserialization to enforce business rules (exactly 3 sources, valid slugs, etc.)

**When to use:** After successful deserialization, before using the data.

**Example:**
```rust
use regex::Regex;
use std::collections::HashSet;

impl Registry {
    /// Validate business rules after deserialization
    pub fn validate(&self) -> Result<(), RegistryError> {
        let mut seen_slugs = HashSet::new();
        let slug_regex = Regex::new(r"^[a-z][a-z0-9]*(-[a-z0-9]+)*$").unwrap();

        for category in &self.categories {
            // Check for duplicate slugs
            if !seen_slugs.insert(&category.slug) {
                return Err(RegistryError::DuplicateSlug(category.slug.clone()));
            }

            // Validate kebab-case slug format
            if !slug_regex.is_match(&category.slug) {
                return Err(RegistryError::InvalidSlug(category.slug.clone()));
            }

            // Exactly 3 sources required
            if category.sources.len() != 3 {
                return Err(RegistryError::InvalidSourceCount {
                    category: category.slug.clone(),
                    expected: 3,
                    actual: category.sources.len(),
                });
            }

            // At least 3 query patterns required
            if category.query_patterns.len() < 3 {
                return Err(RegistryError::InsufficientQueryPatterns {
                    category: category.slug.clone(),
                    minimum: 3,
                    actual: category.query_patterns.len(),
                });
            }

            // Validate ranks are 1, 2, 3
            let ranks: Vec<u8> = category.sources.iter().map(|s| s.rank).collect();
            let mut sorted_ranks = ranks.clone();
            sorted_ranks.sort_unstable();
            if sorted_ranks != vec![1, 2, 3] {
                return Err(RegistryError::InvalidRanks {
                    category: category.slug.clone(),
                    ranks,
                });
            }
        }

        Ok(())
    }
}
```

**Source:** [Rust kebab-case regex pattern](https://regex101.com/library/z7IorM)

### Pattern 3: Async File Loading with Tokio

**What:** Use `tokio::fs::read_to_string` + `serde_json::from_str` for loading JSON files asynchronously.

**When to use:** Always in async contexts - it's faster than `from_reader` and compatible with tokio.

**Example:**
```rust
use tokio::fs;
use serde_json;
use std::path::Path;

pub async fn load_registry<P: AsRef<Path>>(path: P) -> Result<Registry, RegistryError> {
    // Read entire file asynchronously
    let contents = fs::read_to_string(path)
        .await
        .map_err(|e| RegistryError::FileRead(e.to_string()))?;

    // Parse JSON (this is faster than from_reader even for large files)
    let mut registry: Registry = serde_json::from_str(&contents)
        .map_err(|e| RegistryError::JsonParse(e.to_string()))?;

    // Validate business rules
    registry.validate()?;

    Ok(registry)
}
```

**Source:** [serde_json from_reader performance note](https://docs.rs/serde_json/latest/serde_json/fn.from_reader.html) - "Counterintuitively, from_reader is usually slower than reading a file completely into memory and then applying from_str or from_slice on it."

**Why this works:** serde_json deserialization is synchronous, so after async file read, the parse step is CPU-bound and runs on tokio thread pool without blocking.

### Pattern 4: Environment-Switchable Logging Format

**What:** Configure tracing subscriber to output pretty format (dev) or JSON (production) based on environment variable.

**When to use:** At application startup, before any logging occurs.

**Example:**
```rust
use tracing_subscriber::{fmt, EnvFilter};
use std::env;

pub fn init_logging() {
    let log_format = env::var("LOG_FORMAT").unwrap_or_else(|_| "pretty".to_string());

    match log_format.as_str() {
        "json" => {
            // JSON structured logging for production
            fmt()
                .json()
                .with_env_filter(EnvFilter::from_default_env())
                .with_target(false)
                .with_current_span(false)
                .init();
        }
        _ => {
            // Pretty colored output for development (default)
            fmt()
                .pretty()
                .with_env_filter(EnvFilter::from_default_env())
                .init();
        }
    }
}
```

**Source:** [tracing-subscriber format options](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/index.html)

**Environment variables:**
- `LOG_FORMAT=json` → JSON structured logging
- `LOG_FORMAT=pretty` (or unset) → Pretty colored output
- `RUST_LOG=debug` → Set log level (works with both formats)

### Pattern 5: Type-Safe Environment Variable Loading

**What:** Use `envy` crate to deserialize environment variables into a typed struct with validation.

**When to use:** For required configuration that should fail-fast if missing or invalid.

**Example:**
```rust
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_log_format")]
    pub log_format: String,

    pub registry_path: PathBuf,  // Required - will fail if not set
}

fn default_log_format() -> String {
    "pretty".to_string()
}

pub fn load_config() -> Result<Config, anyhow::Error> {
    dotenvy::dotenv().ok();  // Load .env file if present

    envy::from_env::<Config>()
        .map_err(|e| anyhow::anyhow!("Failed to load configuration: {}", e))
}
```

**Usage:**
```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = load_config()?;
    init_logging(&config.log_format);

    tracing::info!("Loading registry from: {:?}", config.registry_path);
    let registry = load_registry(&config.registry_path).await?;

    tracing::info!(
        "Registry loaded: {} categories, {} total sources, version {}",
        registry.categories.len(),
        registry.categories.iter().map(|c| c.sources.len()).sum::<usize>(),
        registry.version
    );

    Ok(())
}
```

**Environment variables:**
```bash
REGISTRY_PATH=/path/to/registry.json  # Required
LOG_FORMAT=json                        # Optional, defaults to "pretty"
RUST_LOG=info                          # Optional, defaults to "error"
```

**Source:** [envy crate documentation](https://docs.rs/envy/latest/envy/) and [environment variable best practices](https://www.thorsten-hans.com/working-with-environment-variables-in-rust/)

### Pattern 6: Module-Specific Error Types with thiserror

**What:** Each module defines its own error enum using thiserror, main.rs uses anyhow for propagation.

**When to use:** Always - this is the 2026 consensus pattern for Rust error handling.

**Example:**
```rust
// src/registry/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Failed to read registry file: {0}")]
    FileRead(String),

    #[error("Failed to parse registry JSON: {0}")]
    JsonParse(String),

    #[error("Duplicate category slug: {0}")]
    DuplicateSlug(String),

    #[error("Invalid slug format: {0}")]
    InvalidSlug(String),

    #[error("Category {category} must have exactly {expected} sources, found {actual}")]
    InvalidSourceCount {
        category: String,
        expected: usize,
        actual: usize,
    },

    #[error("Category {category} must have at least {minimum} query patterns, found {actual}")]
    InsufficientQueryPatterns {
        category: String,
        minimum: usize,
        actual: usize,
    },

    #[error("Category {category} has invalid ranks {ranks:?}, expected [1, 2, 3]")]
    InvalidRanks {
        category: String,
        ranks: Vec<u8>,
    },
}

// src/main.rs
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = load_config()?;
    init_logging(&config.log_format);

    // anyhow automatically converts RegistryError to context-rich errors
    let registry = load_registry(&config.registry_path)
        .await
        .context("Failed to load registry at startup")?;

    Ok(())
}
```

**Source:** [Rust error handling best practices 2026](https://momori.dev/posts/rust-error-handling-thiserror-anyhow/)

**Key principles from 2026 guidance:**
1. Use thiserror for library/module errors (typed, matchable)
2. Use anyhow for application errors (ergonomic, contextual)
3. Always preserve error chains with `#[source]` or `#[from]`
4. Avoid over-engineering - don't create too many error variants

## Anti-Patterns to Avoid

### Anti-Pattern 1: Using from_reader with Async I/O

**What goes wrong:** Assuming `serde_json::from_reader` is the right choice for async file loading.

**Why it's bad:**
- `from_reader` is slower than reading entire file + `from_str`
- Not compatible with tokio async I/O (requires blocking I/O)
- Official docs explicitly warn against this

**Instead:** Use `tokio::fs::read_to_string` + `serde_json::from_str`

**Source:** [serde_json from_reader documentation](https://docs.rs/serde_json/latest/serde_json/fn.from_reader.html)

### Anti-Pattern 2: Validation After First Use

**What goes wrong:** Deserializing registry, storing it, then validating on first query.

**Why it's bad:**
- Errors occur during request handling (bad UX)
- Hard to debug (error context is lost)
- Violates fail-fast principle

**Instead:** Validate immediately after deserialization, before storing:
```rust
let registry: Registry = serde_json::from_str(&contents)?;
registry.validate()?;  // Fail here if invalid
Arc::new(registry)     // Only store if valid
```

### Anti-Pattern 3: Swallowing Environment Variable Errors

**What goes wrong:** Using `.unwrap_or_default()` for required configuration.

**Why it's bad:**
- Silent failures - app runs with wrong config
- Hard to debug - no error message about missing env var
- Violates fail-fast principle

**Instead:** Use `.expect()` for required config:
```rust
// BAD
let path = env::var("REGISTRY_PATH").unwrap_or_else(|_| "registry.json".to_string());

// GOOD
let path = env::var("REGISTRY_PATH")
    .expect("REGISTRY_PATH environment variable must be set");
```

**Source:** [Rust startup validation best practices](https://users.rust-lang.org/t/best-practices-on-failing-fast-during-start-up/4067)

### Anti-Pattern 4: Forgetting deny_unknown_fields on Nested Types

**What goes wrong:** Adding `#[serde(deny_unknown_fields)]` only to top-level struct, not nested types.

**Why it's bad:**
- Nested types silently ignore unknown fields
- Schema validation is incomplete
- Data errors slip through

**Instead:** Apply to ALL types in the schema:
```rust
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]  // ✓ On top-level
pub struct Registry {
    pub categories: Vec<Category>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]  // ✓ On nested type
pub struct Category {
    pub sources: Vec<Source>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]  // ✓ On deeply nested type
pub struct Source {
    pub rank: u8,
}
```

**Source:** [Serde deny_unknown_fields documentation](https://serde.rs/field-attrs.html)

### Anti-Pattern 5: Not Logging Startup Summary

**What goes wrong:** Loading registry silently, no confirmation of what was loaded.

**Why it's bad:**
- Hard to debug startup issues
- No visibility into configuration
- Can't verify correct data loaded

**Instead:** Log full summary after successful load:
```rust
tracing::info!(
    version = %registry.version,
    updated = %registry.updated,
    categories = registry.categories.len(),
    total_sources = registry.categories.iter().map(|c| c.sources.len()).sum::<usize>(),
    "Registry loaded successfully"
);
```

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Env var parsing | Manual env::var + parse | `envy` crate | Type-safe deserialization, validation, better errors |
| Slug validation | Custom char iteration | `regex` crate | Handles edge cases, well-tested, faster |
| Custom validation | Manual if/else checks | `serde_valid` crate | Declarative, integrates with serde, better error messages |
| Error boilerplate | Manual impl Error | `thiserror` crate | Eliminates boilerplate, same performance |
| JSON pretty-print | Custom formatter | `serde_json::to_string_pretty` | Handles all edge cases |

**Key insight:** Rust has mature crates for common patterns. The ecosystem values composition over custom code. Using established crates reduces bugs and improves maintainability.

## Common Pitfalls

### Pitfall 1: Rust Edition Mismatch in Cargo.toml

**What goes wrong:** Specifying `edition = "2024"` but using older Rust toolchain that doesn't support it.

**Why it happens:** Rust Edition 2024 only became stable with Rust 1.85.0 (released 2025-02-20).

**How to avoid:**
```toml
# Cargo.toml
[package]
edition = "2024"
rust-version = "1.85"  # Minimum Rust version required
```

**Verification:**
```bash
rustc --version  # Must be >= 1.85.0
```

**Warning signs:** Compiler errors about "edition 2024 is unstable" or "unknown edition".

**Source:** [Announcing Rust 1.85.0 and Rust 2024](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/)

### Pitfall 2: Missing JSON Feature Flag for tracing-subscriber

**What goes wrong:** Calling `.json()` on subscriber builder results in compilation error.

**Why it happens:** JSON support is behind a feature flag that's not enabled by default.

**How to avoid:**
```toml
# Cargo.toml
tracing-subscriber = { version = "0.3", features = ["env-filter", "json", "fmt"] }
```

**Warning signs:** Compiler error "no method named `json` found for type `SubscriberBuilder`".

**Source:** [tracing-subscriber JSON format documentation](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/format/struct.Json.html)

### Pitfall 3: Registry Path Relative to Wrong Directory

**What goes wrong:** Registry loads in tests but fails in production because path is relative.

**Why it happens:** Working directory differs between `cargo run` (project root) and deployed binary.

**How to avoid:**
- Always use absolute paths via environment variable
- Fail-fast if REGISTRY_PATH not set (no default)
- Log the exact path being loaded

```rust
let path = env::var("REGISTRY_PATH")
    .expect("REGISTRY_PATH environment variable must be set");

tracing::info!("Loading registry from: {}", path);
```

**Warning signs:** "No such file or directory" errors in production but not development.

### Pitfall 4: Serde Error Messages Missing Line Numbers

**What goes wrong:** JSON parse error says "missing field" but doesn't show where in the file.

**Why it happens:** Default serde_json errors don't include location context.

**How to avoid:** Serde errors automatically include line/column for syntax errors. For better error messages on validation errors, include context:

```rust
let registry: Registry = serde_json::from_str(&contents)
    .map_err(|e| anyhow::anyhow!("Failed to parse registry.json at line {}: {}", e.line(), e.column()))?;
```

**Warning signs:** Spending time searching entire JSON file for the error location.

### Pitfall 5: Empty Endorsements Array Failing Deserialization

**What goes wrong:** Registry with `"endorsements": []` fails to load despite being valid JSON.

**Why it happens:** Misunderstanding of "required but can be empty" - the field must exist, value can be empty array.

**How to avoid:**
```rust
#[derive(Deserialize)]
pub struct Registry {
    pub endorsements: Vec<String>,  // Field is required, but Vec can be empty
}
```

**Correct JSON:**
```json
{
  "endorsements": []  // ✓ Valid - field exists, array is empty
}
```

**Incorrect JSON:**
```json
{
  // ✗ Invalid - field is missing entirely
}
```

**Warning signs:** Error message "missing field `endorsements`" when field seems optional.

## Code Examples

Verified patterns from official sources:

### Complete Startup Sequence

```rust
// src/main.rs
use anyhow::Context;

mod registry;
mod error;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Load configuration from environment
    dotenvy::dotenv().ok();
    let config = load_config()?;

    // 2. Initialize logging based on configuration
    init_logging(&config);
    tracing::info!("Starting 3GS server");

    // 3. Load and validate registry
    let registry = registry::load(&config.registry_path)
        .await
        .context("Failed to load registry at startup")?;

    // 4. Log startup summary
    tracing::info!(
        version = %registry.version,
        updated = %registry.updated,
        categories = registry.categories.len(),
        total_sources = registry.categories.iter().map(|c| c.sources.len()).sum::<usize>(),
        curator = %registry.curator.name,
        "Registry loaded successfully"
    );

    // Phase 1 ends here - no server, no protocol, just validated data
    Ok(())
}

fn load_config() -> anyhow::Result<Config> {
    #[derive(serde::Deserialize)]
    struct Config {
        registry_path: std::path::PathBuf,
        #[serde(default = "default_log_format")]
        log_format: String,
    }

    fn default_log_format() -> String { "pretty".to_string() }

    envy::from_env::<Config>()
        .context("Failed to parse environment variables")
}

fn init_logging(config: &Config) {
    use tracing_subscriber::{fmt, EnvFilter};

    match config.log_format.as_str() {
        "json" => {
            fmt()
                .json()
                .with_env_filter(EnvFilter::from_default_env())
                .with_target(false)
                .with_current_span(false)
                .init();
        }
        _ => {
            fmt()
                .pretty()
                .with_env_filter(EnvFilter::from_default_env())
                .init();
        }
    }
}
```

### Registry Loading with Validation

```rust
// src/registry/loader.rs
use super::{Registry, RegistryError};
use std::path::Path;
use tokio::fs;

pub async fn load<P: AsRef<Path>>(path: P) -> Result<Registry, RegistryError> {
    let path = path.as_ref();

    // Read file asynchronously
    let contents = fs::read_to_string(path)
        .await
        .map_err(|e| RegistryError::FileRead {
            path: path.to_string_lossy().to_string(),
            error: e.to_string(),
        })?;

    // Parse JSON (synchronous, but fast)
    let registry: Registry = serde_json::from_str(&contents)
        .map_err(|e| RegistryError::JsonParse {
            path: path.to_string_lossy().to_string(),
            error: e.to_string(),
            line: e.line(),
            column: e.column(),
        })?;

    // Validate business rules
    registry.validate()?;

    Ok(registry)
}
```

### Custom Validation Implementation

```rust
// src/registry/mod.rs
use regex::Regex;
use std::collections::HashSet;
use std::sync::OnceLock;

static SLUG_REGEX: OnceLock<Regex> = OnceLock::new();

impl Registry {
    pub fn validate(&self) -> Result<(), RegistryError> {
        let slug_regex = SLUG_REGEX.get_or_init(|| {
            Regex::new(r"^[a-z][a-z0-9]*(-[a-z0-9]+)*$").unwrap()
        });

        let mut seen_slugs = HashSet::new();

        for category in &self.categories {
            // Duplicate slug check
            if !seen_slugs.insert(&category.slug) {
                return Err(RegistryError::DuplicateSlug(category.slug.clone()));
            }

            // Kebab-case validation
            if !slug_regex.is_match(&category.slug) {
                return Err(RegistryError::InvalidSlug {
                    slug: category.slug.clone(),
                    expected: "kebab-case (lowercase letters, numbers, hyphens only)",
                });
            }

            // Exactly 3 sources
            if category.sources.len() != 3 {
                return Err(RegistryError::InvalidSourceCount {
                    category: category.slug.clone(),
                    expected: 3,
                    actual: category.sources.len(),
                });
            }

            // At least 3 query patterns
            if category.query_patterns.len() < 3 {
                return Err(RegistryError::InsufficientQueryPatterns {
                    category: category.slug.clone(),
                    minimum: 3,
                    actual: category.query_patterns.len(),
                });
            }

            // Ranks must be exactly [1, 2, 3]
            let mut ranks: Vec<u8> = category.sources.iter().map(|s| s.rank).collect();
            ranks.sort_unstable();
            if ranks != vec![1, 2, 3] {
                return Err(RegistryError::InvalidRanks {
                    category: category.slug.clone(),
                    expected: vec![1, 2, 3],
                    actual: category.sources.iter().map(|s| s.rank).collect(),
                });
            }
        }

        Ok(())
    }
}
```

## State of the Art

| Old Approach | Current Approach (2026) | When Changed | Impact |
|--------------|-------------------------|--------------|--------|
| `log` + `env_logger` | `tracing` + `tracing-subscriber` | ~2020 | Structured logging with spans, async-aware |
| Manual `impl Error` | `thiserror` derive macro | ~2019 | Less boilerplate, same performance |
| `error-chain` | `anyhow` for apps | ~2020 | Simpler API, better ergonomics |
| Rust Edition 2021 | Rust Edition 2024 | 2025-02-20 | New language features, better diagnostics |
| `std::env::var` | `envy` crate | ~2018 | Type-safe environment variable deserialization |

**Deprecated/outdated:**
- `error-chain`: Superseded by `thiserror` + `anyhow` pattern
- `env_logger`: Still maintained but `tracing-subscriber` is more powerful
- Rust 1.84 or earlier for Edition 2024: Requires Rust 1.85+

## Open Questions

Things that couldn't be fully resolved:

1. **Optimal registry file size limit**
   - What we know: Should enforce limit for 512MB Render deployment
   - What's unclear: What size is reasonable for 10 categories with 3 sources each? Likely < 1MB but need to measure actual serialized size
   - Recommendation: Start with 10MB limit (conservative), measure actual size, adjust down if needed

2. **Error message verbosity in production**
   - What we know: Should include validation details for debugging
   - What's unclear: Whether to sanitize file paths in production logs
   - Recommendation: Include full paths in error messages (helps ops), log level controls verbosity

3. **Module organization for future phases**
   - What we know: Phase 1 uses nested modules under `src/`
   - What's unclear: At what point to split into workspace crates (if ever)?
   - Recommendation: Stay with nested modules until codebase exceeds 10k LOC, then re-evaluate

## Sources

### Primary (HIGH confidence)

- [Announcing Rust 1.85.0 and Rust 2024](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/) - Edition 2024 stable release
- [Serde attributes documentation](https://serde.rs/attributes.html) - deny_unknown_fields official docs
- [serde_json from_reader documentation](https://docs.rs/serde_json/latest/serde_json/fn.from_reader.html) - Performance note about from_str
- [tracing-subscriber documentation](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/) - Format options and EnvFilter
- [Rust error handling best practices 2026](https://momori.dev/posts/rust-error-handling-thiserror-anyhow/) - thiserror + anyhow pattern
- [envy crate documentation](https://docs.rs/envy/latest/envy/) - Type-safe env var deserialization

### Secondary (MEDIUM confidence)

- [Rust module organization best practices](https://www.djamware.com/post/68b2c7c451ce620c6f5efc56/rust-project-structure-and-best-practices-for-clean-scalable-code) - Project structure patterns
- [Environment variable best practices](https://www.thorsten-hans.com/working-with-environment-variables-in-rust/) - Configuration management
- [Rust startup validation discussion](https://users.rust-lang.org/t/best-practices-on-failing-fast-during-start-up/4067) - Fail-fast patterns
- [Kebab-case regex pattern](https://regex101.com/library/z7IorM) - Slug validation regex
- [serde validation crates](https://docs.rs/serde_valid/latest/serde_valid/) - Custom validation approaches

### Tertiary (LOW confidence, marked for validation)

- Docker rust:1.85 availability - Search results showed rust:1.93 and rust:1.86, not 1.85 specifically. May be mislabeled or version numbering confusion. Use `rust:1-slim-bookworm` tag for latest stable.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All crates are stable, widely used, well-documented
- Architecture patterns: HIGH - Follows Rust idioms, validated by official docs and community consensus
- Error handling: HIGH - thiserror + anyhow is the 2026 consensus pattern
- Startup validation: HIGH - Well-established patterns in Rust ecosystem
- Rust Edition 2024: HIGH - Official stable release confirmed
- Logging patterns: HIGH - tracing-subscriber officially supports JSON and pretty formats
- Docker images: MEDIUM - Specific rust:1.85 tag not verified, but rust:1-slim-bookworm confirmed available

**Research date:** 2026-02-01
**Valid until:** 2026-03-01 (30 days - Rust ecosystem is stable, no fast-moving changes expected)

**Critical for Phase 1 planning:**
- All locked decisions from CONTEXT.md are well-supported
- No blockers identified
- Standard Rust patterns apply throughout
- Ready for immediate planning and implementation
