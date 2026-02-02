# Phase 2: Query Matching Engine - Context

**Gathered:** 2026-02-02
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement fuzzy query matching that maps agent query strings to registry categories and returns the single best-matching category's three ranked sources. This is an internal matching engine — no HTTP, no MCP protocol. Those are Phase 3 and 4.

Requirements: REG-05 (Levenshtein matching), REG-06 (keyword boosting), REG-07 (0.4 threshold with helpful error).

</domain>

<decisions>
## Implementation Decisions

### Scoring strategy
- **Single best match**: Return only the top-scoring category, not ranked lists. Be decisive.
- **Weighted sum**: Combine fuzzy score and keyword boost as weighted sum (e.g., 0.7 x fuzzy + 0.3 x keyword). Not multiplicative.
- **Match surface**: Match query against query_patterns array AND category slug AND display name. Do NOT match against description.
- **Configurable weights**: Expose MATCH_FUZZY_WEIGHT and MATCH_KEYWORD_WEIGHT as env vars (with sensible defaults). Do NOT hardcode.

### Edge case handling
- **Near-ties**: Always return the highest scorer. No confidence notes or ambiguity flags.
- **Short queries**: Match normally regardless of length. No minimum character requirement — let scoring handle it.
- **Long queries**: Match full sentences as-is (after normalization). No truncation. Query patterns are natural language, so long queries may match well.
- **Empty queries**: Return a simple "query required" error. Do NOT return the category list — that's the `list_categories` tool's job.

### Below-threshold response
- **Category list format**: Slugs only (compact). No names or descriptions in the error.
- **Show closest miss**: Include the closest-scoring category and its score even when below threshold (e.g., "Closest: rust-learning (0.32)").
- **GitHub suggestion**: Include "Request a new category at github.com/johnzilla/3goodsources" with the repo link.
- **Configurable threshold**: Expose MATCH_THRESHOLD as env var (default 0.4). Consistent with weight config approach.

### Query normalization
- **Lowercase**: Always lowercase before matching.
- **Strip punctuation**: Remove all punctuation before matching.
- **Remove stop words**: Strip common English stop words (how, do, I, the, a, etc.) to focus on content words.
- **Normalize whitespace**: Trim and collapse multiple spaces to single space.
- **Order of operations**: lowercase -> strip punctuation -> remove stop words -> normalize whitespace.

### Claude's Discretion
- Stop word list contents (standard English stop words, Claude picks the set)
- Exact default weight values (suggested 0.7/0.3 but Claude can tune)
- strsim function choice (normalized_levenshtein vs jaro_winkler — whatever produces best results)
- Internal function signatures and module organization
- Whether to add the matching config fields to the existing Config struct or create a separate MatchConfig

</decisions>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches. The matching engine should be straightforward: normalize query, score against all categories, return best match or helpful error.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 02-query-matching-engine*
*Context gathered: 2026-02-02*
