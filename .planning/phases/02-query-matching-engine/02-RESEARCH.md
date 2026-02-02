# Phase 2: Query Matching Engine - Research

**Researched:** 2026-02-02
**Domain:** Fuzzy string matching and query engines
**Confidence:** HIGH

## Summary

Phase 2 implements a fuzzy query matching engine that maps user queries to registry categories using normalized Levenshtein distance (via the `strsim` crate) combined with keyword boosting. The standard approach uses text normalization (lowercase, punctuation removal, stop word filtering), calculates fuzzy similarity scores against category metadata (query_patterns, slug, display name), adds keyword boost scoring, and returns the best-matching category if above threshold.

The Rust ecosystem provides mature, battle-tested libraries for this domain: `strsim` for string similarity metrics (616M+ downloads), `stop-words` for multilingual stop word lists, and `regex` for text normalization. The implementation pattern is straightforward: create a matcher module with a normalize-score-select pipeline, expose matching configuration through environment variables using `envy`, and use `thiserror` for domain-specific error variants.

**Primary recommendation:** Use `strsim::normalized_levenshtein` (not `jaro_winkler`) for general-purpose query matching against multi-word patterns, implement weighted sum scoring (not multiplicative), normalize queries with a standard 4-step pipeline (lowercase -> strip punctuation -> remove stop words -> normalize whitespace), and expose threshold + weights as environment variables with sensible defaults.

## Standard Stack

The established libraries/tools for fuzzy matching in Rust:

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| strsim | 0.11.1 | String similarity metrics (Levenshtein, Jaro-Winkler, etc.) | 616M+ downloads, official rust-lang ecosystem, zero dependencies, implements all major algorithms |
| stop-words | 0.9.0 | Stop word lists for 100+ languages (NLTK + Stopwords ISO sources) | Standard NLP preprocessing, well-maintained, covers English comprehensively |
| regex | 1.12.2 | Pattern-based text processing for normalization | Already in project Cargo.toml, guaranteed linear-time matching, Unicode-aware |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| approx | 0.5.x | Float comparison in tests (abs_diff_eq!, relative_eq!) | Testing fuzzy scores - avoid direct f64 equality assertions |
| unicode-normalization | 0.1.x | Unicode NFC/NFD normalization | If registry contains non-ASCII category names or patterns (not needed for ASCII-only slugs) |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| strsim | rapidfuzz-rs | rapidfuzz optimizes for speed over binary size - overkill for matching ~10 categories, adds complexity |
| strsim | fuzzy-matcher (skim-rs) | Uses Smith-Waterman algorithm optimized for CLI fuzzy finding - different use case than query matching |
| stop-words | Hand-rolled list | Custom lists miss edge cases, require maintenance, reinvent solved problem |

**Installation:**
```bash
cargo add strsim@0.11.1
cargo add stop-words@0.9.0 --features iso,nltk
cargo add --dev approx@0.5
```

Note: `regex` already present in Cargo.toml at 1.12.2.

## Architecture Patterns

### Recommended Project Structure

```
src/
├── matcher/
│   ├── mod.rs           # Public API: match_query() function + MatchResult type
│   ├── normalize.rs     # Text normalization pipeline
│   ├── scorer.rs        # Fuzzy + keyword scoring logic
│   ├── error.rs         # MatchError enum (thiserror)
│   └── config.rs        # MatchConfig struct (or extend src/config.rs)
└── registry/            # Existing Phase 1 code
    ├── types.rs
    ├── loader.rs
    └── error.rs
```

### Pattern 1: Normalize-Score-Select Pipeline

**What:** Three-stage pipeline for query matching - normalize both query and target strings, score all candidates, select best match above threshold.

**When to use:** Standard approach for fuzzy search against a small (<100) set of candidates.

**Example:**
```rust
// Source: Research synthesis - standard fuzzy matching pattern
pub fn match_query(
    query: &str,
    registry: &Registry,
    config: &MatchConfig,
) -> Result<MatchResult, MatchError> {
    // Stage 1: Normalize query
    let normalized_query = normalize::normalize_text(query)?;

    // Stage 2: Score all categories
    let mut scores: Vec<(String, f64)> = registry
        .categories
        .iter()
        .map(|(slug, category)| {
            let score = scorer::calculate_score(
                &normalized_query,
                slug,
                category,
                config,
            );
            (slug.clone(), score)
        })
        .collect();

    // Stage 3: Select best match
    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let (best_slug, best_score) = scores.first().unwrap();

    if best_score >= config.threshold {
        Ok(MatchResult::Match { slug: best_slug.clone(), score: best_score })
    } else {
        Err(MatchError::BelowThreshold {
            closest_slug: best_slug.clone(),
            closest_score: best_score,
            all_slugs: registry.categories.keys().cloned().collect(),
        })
    }
}
```

### Pattern 2: Weighted Sum Scoring

**What:** Combine multiple scoring signals (fuzzy similarity, keyword presence) using weighted linear combination.

**When to use:** When you have 2-4 distinct scoring signals to combine. Use weighted sum (not multiplicative) to allow tuning individual signal importance.

**Example:**
```rust
// Source: https://www.flagright.com/post/jaro-winkler-vs-levenshtein-choosing-the-right-algorithm-for-aml-screening
// Source: https://dataladder.com/fuzzy-matching-101/
pub fn calculate_score(
    query: &str,
    slug: &str,
    category: &Category,
    config: &MatchConfig,
) -> f64 {
    // Signal 1: Fuzzy similarity (match against patterns, slug, name)
    let fuzzy_score = calculate_fuzzy_score(query, slug, category);

    // Signal 2: Keyword boost (query contains slug terms)
    let keyword_score = calculate_keyword_score(query, slug);

    // Weighted sum combination (not multiplicative - allows independent tuning)
    let combined = (config.fuzzy_weight * fuzzy_score)
                 + (config.keyword_weight * keyword_score);

    combined.min(1.0) // Clamp to [0.0, 1.0] range
}

fn calculate_fuzzy_score(query: &str, slug: &str, category: &Category) -> f64 {
    // Best score across all match surfaces
    let mut max_score = strsim::normalized_levenshtein(query, slug);
    max_score = max_score.max(strsim::normalized_levenshtein(query, &category.name.to_lowercase()));

    for pattern in &category.query_patterns {
        let pattern_normalized = normalize::normalize_text(pattern).unwrap();
        let score = strsim::normalized_levenshtein(query, &pattern_normalized);
        max_score = max_score.max(score);
    }

    max_score
}
```

### Pattern 3: Multi-Stage Text Normalization

**What:** Apply text transformations in a specific order to create canonical form for matching.

**When to use:** Always normalize both queries and match targets before scoring. Order matters.

**Example:**
```rust
// Source: https://apxml.com/courses/nlp-fundamentals/chapter-1-nlp-text-processing-techniques/text-normalization-techniques
// Source: https://towardsdatascience.com/text-normalization-7ecc8e084e31/
use regex::Regex;
use stop_words::{get, LANGUAGE};

pub fn normalize_text(text: &str) -> Result<String, MatchError> {
    if text.trim().is_empty() {
        return Err(MatchError::EmptyQuery);
    }

    // Stage 1: Lowercase
    let mut normalized = text.to_lowercase();

    // Stage 2: Strip punctuation (keep letters, digits, spaces)
    let punct_re = Regex::new(r"[^\w\s]").unwrap(); // \w = letters+digits, \s = whitespace
    normalized = punct_re.replace_all(&normalized, "").to_string();

    // Stage 3: Remove stop words
    let stop_words: HashSet<String> = get(LANGUAGE::English)
        .into_iter()
        .map(|s| s.to_string())
        .collect();

    normalized = normalized
        .split_whitespace()
        .filter(|word| !stop_words.contains(*word))
        .collect::<Vec<_>>()
        .join(" ");

    // Stage 4: Normalize whitespace (trim + collapse multiple spaces)
    let ws_re = Regex::new(r"\s+").unwrap();
    normalized = ws_re.replace_all(normalized.trim(), " ").to_string();

    if normalized.is_empty() {
        return Err(MatchError::QueryAllStopWords);
    }

    Ok(normalized)
}
```

### Pattern 4: Configuration via Environment Variables

**What:** Expose matching tuning parameters (threshold, weights) as environment variables with sensible defaults.

**When to use:** When values need runtime configuration without code changes. Use `envy` for type-safe deserialization.

**Example:**
```rust
// Source: https://docs.rs/envy/latest/envy/
// Source: https://github.com/softprops/envy
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MatchConfig {
    /// Minimum score to accept a match (default: 0.4)
    #[serde(default = "default_threshold")]
    pub match_threshold: f64,

    /// Weight for fuzzy similarity score (default: 0.7)
    #[serde(default = "default_fuzzy_weight")]
    pub match_fuzzy_weight: f64,

    /// Weight for keyword boost score (default: 0.3)
    #[serde(default = "default_keyword_weight")]
    pub match_keyword_weight: f64,
}

fn default_threshold() -> f64 { 0.4 }
fn default_fuzzy_weight() -> f64 { 0.7 }
fn default_keyword_weight() -> f64 { 0.3 }

impl MatchConfig {
    pub fn load() -> Result<Self, anyhow::Error> {
        envy::from_env::<MatchConfig>()
            .map_err(|e| anyhow::anyhow!("Failed to load match config: {}", e))
    }

    pub fn validate(&self) -> Result<(), anyhow::Error> {
        if self.match_threshold < 0.0 || self.match_threshold > 1.0 {
            anyhow::bail!("MATCH_THRESHOLD must be between 0.0 and 1.0");
        }
        if (self.match_fuzzy_weight + self.match_keyword_weight - 1.0).abs() > 0.01 {
            anyhow::bail!("MATCH_FUZZY_WEIGHT + MATCH_KEYWORD_WEIGHT must sum to 1.0");
        }
        Ok(())
    }
}
```

**Environment variables:**
```bash
MATCH_THRESHOLD=0.4        # Minimum score (0.0-1.0)
MATCH_FUZZY_WEIGHT=0.7     # Fuzzy similarity weight
MATCH_KEYWORD_WEIGHT=0.3   # Keyword boost weight
```

### Anti-Patterns to Avoid

- **Don't use multiplicative scoring** - `fuzzy_score * keyword_score` makes signals interdependent. One zero kills the score. Use weighted sum instead.
- **Don't match against descriptions** - Too noisy, leads to false positives. Match only against query_patterns, slug, and name.
- **Don't hardcode threshold/weights** - Environment variables allow tuning without rebuilds. Always use `#[serde(default)]` for optional config.
- **Don't normalize after scoring** - Normalize once upfront, not per comparison. Cache normalized patterns if needed.
- **Don't use `assert_eq!` for float scores** - Floats have precision errors. Use `approx::assert_relative_eq!` in tests.
- **Don't skip empty query validation** - `normalized_levenshtein("", "anything")` returns 0.0. Check emptiness first and return helpful error.

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Stop word list | Custom Vec<&str> with common words | `stop-words` crate with LANGUAGE::English | NLTK's list has 127 words covering edge cases (e.g., "shan't", "mustn't", "mightn't"). Hand-rolled lists miss these. |
| Text normalization | String.chars().filter().collect() | `regex` with `replace_all()` | Unicode edge cases (combining characters, non-breaking spaces). Regex handles `\w` and `\s` character classes correctly. |
| Float comparison | `score1 == score2` | `approx` crate macros | Floating-point arithmetic introduces rounding errors. `0.1 + 0.2 != 0.3` in binary. |
| Levenshtein distance | Custom DP matrix implementation | `strsim::normalized_levenshtein` | Optimized implementation, normalized to [0.0, 1.0], handles empty strings, well-tested. |

**Key insight:** Text processing has many Unicode and edge case pitfalls. Normalized Levenshtein looks simple (2D array DP) but production implementations handle empty strings, Unicode grapheme clusters, and performance optimizations. Don't reinvent.

## Common Pitfalls

### Pitfall 1: Choosing Wrong Similarity Metric

**What goes wrong:** Using `jaro_winkler` for multi-word query patterns because it's "faster" or "better for names". Gets poor results on long queries.

**Why it happens:** Jaro-Winkler emphasizes prefix matching and is optimized for single-word name comparisons. Query patterns like "how to self host email server" don't benefit from prefix weighting.

**How to avoid:** Use `normalized_levenshtein` for general-purpose multi-word query matching. Reserve `jaro_winkler` for short single-word scenarios where prefix matters (autocomplete, name matching).

**Warning signs:**
- User queries "email server" but doesn't match category with pattern "self-hosted email setup" (Jaro-Winkler penalizes lack of common prefix)
- Short queries work well but long queries fail (Jaro-Winkler prefix boost becomes insignificant in long strings)

**Source:** [Jaro-Winkler vs. Levenshtein in AML Screening](https://www.flagright.com/post/jaro-winkler-vs-levenshtein-choosing-the-right-algorithm-for-aml-screening), [Medium: Jaro Winkler vs Levenshtein Distance](https://srinivas-kulkarni.medium.com/jaro-winkler-vs-levenshtein-distance-2eab21832fd6)

### Pitfall 2: Wrong Normalization Order

**What goes wrong:** Removing stop words before stripping punctuation leaves artifacts. Example: "don't" -> (strip stop "don't") -> "don't" still present -> (strip punct) -> "dont" (malformed).

**Why it happens:** Punctuation can be part of stop words (contractions like "don't", "I'm"). Order matters in text pipelines.

**How to avoid:** Always follow this order: lowercase -> strip punctuation -> remove stop words -> normalize whitespace. This ensures "Don't run!" -> "dont run" -> "run" -> "run".

**Warning signs:**
- Normalized queries contain fragments like "dont", "im", "cant" (should be removed as stop words)
- Queries with punctuation behave differently than queries without

**Source:** [Text Normalization Techniques in NLP](https://apxml.com/courses/nlp-fundamentals/chapter-1-nlp-text-processing-techniques/text-normalization-techniques)

### Pitfall 3: Threshold Too High or Too Low

**What goes wrong:** Threshold at 0.8+ rejects valid fuzzy matches. Threshold at 0.2 returns garbage matches.

**Why it happens:** Different domains need different thresholds. Financial systems use 0.85-0.95 (high precision), marketing/CRM uses 0.6-0.75 (higher recall).

**How to avoid:**
- Start with 0.4 threshold for general-purpose query matching (allows fuzzy matches while filtering noise)
- Make threshold configurable via environment variable
- Log closest match and score even on failure to tune threshold empirically
- For this phase: 0.4 is a good balance for user-facing query search

**Warning signs:**
- Users complain "search doesn't find anything" (threshold too high)
- Irrelevant categories returned frequently (threshold too low)
- Tuning threshold requires code changes (not configurable)

**Source:** [Fuzzy Matching 101](https://dataladder.com/fuzzy-matching-101/), [Normalized Levenshtein Threshold Guide](https://medium.com/analytics-vidhya/fuzzy-matching-in-python-2def168dee4a)

### Pitfall 4: Empty String Edge Cases

**What goes wrong:** Calling `normalized_levenshtein("", "bitcoin")` returns 0.0. If query normalizes to empty string (all stop words), matcher silently returns score 0.0 for all categories.

**Why it happens:** After removing stop words, some queries become empty ("how do I the"). `strsim` returns 0.0 for any empty string comparison.

**How to avoid:** Validate after normalization:
```rust
if normalized_query.is_empty() {
    return Err(MatchError::QueryAllStopWords);
}
```

Also check upfront:
```rust
if query.trim().is_empty() {
    return Err(MatchError::EmptyQuery);
}
```

**Warning signs:**
- Queries like "the a an" fail silently with no helpful error
- Error messages don't distinguish between empty input vs all-stop-words

**Source:** [String Similarity Edge Cases](https://winpure.com/fuzzy-matching-common-mistakes/)

### Pitfall 5: Unicode Handling Assumptions

**What goes wrong:** User queries contain Unicode characters (e.g., "café", "naïve", "Müller") and matching fails or behaves unexpectedly.

**Why it happens:** Assuming all text is ASCII. Rust strings are UTF-8 by default, but normalization (punctuation removal, case folding) can have Unicode edge cases.

**How to avoid:**
- Rust's `to_lowercase()` handles Unicode correctly (ß -> ss, İ -> i̇)
- Regex `\w` includes Unicode letters, not just ASCII
- For production, consider `unicode-normalization` crate if registry has non-ASCII content
- For this phase (ASCII slugs only), default handling is sufficient

**Warning signs:**
- Queries with accented characters fail to match
- Case conversion produces wrong results for non-ASCII

**Source:** [Unicode in Rust](https://users.rust-lang.org/t/unicode-string-segmentation-normalization-matching-and-tries/92573), [unicode-normalization crate](https://docs.rs/unicode-normalization/0.1.19/unicode_normalization/)

### Pitfall 6: Float Comparison in Tests

**What goes wrong:** Test `assert_eq!(score, 0.7)` fails even when score is correct due to floating-point precision errors.

**Why it happens:** Binary floating-point can't represent some decimals exactly. `0.1 + 0.2 != 0.3` in IEEE 754.

**How to avoid:** Use `approx` crate for tests:
```rust
use approx::assert_relative_eq;

assert_relative_eq!(score, 0.7, epsilon = 1e-6);
```

**Warning signs:**
- Tests fail with messages like "left: 0.6999999999999999, right: 0.7"
- Flaky tests that sometimes pass, sometimes fail

**Source:** [Rust Float Comparison](https://users.rust-lang.org/t/assert-eq-for-float-numbers/7034), [approx crate docs](https://docs.rs/approx)

## Code Examples

Verified patterns from official sources:

### Normalized Levenshtein Usage

```rust
// Source: https://docs.rs/strsim/0.11.1/strsim/fn.normalized_levenshtein.html
use strsim::normalized_levenshtein;

let score = normalized_levenshtein("kitten", "sitting");
// Returns: 0.5714285714285714 (roughly 57% similar)

// Edge cases:
assert_eq!(normalized_levenshtein("", ""), 1.0);      // Both empty = identical
assert_eq!(normalized_levenshtein("", "abc"), 0.0);   // Empty vs non-empty = no match
assert_eq!(normalized_levenshtein("abc", "abc"), 1.0); // Identical = perfect match
```

**Key behavior:**
- Returns f64 in range [0.0, 1.0]
- 1.0 = identical strings
- 0.0 = completely different (or one is empty)
- Normalized by length of longer string

### Stop Words Filtering

```rust
// Source: https://docs.rs/stop-words/latest/stop_words/
use stop_words::{get, LANGUAGE};
use std::collections::HashSet;

fn remove_stop_words(text: &str) -> String {
    let stop_words: HashSet<String> = get(LANGUAGE::English)
        .into_iter()
        .map(|s| s.to_string())
        .collect();

    text.split_whitespace()
        .filter(|word| !stop_words.contains(*word))
        .collect::<Vec<_>>()
        .join(" ")
}

// Example:
let result = remove_stop_words("how do i run a bitcoin node");
// Returns: "run bitcoin node"
```

**Stop words included:** i, me, my, you, a, an, the, is, are, was, were, have, has, do, does, how, when, where, why, and, or, but, if, because, of, at, by, for, with, in, on, under, very, too, only, so, then, now, here, there, etc. (~127 words from NLTK)

### Regex Text Normalization

```rust
// Source: https://docs.rs/regex/latest/regex/
use regex::Regex;

fn strip_punctuation(text: &str) -> String {
    // \w matches letters, digits, underscore (Unicode-aware)
    // \s matches whitespace
    // [^\w\s] matches everything except word chars and whitespace
    let re = Regex::new(r"[^\w\s]").unwrap();
    re.replace_all(text, "").to_string()
}

fn normalize_whitespace(text: &str) -> String {
    let re = Regex::new(r"\s+").unwrap();
    re.replace_all(text.trim(), " ").to_string()
}

// Examples:
assert_eq!(strip_punctuation("don't panic!"), "dont panic");
assert_eq!(normalize_whitespace("  too   many    spaces  "), "too many spaces");
```

### Float Comparison in Tests

```rust
// Source: https://docs.rs/approx
use approx::assert_relative_eq;

#[test]
fn test_fuzzy_score() {
    let score = calculate_score("learn rust", "rust-learning", &config);

    // BAD: assert_eq!(score, 0.85); // May fail due to floating-point precision

    // GOOD: Use approx for tolerance-based comparison
    assert_relative_eq!(score, 0.85, epsilon = 1e-6);
}
```

### Error Enum with thiserror

```rust
// Source: https://github.com/dtolnay/thiserror, existing codebase pattern
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MatchError {
    #[error("Query cannot be empty")]
    EmptyQuery,

    #[error("Query contains only stop words and has no searchable content")]
    QueryAllStopWords,

    #[error(
        "No category matches query (threshold: {threshold}). \
         Closest: {closest_slug} ({closest_score:.2}). \
         Available categories: {all_slugs:?}. \
         Request a new category at https://github.com/johnzilla/3goodsources"
    )]
    BelowThreshold {
        threshold: f64,
        closest_slug: String,
        closest_score: f64,
        all_slugs: Vec<String>,
    },
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Hand-coded Levenshtein | `strsim` crate | Pre-2015 | Mature, well-tested implementation with 616M+ downloads |
| Single similarity metric | Weighted multi-signal scoring | 2020s | Better relevance through combining fuzzy + keyword signals |
| Hardcoded thresholds | Environment variable configuration | Modern best practice | Runtime tuning without recompilation |
| ASCII-only text handling | Unicode-aware by default (Rust) | Rust 1.0+ (2015) | Correct handling of international text |
| `assert_eq!` for floats | `approx` crate | Established pattern | Reliable float comparisons in tests |

**Deprecated/outdated:**
- **Jaro-Winkler for long text** - Research shows normalized Levenshtein performs better for multi-word queries and long strings. Jaro-Winkler best for short single-word name matching.
- **Multiplicative score combination** - Modern ranking uses weighted sum (linear) for interpretability and independent signal tuning. Multiplicative coupling makes one zero signal kill entire score.
- **Ignoring stop words** - Old search engines kept all words. Modern NLP removes stop words to focus on content-bearing terms, improving precision.

## Open Questions

Things that couldn't be fully resolved:

1. **Optimal default weights (fuzzy vs keyword)**
   - What we know: Industry uses 0.6-0.8 for primary signal, 0.2-0.4 for secondary. Context document suggests 0.7/0.3.
   - What's unclear: Ideal ratio depends on data. Registry has good query_patterns (favors fuzzy) but keyword boosting helps with direct slug mentions.
   - Recommendation: Start with 0.7 fuzzy / 0.3 keyword as suggested. Make configurable. Log scores during testing to tune empirically.

2. **Whether to cache normalized patterns**
   - What we know: Registry has ~10 categories with 3-5 patterns each (~40 strings to normalize). Normalization is cheap (regex replacements, stop word lookup).
   - What's unclear: Whether caching normalized patterns at load time improves performance meaningfully.
   - Recommendation: Don't optimize prematurely. Match all categories on every query (simple implementation). Profile if performance becomes an issue. With ~10 categories, brute-force scoring is fine.

3. **Stop word list completeness**
   - What we know: NLTK list has 127 words. Stopwords ISO has similar coverage. Both well-established.
   - What's unclear: Whether domain-specific stop words ("source", "good", "best") should be added for this use case.
   - Recommendation: Start with standard NLTK English list. Monitor query logs. Add domain-specific stop words only if they cause false positives.

4. **Unicode normalization necessity**
   - What we know: Registry slugs are ASCII-only (lowercase-alphanumeric-hyphens). User queries might contain Unicode.
   - What's unclear: Whether to normalize Unicode (NFC/NFD) or just handle via `to_lowercase()`.
   - Recommendation: Rust's default `to_lowercase()` handles most Unicode correctly. Don't add `unicode-normalization` crate unless testing reveals issues with accented queries.

## Sources

### Primary (HIGH confidence)

- [strsim crate docs (0.11.1)](https://docs.rs/strsim/latest/strsim/) - Official API documentation, function signatures, return values
- [strsim crates.io metadata](https://crates.io/crates/strsim) - Version 0.11.1, published 2024-04-02, 616M+ downloads
- [stop-words crate docs](https://docs.rs/stop-words/latest/stop_words/) - API, language support, NLTK + Stopwords ISO sources
- [stop-words crates.io metadata](https://crates.io/crates/stop-words) - Version 0.9.0, published 2025-08-22
- [envy crate docs](https://docs.rs/envy/latest/envy/) - Environment variable deserialization, supported types, examples
- [regex crate docs](https://docs.rs/regex/latest/regex/) - Pattern syntax, replace_all method, character classes
- [approx crate docs](https://docs.rs/approx) - Float comparison macros for tests
- [thiserror GitHub](https://github.com/dtolnay/thiserror) - Error derive macro patterns, best practices
- [NLTK English stop words list](https://gist.githubusercontent.com/sebleier/554280/raw/7e0e4a1ce04c2bb7bd41089c9821dbcf6d0c786c/NLTK's%2520list%2520of%2520english%2520stopwords) - Actual 127-word list

### Secondary (MEDIUM confidence)

- [Jaro-Winkler vs. Levenshtein in AML Screening](https://www.flagright.com/post/jaro-winkler-vs-levenshtein-choosing-the-right-algorithm-for-aml-screening) - Algorithm comparison, when to use each, industry thresholds
- [Medium: Jaro Winkler vs Levenshtein Distance](https://srinivas-kulkarni.medium.com/jaro-winkler-vs-levenshtein-distance-2eab21832fd6) - Practical differences, use cases
- [Fuzzy Matching 101 (Data Ladder)](https://dataladder.com/fuzzy-matching-101/) - Weighted scoring, threshold selection, best practices
- [WinPure Common Fuzzy Matching Mistakes](https://winpure.com/fuzzy-matching-common-mistakes/) - Edge cases, pitfalls
- [Text Normalization Techniques in NLP](https://apxml.com/courses/nlp-fundamentals/chapter-1-nlp-text-processing-techniques/text-normalization-techniques) - Normalization order, best practices
- [Rust Keyword Extraction (TF-IDF Tutorial)](https://dev.to/tugascript/rust-keyword-extraction-creating-the-tf-idf-algorithm-from-scratch-57a) - Tokenization patterns
- [Understanding Ranking Algorithms](https://spotintelligence.com/2024/07/26/ranking-algorithms/) - Weighted combination approaches
- [Fuzzy Matching at Scale (Medium)](https://medium.com/trusted-data-science-haleon/fuzzy-matching-at-scale-part-i-4621b0b36ba5) - Multi-signal scoring patterns
- [GitHub: rust-stop-words](https://github.com/cmccomb/rust-stop-words) - Stop words source repository

### Tertiary (LOW confidence)

- [rapidfuzz-rs GitHub](https://github.com/rapidfuzz/rapidfuzz-rs) - Alternative library (not chosen due to complexity/size tradeoff)
- [fuzzy-matcher GitHub](https://github.com/skim-rs/fuzzy-matcher) - Alternative approach (Smith-Waterman for CLI fuzzy finding)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - strsim (616M downloads, official ecosystem), stop-words (well-maintained, dual sources), regex (already in use)
- Architecture: HIGH - Patterns verified from official docs, existing codebase follows similar structure (registry module pattern)
- Pitfalls: MEDIUM-HIGH - Algorithm selection guidance from industry sources, text processing pitfalls from NLP best practices, some based on general experience rather than Rust-specific sources

**Research date:** 2026-02-02
**Valid until:** 2026-04-02 (60 days - stable ecosystem, mature libraries)
