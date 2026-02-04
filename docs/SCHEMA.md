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
    "pubkey": "pk:placeholder"
  },
  "endorsements": [],
  "categories": {
    "category-slug": { /* Category object */ }
  }
}
```

### Top-Level Fields

- **`version`** (string, required): Semver version string indicating the registry schema version. Example: `"0.1.0"`
- **`updated`** (string, required): ISO 8601 date string indicating the last update to the registry. Example: `"2026-02-01"`
- **`curator`** (object, required): Curator identity information (see Curator Object below)
- **`endorsements`** (array, required): Array of endorsement objects. Empty in v1, reserved for future federated trust features. Example: `[]`
- **`categories`** (object, required): HashMap of categories keyed by slug. Each key is a category slug (e.g., `"rust-learning"`), and each value is a Category object.

## Curator Object

Identifies the curator who maintains this registry. The pubkey provides cryptographic identity verification.

```json
{
  "name": "3GS Curator",
  "pubkey": "pk:placeholder"
}
```

### Curator Fields

- **`name`** (string, required): Human-readable display name for the curator
- **`pubkey`** (string, required): PKARR public key in z-base-32 encoding (52 characters). This key proves the identity of the registry operator and enables cryptographic verification.

Note: In production, the server's live public key (from `PKARR_SECRET_KEY` environment variable) is used for verification, not the static registry value. The registry pubkey is a placeholder; the `get_provenance` MCP tool returns the actual server keypair's public key.

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

This fail-fast approach ensures the registry stays clean and schema violations are impossible to deploy.

## Full Example

Here's a complete single-category example from the actual registry:

```json
{
  "version": "0.1.0",
  "updated": "2026-02-01",
  "curator": {
    "name": "3GS Curator",
    "pubkey": "pk:placeholder"
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
  }
}
```

## Implementation Details

The registry schema is implemented in Rust using serde for serialization:

- **Types defined in:** `src/registry/types.rs`
- **Loader and validator:** `src/registry/loader.rs`
- **Validation on startup:** Server crashes with descriptive errors if registry is invalid
- **Thread-safe sharing:** Registry is wrapped in `Arc<Registry>` and shared across HTTP handlers

See `src/registry/types.rs` for the complete Rust type definitions and `src/registry/loader.rs` for the validation logic.
