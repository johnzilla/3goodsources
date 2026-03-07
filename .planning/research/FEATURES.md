# Feature Landscape

**Domain:** Curated trust registry with audit trail, identity linking, and community contributions
**Researched:** 2026-03-07
**Scope:** v2.0 milestone features ONLY (audit log, identity linking, community contributions)

## Table Stakes

Features users expect for v2.0. Missing = the milestone feels incomplete.

### Audit Log

| Feature | Why Expected | Complexity | Dependencies | Notes |
|---------|--------------|------------|--------------|-------|
| Append-only `audit_log.json` file | Core transparency promise -- without it, curation changes are opaque | Low | None (new file) | Curator appends entries manually or via CLI; server loads on startup like `registry.json` |
| Each entry has timestamp, action, actor pubkey, and affected entity | Standard audit schema -- who did what, when, to what | Low | PKARR pubkey (exists) | Actions: `source_added`, `source_removed`, `source_updated`, `category_added`, `category_removed`, `endorsement_added` |
| `GET /audit` endpoint returning full log | Users/agents need to read the log | Low | Audit log loaded in AppState | Return raw JSON array, newest-first |
| Filter `/audit` by action type and category | Unfiltered logs are useless at scale | Medium | `/audit` endpoint | Query params: `?action=source_added&category=rust-learning` |
| `get_audit_log` MCP tool | Agents need programmatic access to curation history | Low | Audit log in AppState | Mirror the `/audit` endpoint filtering via tool arguments |
| Entry includes human-readable `description` field | Audit entries need context, not just action codes | Low | None | e.g., "Added 'Zero To Production' as rank 3 source for rust-learning" |

### Identity Linking

| Feature | Why Expected | Complexity | Dependencies | Notes |
|---------|--------------|------------|--------------|-------|
| `identities.json` mapping PKARR pubkeys to platform claims | Core identity linking data store | Low | None (new file) | Curator-managed; each identity has pubkey + array of platform claims |
| Platform claims for GitHub, X (Twitter), and Nostr | These are the three platforms in the project's ecosystem | Medium | None | Follow NIP-39 model: platform + identity + proof URL |
| `GET /identities` endpoint listing all linked identities | Public discoverability of identity claims | Low | Identities loaded in AppState | Returns array of identity objects |
| `GET /identities/:pubkey` endpoint for single identity lookup | Direct lookup by pubkey | Low | `/identities` endpoint | 404 if pubkey not found |
| `get_identity` MCP tool | Agents need to verify curator identity across platforms | Low | Identities in AppState | Takes pubkey argument, returns linked identities |
| Proof URLs pointing to verifiable public posts | Without proof, claims are meaningless -- anyone can claim any handle | Low | Platform accounts | GitHub: Gist with pubkey. X: Tweet with pubkey. Nostr: kind 0 event with pubkey |

### Community Contributions

| Feature | Why Expected | Complexity | Dependencies | Notes |
|---------|--------------|------------|--------------|-------|
| `contributions.json` file for community proposals | Structured storage for source suggestions | Low | None (new file) | Curator-managed; proposals submitted via GitHub Issues, curator adds to JSON |
| Proposal schema: category, proposed source, submitter info, status | Users need to know what's proposed and its current state | Low | None | Statuses: `pending`, `accepted`, `rejected`, `deferred` |
| `GET /proposals` endpoint listing proposals | Public visibility of community input | Low | Contributions loaded in AppState | Filterable by status and category |
| `GET /proposals/:id` endpoint for single proposal | Direct proposal lookup | Low | `/proposals` endpoint | |
| `list_proposals` and `get_proposal` MCP tools | Agents should surface community proposals | Low | Contributions in AppState | |
| Curator rationale on accepted/rejected proposals | Transparency about why decisions were made | Low | None | `curator_notes` field on each proposal |

## Differentiators

Features that set 3GS apart. Not expected, but valuable.

| Feature | Value Proposition | Complexity | Dependencies | Notes |
|---------|-------------------|------------|--------------|-------|
| **Signed audit entries** | Each audit entry signed with curator's Ed25519 key, proving the curator actually made the change -- not just claimed to | High | PKARR keypair, JSON canonicalization | Requires deterministic JSON serialization (sorted keys). Huge trust differentiator but adds complexity. Defer to v2.1 unless time permits |
| **Audit entry hash chain** | Each entry includes hash of previous entry, creating tamper-evident chain | Medium | Audit log entries | Like a mini blockchain for curation decisions. SHA-256 of previous entry's canonical JSON. Detects if someone edits history |
| **Human vs bot vote signal separation** | Community proposals track whether support comes from humans (GitHub accounts with history) vs fresh/bot accounts | High | Contributions system, heuristics | No perfect solution exists. Practical approach: record GitHub account age, contribution count, and let consumers decide. Do NOT try to build Sybil resistance -- just expose signals |
| **Cross-platform identity proof verification** | Server validates proof URLs on startup or on-demand, marking claims as `verified` or `unverified` | High | Identity claims, HTTP client | Requires fetching external URLs (GitHub Gists, tweets). Fragile -- platforms change APIs, tweets get deleted. Better as optional enrichment, not hard requirement |
| **Proposal voting/endorsement counts** | Track how many community members support a proposal | Medium | Contributions system | Read-only: curator tallies votes from GitHub Issue reactions and records count in JSON. No live voting on server |
| **Audit log RSS/Atom feed** | `/audit/feed` endpoint for subscribing to curation changes | Low | Audit log | Nice for transparency watchers. Low effort, high signal |

## Anti-Features

Features to explicitly NOT build for v2.0.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **Write API for submissions** | Server is read-only by design. Write APIs introduce auth, rate limiting, spam, input validation complexity. The curator-managed JSON model is the core architectural decision | Accept proposals via GitHub Issues. Curator manually adds to `contributions.json`. Link to issue in proposal |
| **OAuth-based identity verification** | Requires registering OAuth apps with every platform, managing tokens, handling revocations. Massive complexity for marginal benefit | Use Keybase/NIP-39 model: user posts proof on their platform, curator records the URL. Verification is out-of-band |
| **Live voting system** | Real-time voting requires auth, Sybil resistance, rate limiting, database writes -- all antithetical to the read-only JSON architecture | Record vote counts from GitHub Issue reactions in the JSON. Curator updates periodically |
| **Automated identity verification on server** | Fetching and parsing external platform pages is fragile, breaks when platforms change HTML/APIs, and adds runtime dependencies | Mark identity claims as `claimed` with proof URLs. Let consumers verify independently. Optionally verify during curator review |
| **User accounts or authentication** | No user accounts. The server serves curated data. Period | Community interaction happens on GitHub (Issues for proposals, Gists for identity proofs) |
| **Nostr relay or event infrastructure** | Running a Nostr relay is an entirely separate system. 3GS is an MCP server, not a Nostr relay | Reference Nostr events by ID. Link to Nostr event URLs. Don't process or verify Nostr events on server |
| **Weighted or quadratic voting** | Sybil resistance in voting is an unsolved research problem. Bond voting, quadratic voting, proof-of-personhood all require infrastructure 3GS does not have | Simple counts from GitHub reactions. Label as "signal, not governance." Curator makes final decisions |

## Feature Dependencies

```
registry.json (exists) --> audit_log.json (logs changes to registry)
                       --> contributions.json (proposals reference categories/sources)

PKARR keypair (exists) --> identities.json (maps pubkeys to platform claims)
                       --> audit_log.json (actor field uses pubkey)

audit_log.json --> GET /audit endpoint --> get_audit_log MCP tool
identities.json --> GET /identities endpoint --> get_identity MCP tool
contributions.json --> GET /proposals endpoint --> list_proposals + get_proposal MCP tools

All three new JSON files follow the same pattern as registry.json:
  1. Curator edits file locally
  2. File committed to git
  3. Server loads on startup via include_str! or file read
  4. Stored in Arc<T> in AppState
  5. Served read-only via HTTP + MCP
```

## Proposed Schemas

### audit_log.json

```json
{
  "version": "1.0.0",
  "entries": [
    {
      "id": "2026-03-07-001",
      "timestamp": "2026-03-07T14:30:00Z",
      "action": "source_added",
      "actor": "o4dksfbqk85ogzdb5osziw6befigbuxmuxkuxq8434q89uj56uyy",
      "category": "rust-learning",
      "target": "Zero To Production In Rust",
      "description": "Added 'Zero To Production' as rank 3 source for rust-learning",
      "previous_hash": null
    }
  ]
}
```

**Action enum:** `source_added`, `source_removed`, `source_updated`, `source_reranked`, `category_added`, `category_removed`, `category_updated`, `endorsement_added`, `endorsement_removed`, `identity_linked`, `identity_unlinked`, `proposal_accepted`, `proposal_rejected`

**Design rationale:** The `id` field uses date-based sequential IDs (not UUIDs) because the curator manages this file manually. The `previous_hash` field is optional for v2.0 (hash chain is a differentiator, not table stakes). The `actor` field is always the curator's PKARR pubkey since only the curator can modify these files.

### identities.json

```json
{
  "version": "1.0.0",
  "identities": [
    {
      "pubkey": "o4dksfbqk85ogzdb5osziw6befigbuxmuxkuxq8434q89uj56uyy",
      "display_name": "John Turner",
      "claims": [
        {
          "platform": "github",
          "identity": "johnturner",
          "proof": "https://gist.github.com/johnturner/abc123",
          "claimed_at": "2026-03-07T14:30:00Z"
        },
        {
          "platform": "x",
          "identity": "johnturner",
          "proof": "https://x.com/johnturner/status/123456789",
          "claimed_at": "2026-03-07T14:30:00Z"
        },
        {
          "platform": "nostr",
          "identity": "npub1abc123...",
          "proof": "note1xyz789...",
          "claimed_at": "2026-03-07T14:30:00Z"
        }
      ]
    }
  ]
}
```

**Platform claim model** follows the NIP-39 / Keybase pattern:
- Platform name (normalized lowercase)
- Identity on that platform (username, npub, etc.)
- Proof URL or event ID pointing to a public post containing the PKARR pubkey
- Verification is out-of-band: anyone can follow the proof URL and check

**Supported platforms for v2.0:** `github` (Gist proof), `x` (Tweet proof), `nostr` (kind 0 profile event or signed note). Additional platforms can be added later by extending the platform enum.

### contributions.json

```json
{
  "version": "1.0.0",
  "proposals": [
    {
      "id": "prop-001",
      "submitted_at": "2026-03-07T14:30:00Z",
      "submitter": {
        "name": "community_member",
        "github": "https://github.com/community_member",
        "account_created": "2020-01-15",
        "public_repos": 42
      },
      "category": "rust-learning",
      "proposed_source": {
        "name": "Rustlings",
        "url": "https://github.com/rust-lang/rustlings",
        "type": "tutorial",
        "why": "Interactive exercises for learning Rust syntax and concepts"
      },
      "action": "add_source",
      "status": "pending",
      "github_issue": "https://github.com/3goodsources/3goodsources/issues/42",
      "community_signals": {
        "github_reactions": 12,
        "unique_commenters": 5
      },
      "curator_notes": null,
      "resolved_at": null
    }
  ]
}
```

**Action types:** `add_source` (propose new source for category), `replace_source` (propose replacing existing source), `new_category` (propose entirely new category), `update_source` (fix URL, update description)

**Submitter metadata** serves as a soft Sybil signal: GitHub account age and repo count give consumers (and the curator) context about whether the submitter is a real person with history or a fresh throwaway account. This is NOT automated verification -- just recorded data points.

**Status lifecycle:** `pending` -> `accepted` | `rejected` | `deferred`. Once resolved, `curator_notes` explains the decision and `resolved_at` gets a timestamp.

## MVP Recommendation

Prioritize (build in this order due to dependencies):
1. **Audit log** -- `audit_log.json` + `/audit` endpoint + `get_audit_log` MCP tool. Lowest risk, highest transparency value. Backfill initial entries for existing 10 categories and 30 sources
2. **Identity linking** -- `identities.json` + `/identities` endpoints + `get_identity` MCP tool. Curator creates proof posts on GitHub/X/Nostr, records URLs. Immediate credibility boost
3. **Community contributions** -- `contributions.json` + `/proposals` endpoints + MCP tools. Depends on having a GitHub Issues template for submissions

Defer to v2.1:
- **Signed audit entries**: Requires JSON canonicalization library, adds build complexity. The hash chain alone provides tamper evidence
- **Cross-platform proof verification**: Fragile external HTTP calls. Let consumers verify manually for now
- **Vote signal separation**: Record basic GitHub reaction counts first. Sophisticated human/bot separation is premature

## Complexity Assessment

| Feature Area | New Files | New Endpoints | New MCP Tools | Estimated Effort |
|-------------|-----------|---------------|---------------|-----------------|
| Audit Log | 1 (`audit_log.json`) | 1 (`/audit`) | 1 (`get_audit_log`) | Low -- follows exact same pattern as `registry.json` |
| Identity Linking | 1 (`identities.json`) | 2 (`/identities`, `/identities/:pubkey`) | 1 (`get_identity`) | Low-Medium -- new route pattern with path params |
| Community Contributions | 1 (`contributions.json`) | 2 (`/proposals`, `/proposals/:id`) | 2 (`list_proposals`, `get_proposal`) | Low-Medium -- most complexity is in schema design, not code |

**Total new surface area:** 3 JSON files, 5 HTTP endpoints, 4 MCP tools. All read-only. All following the established pattern of "load JSON on startup, serve via Arc in AppState."

The existing `AppState` struct adds three new fields. The existing `build_router` function adds five new routes. The existing `McpHandler` registers four new tools. No architectural changes required -- this is additive.

## Sources

- [NIP-39: External Identities in Profiles](https://github.com/nostr-protocol/nips/blob/master/39.md) -- identity proof model for cross-platform claims (HIGH confidence)
- [Keybase Proof System](https://keybase.io/blog/keybase-proofs-for-mastodon-and-everyone) -- prior art for cross-platform identity verification architecture (HIGH confidence)
- [Martin Fowler: Audit Log Pattern](https://martinfowler.com/eaaDev/AuditLog.html) -- canonical audit log design pattern (HIGH confidence)
- [Mattermost Audit Log JSON Schema](https://docs.mattermost.com/comply/embedded-json-audit-log-schema.html) -- real-world audit log schema reference (MEDIUM confidence)
- [Sybil Attack Resistance in Voting](https://arxiv.org/abs/2407.01844) -- academic context on why automated Sybil resistance is hard and should be deferred (MEDIUM confidence)
- [Append-Only Logs: Design Patterns](https://medium.com/@komalshehzadi/append-only-logs-the-immutable-diary-of-data-58c36a871c7c) -- hash chain and immutability patterns (LOW confidence, blog post)
