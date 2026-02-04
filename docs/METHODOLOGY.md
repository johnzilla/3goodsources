# Source Curation Methodology

## Overview

This document explains the principles, criteria, and processes used to curate sources in the 3GS registry. It covers why we chose the "three sources" constraint, what makes a source worth including, how sources are ranked, and the technical details of the query matching algorithm.

The goal is transparency: you should understand not just what sources are included, but why they were chosen and how the system matches your queries to categories.

## Why Three?

Each category in the 3GS registry contains **exactly three sources**. This is a deliberate design constraint, not an arbitrary limit.

### The Problem with Too Few

With only 1-2 sources per topic:
- No alternatives or second opinions
- Single point of failure if the source goes offline
- Limited perspective on the topic
- Users forced to trust a single viewpoint

### The Problem with Too Many

With 5+ sources per topic:
- **Decision paralysis** - too many options overwhelm users
- Diluted quality - hard to maintain high curation standards
- Unclear prioritization - which source should I start with?
- Maintenance burden - more sources = more link rot monitoring

### Three is the Balance

Three sources force prioritization while providing diversity:

1. **Primary source** - Usually official documentation or the authoritative reference
2. **Practical complement** - A hands-on tutorial, tool, or applied resource
3. **Alternative perspective** - Community knowledge, different approach, or specialized focus

This structure gives agents (and humans) a clear starting point, a practical next step, and a fallback option. It's enough variety without overwhelming choice.

## What Makes a Source "Good"?

Not all resources are created equal. 3GS curates sources based on five core criteria:

### 1. Authoritative

Sources should come from:
- Official project documentation (e.g., Rust Book from rust-lang.org)
- Primary authors or maintainers (e.g., Bitcoin Core docs from bitcoin.org)
- Recognized domain experts (e.g., EFF for security topics)
- Established community consensus (e.g., ArchWiki for Linux topics)

**Red flags:** Random blog posts, uncredited tutorials, abandoned projects, self-promotional content

### 2. Current

Sources must be:
- **Actively maintained** - Recent updates, no "last modified 2015" staleness
- **Relevant to current versions** - No outdated Python 2 tutorials in 2026
- **Living documents** - Preferably maintained by active projects, not one-off articles

**Exception:** Some timeless resources (e.g., threat modeling principles) age well. Currency matters most for rapidly evolving technologies.

### 3. Practical

Sources should provide **actionable information**:
- Step-by-step guides you can follow
- Working code examples you can run
- Tools you can actually use
- Concepts explained with real-world application

**Avoid:** Pure theory without application, vague high-level overviews, marketing fluff

### 4. Accessible

Sources must be:
- **Publicly available** - No paywalls, no login requirements (exception: freemium tools with free tiers)
- **Free or freemium** - Priority to free resources; paid resources only if uniquely valuable
- **Stable URLs** - Hosted on reliable infrastructure, not likely to disappear

**Note:** Books like "Zero to Production in Rust" are included when they provide unique value despite being paid resources. The URL should lead to legitimate purchase/preview, not pirated copies.

### 5. Diverse

Within each category's three sources, aim for **complementary types**:
- Documentation + Tutorial + Tool
- Official docs + Community guide + Working repo
- Reference + Course + Forum

Different source types serve different needs. A trio of three documentation sites provides less value than doc + tutorial + hands-on tool.

## Source Type Taxonomy

The registry uses 10 source types to categorize resources:

| Type | When to Use | Example |
|------|-------------|---------|
| `documentation` | Official docs, API references, spec sheets | Bitcoin Core Documentation |
| `tutorial` | Step-by-step guides, walkthroughs, how-tos | Raspibolt Guide |
| `video` | Video content, screencasts, recorded courses | (not yet in seed data) |
| `article` | Blog posts, essays, whitepapers | ArchWiki Security Guide |
| `tool` | Software tools, platforms, services | Umbrel, Mail-in-a-Box |
| `repo` | Code repositories, libraries, reference implementations | nostr-tools, Pubky Core |
| `forum` | Community forums, Q&A sites, discussion boards | (not yet in seed data) |
| `book` | Books, ebooks, comprehensive long-form guides | The Rust Programming Language |
| `course` | Structured courses, curricula, learning paths | (not yet in seed data) |
| `api` | API endpoints, web services, data sources | (not yet in seed data) |

Use the type that best describes the **primary interface** of the resource. If it's a book with a companion website, it's type `book`. If it's a tool with extensive documentation, it's type `tool`.

## Ranking Methodology

Each source has a rank from 1-3. Here's how to assign ranks:

### Rank 1: Primary Official Source

Almost always the **official documentation** or canonical reference:
- Bitcoin Node Setup → Bitcoin Core Documentation (official)
- Rust Learning → The Rust Programming Language Book (official)
- Password Management → Bitwarden Help Center (official product docs)

**Ask:** If someone has time for only ONE resource, what's the authoritative starting point?

### Rank 2: Best Practical Complement

The resource that **fills the gap** left by Rank 1:
- If Rank 1 is docs → Rank 2 is a tutorial or tool
- If Rank 1 is theory → Rank 2 is hands-on practice
- If Rank 1 is reference → Rank 2 is a guided tour

Examples:
- Bitcoin Node Setup → Umbrel (tool for hands-on node running)
- Rust Learning → Rust by Example (hands-on code examples)
- Self-Hosted Email → NSA's Email Self-Defense Guide (practical privacy tutorial)

**Ask:** What resource best complements the official docs with practical application?

### Rank 3: Community or Alternative Perspective

The resource that provides:
- **Different angle** - Alternative approach or philosophy
- **Community knowledge** - Collective wisdom, forum discussions
- **Specialized focus** - Deep dive into a subset of the topic

Examples:
- Bitcoin Node Setup → Raspibolt Guide (community DIY tutorial)
- Rust Learning → Zero to Production in Rust (real-world backend focus)
- Linux Hardening → ArchWiki Security Guide (community-maintained depth)

**Ask:** What resource provides valuable perspective beyond the first two?

## Worked Example: rust-learning

Let's walk through the `rust-learning` category to see curation in action.

### The Three Sources

1. **Rank 1: The Rust Programming Language Book**
   - Type: `book`
   - URL: https://doc.rust-lang.org/book/
   - Why: Official comprehensive guide from the Rust team. If you're learning Rust, this is the canonical starting point. Covers fundamentals through advanced topics with clear explanations.

2. **Rank 2: Rust by Example**
   - Type: `tutorial`
   - URL: https://doc.rust-lang.org/rust-by-example/
   - Why: Complements The Book with hands-on code snippets. Also official (rust-lang.org), but focuses on learning-by-doing. Perfect second step after reading theory.

3. **Rank 3: Zero to Production in Rust**
   - Type: `book`
   - Why: Practical, production-focused guide to building real backend systems. Fills a gap: official docs teach language, but this teaches *building applications*. Different perspective from the official learning path.

### What Was Rejected?

Several good resources didn't make the cut:

- **Rustlings** - Excellent exercise-based learning tool, but focuses on small exercises rather than comprehensive learning. The Book + Examples cover the fundamentals better.
- **Rust subreddit** - Great community, but forums are better for specific questions than structured learning. Not a top-3 resource for "learn rust" queries.
- **Random blog series** - Many good blog posts exist ("Learning Rust as a Python developer", etc.), but they lack the authority and comprehensiveness of official resources.
- **Video courses** - Several exist, but text-based resources (book + examples) are more searchable and have better official support.

The three chosen sources provide: official reference (Book) + official hands-on (Examples) + production application focus (Zero to Production). They complement each other without overlap.

### Query Patterns

The `rust-learning` category includes these query patterns:
- "learn rust programming"
- "rust tutorial for beginners"
- "how to get started with rust"
- "best resources for learning rust"

These reflect natural language variations of the same intent: *I want to learn Rust from scratch.* More patterns improve matching coverage.

## Query Matching Algorithm

When an agent sends a query like "how do I learn rust", the system needs to find the best category. Here's the technical matching pipeline:

### Stage 1: Normalization

The query is normalized through a 4-stage pipeline (see `src/matcher/normalize.rs`):

1. **Lowercase:** "How Do I Learn Rust" → "how do i learn rust"
2. **Strip punctuation:** "don't panic!" → "dont panic"
3. **Remove stop words:** Uses NLTK English stop word list (via `stop-words` crate). Removes: "the", "a", "an", "how", "do", "I", "to", "is", etc.
   - Example: "how do i learn rust" → "learn rust"
4. **Normalize whitespace:** Trim and collapse multiple spaces to single spaces

**Error handling:**
- Empty query → `EmptyQuery` error
- Query with only stop words → `QueryAllStopWords` error

### Stage 2: Fuzzy Scoring

For each category, calculate normalized Levenshtein distance (via `strsim` crate) across three **match surfaces**:

1. **Query patterns (normalized):** Each category's `query_patterns` array is normalized and compared to the query
   - Example: "learn rust programming" (normalized) vs "learn rust" (query)
2. **Slug (hyphens → spaces):** Category slug with hyphens replaced by spaces
   - Example: "rust-learning" → "rust learning" vs "learn rust"
3. **Category name (lowercased):** Category name lowercased for comparison
   - Example: "Rust Learning" → "rust learning" vs "learn rust"

The **best match** across all three surfaces is the fuzzy score for that category.

**Why these surfaces?**
- Query patterns: Designed specifically for natural language matching
- Slug: Technical identifier often contains keywords
- Name: Human-readable label captures the topic

**Why NOT match against description?**
- Descriptions are long and contain many tangential terms
- Fuzzy matching against descriptions is too noisy
- Query patterns provide better signal for user intent

### Stage 3: Keyword Boosting

Count how many **slug terms** appear in the normalized query:

1. Split slug on hyphens: "bitcoin-node-setup" → ["bitcoin", "node", "setup"]
2. Count matches: How many terms appear in normalized query?
   - Query "bitcoin node" → matches 2/3 terms → 0.67 keyword score
3. Score = matches / total_slug_terms

**Why keyword boosting?**
- Rewards exact term matches (high precision signal)
- Complements fuzzy matching (which is more forgiving)
- Weighted separately so both signals contribute

### Stage 4: Score Combination

Combine fuzzy and keyword scores with configurable weights:

```
final_score = (fuzzy_weight × fuzzy_score) + (keyword_weight × keyword_score)
```

**Default weights:**
- `fuzzy_weight`: 0.7 (70%)
- `keyword_weight`: 0.3 (30%)

**Why weighted sum, not multiplication?**
- Weighted sum allows both signals to contribute independently
- Multiplication would penalize categories where one score is low
- Example: Perfect keyword match (1.0) but low fuzzy (0.3) → sum = 0.91, product = 0.30

Weights are configurable via environment variables:
- `MATCH_FUZZY_WEIGHT`
- `MATCH_KEYWORD_WEIGHT`

Must sum to 1.0 (validated on startup).

### Stage 5: Threshold Filter

After scoring all categories, the **best match** is selected. If its score is below the threshold (default: 0.4), return an error:

```
BelowThreshold {
  threshold: 0.4,
  closest_slug: "rust-learning",
  closest_score: 0.35,
  all_slugs: ["bitcoin-node-setup", "rust-learning", ...]
}
```

This error provides:
- The closest match (even though it didn't meet threshold)
- The actual score achieved
- All available categories (for user reference)

**Threshold is also configurable:**
- Via `MATCH_THRESHOLD` environment variable
- Per-query override in `get_sources` MCP tool (optional `threshold` parameter)

### Implementation References

- **Normalization:** `src/matcher/normalize.rs`
- **Scoring logic:** `src/matcher/scorer.rs`
- **Configuration:** `src/matcher/config.rs`
- **Tests:** See `scorer.rs` test suite for real examples against seed data

## Bias Acknowledgment

The current registry reflects the curator's domain expertise:

- **Security** - Threat modeling, Linux hardening, password management
- **Bitcoin** - Node setup, self-sovereignty, cryptography
- **Maker/Self-Hosting** - Email servers, home automation, infrastructure
- **Development** - Rust, MCP, Nostr, Pubky protocols

**What's missing:**
- Frontend web development (React, Vue, Angular)
- Data science and machine learning
- Mobile development (iOS, Android)
- Game development
- Enterprise Java, .NET, etc.

This bias is intentional in v1: curate what you know deeply. Shallow curation of unfamiliar topics would violate the "authoritative" criterion.

**Expanding coverage:**
- Community contributions can add new domains
- Multiple curators (future: federated registries) provide different perspectives
- Endorsement system (future) allows curators to vouch for each other's expertise

## Adding New Categories

To add a new category to the registry:

1. **Choose a descriptive slug:** Lowercase, hyphen-separated, reflects the topic clearly
   - Good: `docker-compose-setup`, `graphql-api-design`
   - Bad: `misc`, `tools`, `dev` (too vague)

2. **Write at least 3 query patterns:** Natural language variations of how users might ask
   - Think: "How would an agent phrase this request?"
   - Example: "docker compose tutorial", "learn docker-compose", "multi-container docker setup"

3. **Curate exactly 3 sources:**
   - Apply the five criteria (authoritative, current, practical, accessible, diverse)
   - Follow the ranking methodology (official → practical → alternative)
   - Write a substantive `why` for each source (not just "good tutorial")

4. **Add to registry.json:**
   ```json
   "your-new-category": {
     "name": "Your Category Name",
     "description": "Detailed description of topic scope and key concepts.",
     "query_patterns": ["pattern one", "pattern two", "pattern three"],
     "sources": [/* your 3 sources */]
   }
   ```

5. **Validate:** Run the server (`cargo run`) - it validates the registry on startup. If validation fails, you'll get a descriptive error with line numbers.

6. **Test:** Query the category via MCP to verify matching works as expected.

## Community Contribution Path

### Suggesting New Categories

Open an issue with:
- Proposed category slug and name
- Brief justification for why this topic deserves coverage
- 1-2 example sources you'd recommend

Maintainer will evaluate fit and prioritize for inclusion.

### Proposing Source Updates

Open an issue or PR with:
- Category and source being updated
- Reason for change (link rot, better alternative, etc.)
- Replacement source (if applicable) with justification

**Review criteria:**
- Does the new source meet the five criteria?
- Is the ranking still correct with the change?
- Are we solving a real problem (broken link, outdated info) or just bike-shedding?

### Maintenance Cadence

Sources are reviewed quarterly for:
- **Link rot:** Dead links, moved content
- **Currency:** Outdated information, abandoned projects
- **Better alternatives:** New authoritative resources that supersede existing ones

Major registry updates are versioned (e.g., v0.1.0 → v0.2.0) and announced.
