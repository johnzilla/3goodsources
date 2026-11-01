# Registry Schema (registry.json)

## Overview

The `registry.json` file is the single source of truth for all curated sources in the 3GS system. It contains categorized, human-vetted resources organized by topic, with each category providing exactly three sources ranked by priority.

This registry is loaded on server startup and validated against strict schema rules to ensure data integrity. The server will fail to start if the registry contains invalid data, making schema violations immediately visible during development and deployment.

## Top-Level Structure

```json
{
  "version": "0.1.0",
  "updated": "2026-02-01",
  "curator": {
    "name": "3GS Curator",
    "pubkey": "qp6xytgxz4f1n3g3uiofktpbwuwzi89a4homcedbpb6ar4edbr5o"
  },
  "endorsements": [],
  "categories": {
    "category-slug": { /* Category object */ }
  },
  "signature": { "alg": "ed25519", "sig": "<128 hex chars>" }
}
```

### Top-Level Fields

- **`version`** (string, required): Semver version string indicating the registry schema version. Example: `"0.1.0"`
- **`updated`** (string, required): ISO 8601 date string indicating the last update to the registry. Example: `"2026-02-01"`
- **`curator`** (object, required): Curator identity information (see Curator Object below)
- **`endorsements`** (array, required): Array of endorsement objects. Empty in v1, reserved for future federated trust features. Example: `[]`
- **`categories`** (object, required): HashMap of categories keyed by slug. Each key is a category slug (e.g., `"rust-learning"`), and each value is a Category object.
- **`signature`** (object, required to start the server): Ed25519 signature by `curator.pubkey` over the rest of the document. See [Signature](#signature) below. Written by `3gs sign-registry`; never edit it by hand.

## Curator Object

Identifies the curator who maintains this registry. The pubkey provides cryptographic identity verification.

```json
{
  "name": "3GS Curator",
  "pubkey": "qp6xytgxz4f1n3g3uiofktpbwuwzi89a4homcedbpb6ar4edbr5o"
}
```

### Curator Fields

- **`name`** (string, required): Human-readable display name for the curator
- **`pubkey`** (string, required): PKARR public key in z-base-32 encoding (52 characters). This key proves the identity of the registry operator and enables cryptographic verification.

`curator.pubkey` must be the public key of the server's `PKARR_SECRET_KEY`, and the registry must be signed with that key. The server refuses to start otherwise (see [Signature](#signature)).

## Category Object

Each category represents a specific topic with curated sources and query patterns for matching.

```json
{
  "name": "Rust Learning",
  "description": "Learning the Rust programming language from basics through advanced concepts including ownership, lifetimes, async programming, and systems design.",
  "query_patterns": [
    "learn rust programming",
    "rust tutorial for beginners",
    "how to get started with rust",
    "best resources for learning rust"
  ],
  "sources": [
    { /* Source object */ },
    { /* Source object */ },
    { /* Source object */ }
  ]
}
```

### Category Fields

- **`name`** (string, required): Human-readable category name. Example: `"Rust Learning"`
- **`description`** (string, required): Detailed description of what this category covers, including key concepts and scope
- **`query_patterns`** (array of strings, required): Natural language query patterns that users might ask when looking for this category. Minimum of 3 patterns required. These patterns are normalized and used for fuzzy matching against user queries.
- **`sources`** (array of Source objects, required): Exactly 3 curated sources. Validation enforces this count strictly in v1.

## Source Object

Individual resource within a category, ranked by priority.

```json
{
  "rank": 1,
  "name": "The Rust Programming Language Book",
  "url": "https://doc.rust-lang.org/book/",
  "type": "book",
  "why": "Official comprehensive guide to Rust, covering fundamentals through advanced topics with clear examples and exercises."
}
```

### Source Fields

- **`rank`** (integer, required): Priority ranking from 1 to 3, where 1 is the primary source. For v1 with exactly 3 sources, ranks must be sequential: 1, 2, 3.
- **`name`** (string, required): Human-readable source display name
- **`url`** (string, required): Full URL to the source resource
- **`type`** (string enum, required): Source type category. Must be one of:
  - `documentation` - Official documentation, API references
  - `tutorial` - Step-by-step guides, walkthroughs
  - `video` - Video content, screencasts, courses
  - `article` - Blog posts, essays, papers
  - `tool` - Software tools, platforms, services
  - `repo` - Code repositories, libraries
  - `forum` - Community forums, discussion boards
  - `book` - Books, ebooks, comprehensive guides
  - `course` - Structured courses, curricula
  - `api` - API endpoints, web services
- **`why`** (string, required): Curator's explanation of why this source is valuable and why it deserves its ranking. This transparency helps users understand the curation rationale.

## Validation Rules

The registry loader enforces these business rules at startup:

### Slug Format

Category slugs must match the regex: `^[a-z0-9]+(-[a-z0-9]+)*$`

- Lowercase letters and numbers only
- Hyphens allowed as separators (not at start/end)
- No consecutive hyphens
- Examples: `rust-learning`, `bitcoin-node-setup`, `self-hosted-email`

### Source Count

Each category must have **exactly 3 sources**. This design constraint forces prioritization and keeps the UX focused. Too few sources (1-2) provide no alternatives; too many (5+) create decision paralysis.

### Source Ranks

Source ranks must be sequential integers: 1, 2, 3. No gaps, no duplicates, no other values. The loader sorts ranks and verifies they match `[1, 2, 3]` exactly.

### Query Patterns

Each category must have a **minimum of 3 query patterns**. More patterns improve matching coverage but aren't required. Patterns should reflect natural language variations of how users might ask for this topic.

### Unknown Fields

All registry structs use `#[serde(deny_unknown_fields)]` in Rust, which means:
- Any field not defined in the schema will cause a parse error
- Typos in field names are caught immediately
- The schema is strictly enforced with no silent ignoring of extra data

This fail-fast approach ensures the registry stays clean and schema violations are impossible to deploy. (Peers parse each other's registries leniently, so a newer peer can add fields; those fields are still covered by its signature.)

## Signature

The registry carries an embedded Ed25519 signature so that anyone holding the curator's public key can check that the sources haven't been altered, whether they got the file from the server, a peer, a mirror, or git.

```json
"signature": { "alg": "ed25519", "sig": "dd3ede96...9b05" }
```

- **`alg`** (string): Always `"ed25519"`.
- **`sig`** (string): Hex-encoded 64-byte Ed25519 signature.

### What is signed

1. Parse the registry JSON.
2. Remove the top-level `signature` field. Everything else is signed, including fields this version of the schema doesn't know about and explicit `null`s.
3. Serialize the rest as **canonical JSON**: no whitespace, object keys sorted by Unicode code point at every level (equivalently, UTF-8 byte order), array order kept, strings escaped the way `serde_json` and Python's `json.dumps(..., ensure_ascii=False)` escape them. The registry only contains strings, integers, booleans, and nulls, so number formatting can't vary.
4. Prefix the bytes with the domain tag `3gs-registry-v1\n` (the literal text plus one newline byte). The tag keeps a registry signature from being reused as any other kind of 3GS signature, such as an audit log entry.
5. The signature is Ed25519 over those bytes, with the key in `curator.pubkey` (z-base-32 decoded to 32 bytes).

In Python this is `b"3gs-registry-v1\n" + json.dumps(doc_without_signature, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()`. A complete verifier is in `scripts/verify_registry.py`.

Because the signed bytes are computed from the parsed document, reformatting the file (indentation, key order) doesn't change the signature; changing any value does.

### Who checks it

- **The server, on startup.** `curator.pubkey` must equal the public key of `PKARR_SECRET_KEY`, and the signature must verify. If either fails, the server exits with an error. `ALLOW_UNSIGNED_REGISTRY=true` downgrades this to a warning, for local development only.
- **Peers, on every fetch.** A node verifies an endorsed peer's `GET /registry` against the pubkey it endorsed, not the key the peer claims. Unsigned, tampered, or foreign-key registries are discarded and the peer is marked `InvalidSignature`.
- **Anyone.** `GET /registry` serves the file byte-for-byte, so you can verify it yourself (see [PUBKY.md](PUBKY.md#registry-signatures)).

### Signing after an edit

Any edit to `registry.json` invalidates the signature. Re-sign it offline:

```bash
PKARR_SECRET_KEY=<64 hex chars> cargo run -- sign-registry   # or put the key in .env
cargo run -- sign-registry --check                           # verify only, no key needed
```

The signer refuses to run if `curator.pubkey` doesn't match the key. It keeps the file's layout and writes the signature as the last top-level field. The test suite (`tests/integration_signing.rs`) fails if the committed `registry.json` isn't signed by the production key.

### Limits

A signature proves who published a registry and that it hasn't changed since. It doesn't prove it's the latest version: someone could replay an older, validly signed registry. The signed `updated` date is the only freshness signal today.

## Full Example

Here's a complete single-category example from the actual registry:

```json
{
  "version": "0.1.0",
  "updated": "2026-02-01",
  "curator": {
    "name": "3GS Curator",
    "pubkey": "qp6xytgxz4f1n3g3uiofktpbwuwzi89a4homcedbpb6ar4edbr5o"
  },
  "endorsements": [],
  "categories": {
    "rust-learning": {
      "name": "Rust Learning",
      "description": "Learning the Rust programming language from basics through advanced concepts including ownership, lifetimes, async programming, and systems design.",
      "query_patterns": [
        "learn rust programming",
        "rust tutorial for beginners",
        "how to get started with rust",
        "best resources for learning rust"
      ],
      "sources": [
        {
          "rank": 1,
          "name": "The Rust Programming Language Book",
          "url": "https://doc.rust-lang.org/book/",
          "type": "book",
          "why": "Official comprehensive guide to Rust, covering fundamentals through advanced topics with clear examples and exercises."
        },
        {
          "rank": 2,
          "name": "Rust by Example",
          "url": "https://doc.rust-lang.org/rust-by-example/",
          "type": "tutorial",
          "why": "Collection of runnable examples illustrating Rust concepts and standard libraries with hands-on code snippets."
        },
        {
          "rank": 3,
          "name": "Zero To Production In Rust",
          "url": "https://www.zero2prod.com/",
          "type": "book",
          "why": "Practical guide to building production-ready backend applications in Rust, covering real-world API development and deployment."
        }
      ]
    }
  },
  "signature": { "alg": "ed25519", "sig": "<128 hex chars>" }
}
```

## Implementation Details

The registry schema is implemented in Rust using serde for serialization:

- **Types defined in:** `src/registry/types.rs`
- **Loader and validator:** `src/registry/loader.rs`
- **Signing and verification:** `src/registry/signing.rs` (`3gs sign-registry` lives in `src/sign.rs`)
- **Validation on startup:** Server crashes with descriptive errors if registry is invalid
- **Thread-safe sharing:** Registry is wrapped in `Arc<Registry>` and shared across HTTP handlers

See `src/registry/types.rs` for the complete Rust type definitions and `src/registry/loader.rs` for the validation logic.
