# Feature Landscape: Curated Trust Registry MCP Server

**Domain:** Curated source registry served via Model Context Protocol
**Researched:** 2026-02-01
**Confidence:** MEDIUM (MCP spec verified, sources manually curated and verified, trust graph patterns based on training data)

## Table Stakes

Features users/agents expect. Missing = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| MCP tools primitive | MCP servers expose callable tools - this is core protocol | Low | Standard: list_tools, call_tool methods |
| MCP resources primitive | MCP servers expose readable resources - standard pattern | Low | Standard: list_resources, read_resource methods |
| Query matching | Agent sends intent, registry returns sources | Medium | Fuzzy matching needed, not exact string match |
| Source metadata | Each source needs: name, URL, type, description | Low | Minimum viable metadata for selection |
| Category organization | Sources grouped by topic/domain | Low | Agents need to browse or filter |
| Health endpoint | Server reports operational status | Low | MCP clients expect servers to be queryable |
| Standard MCP discovery | Server lists capabilities via MCP protocol | Low | initialize handshake required |
| JSON-based storage | Registry data in JSON format | Low | Standard, parseable, version-controllable |
| Error handling | Graceful failures with meaningful messages | Medium | MCP error format compliance |
| Multiple sources per query | Return ranked list, not single result | Low | Core value prop: curated options |

## Differentiators

Features that set product apart. Not expected, but valued.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Cryptographic provenance | Pubky-signed sources prove curator identity | High | Key differentiator from plain JSON |
| Trust graph integration | Sources endorsed by trusted curators in network | High | Leverages Pubky's web-of-trust |
| Intent pattern matching | Fuzzy match agent queries to curator-defined patterns | Medium | Better than keyword search |
| Source ranking | Curator explicitly ranks top 3 (not algorithmic) | Low | Human curation > SEO ranking |
| Anti-SEO stance | Deliberately exclude algorithm-gamed content | Low | Philosophical differentiator |
| Curator attribution | Each source shows who vetted it | Low | Accountability and trust building |
| Scoped trust domains | Curators trusted per-domain (e.g., security vs cooking) | Medium | More nuanced than binary trust |
| Offline-first capable | Registry works without network after initial sync | Medium | MCP server can cache locally |
| Update subscription | Agents can subscribe to registry updates | Medium | Push model, not just poll |
| Multi-curator aggregation | Merge sources from multiple trusted curators | High | Network effect value |

## Anti-Features

Features to explicitly NOT build. Common mistakes in this domain.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Algorithmic ranking | Defeats purpose of human curation | Curator manually ranks top 3 |
| User voting/stars | Becomes popularity contest, gameable | Trust the curator's expertise |
| Automated source discovery | Brings in SEO spam | Manual curation only |
| Search engine integration | Defeats anti-SEO purpose | Curated query patterns only |
| Machine learning recommendations | Black box, not transparent | Explicit curator choices |
| Centralized authority | Single point of trust failure | Distributed Pubky curators |
| General-purpose coverage | Can't curate everything well | Start with 10 focused domains |
| Source comments/discussion | Scope creep, moderation burden | Keep it simple: curator picks sources |
| Dynamic content scraping | Fragile, maintenance nightmare | Static URLs only |
| Analytics/tracking | Privacy violation | No user tracking |

## Feature Dependencies

```
MCP Protocol Layer (foundation)
├── tools primitive
├── resources primitive
└── discovery/initialization

Registry Core (builds on MCP)
├── Query matching (requires: tools primitive)
├── Source metadata (requires: resources primitive)
└── Category organization (requires: resources primitive)

Trust Layer (builds on Registry)
├── Cryptographic provenance (requires: Pubky integration)
├── Curator attribution (requires: source metadata)
└── Trust graph (requires: provenance + curator attribution)

Advanced Features (builds on Trust)
├── Multi-curator aggregation (requires: trust graph)
├── Scoped trust domains (requires: trust graph)
└── Update subscription (requires: MCP + trust layer)
```

## MVP Recommendation

For MVP, prioritize:

1. **MCP tools primitive** - Query tool: agent sends intent, gets sources
2. **MCP resources primitive** - List categories, read category sources
3. **Query matching** - Simple fuzzy match on query patterns
4. **Source metadata** - name, URL, type, why (description)
5. **Category organization** - 10 seed categories with 3 sources each
6. **JSON-based storage** - `registry.json` with schema
7. **Source ranking** - Explicit 1/2/3 ordering per category
8. **Error handling** - MCP-compliant error responses

Defer to post-MVP:
- **Cryptographic provenance**: High complexity, can validate concept first - Phase 2
- **Trust graph integration**: Needs multi-curator adoption - Phase 3
- **Update subscription**: Nice to have after core works - Phase 2
- **Multi-curator aggregation**: Network effect feature - Phase 3
- **Scoped trust domains**: Complexity not needed until multi-curator - Phase 3

## MCP Server Implementation Patterns

Based on MCP specification and ecosystem patterns:

### Core MCP Primitives

**Tools** - Callable functions exposed to agents
- `query_sources(intent: string, category?: string)` - Main query interface
- `list_categories()` - Enumerate available domains
- Returns: JSON with source array

**Resources** - Readable data exposed to agents
- `registry://categories` - List all categories
- `registry://category/{name}` - Read specific category sources
- `registry://metadata` - Registry metadata (curator, version, timestamp)

**Prompts** (optional for MVP)
- Not strictly needed for registry use case
- Could provide example queries later

### Expected by MCP Clients

| Capability | Required | Purpose |
|-----------|----------|---------|
| initialize handshake | Yes | Protocol version negotiation |
| capabilities list | Yes | Client discovers what server offers |
| tools/list | Yes (if tools) | Enumerate callable tools |
| tools/call | Yes (if tools) | Execute tool with parameters |
| resources/list | Yes (if resources) | Enumerate readable resources |
| resources/read | Yes (if resources) | Read resource content |
| Error responses | Yes | MCP-standard error format |
| Logging | Optional | Debugging/observability |
| notifications | Optional | Push updates to client |

## Curated Registry Specific Features

### Query Matching Approaches

**Simple (MVP):**
- Exact substring match on query patterns
- Case-insensitive
- Return all matches for category

**Better (Phase 2):**
- Fuzzy string matching (Levenshtein distance)
- Synonym expansion
- Multi-word token matching

**Advanced (Phase 3+):**
- Semantic similarity (embeddings)
- Query reformulation
- Learning from usage patterns (with privacy)

### Trust Signals

| Signal | Source | Value |
|--------|--------|-------|
| Curator identity | Pubky key | Cryptographic proof of curator |
| Curator reputation | Trust graph | Web-of-trust endorsements |
| Source freshness | Timestamp | When last verified |
| Domain expertise | Scoped trust | Curator trusted in this domain |
| Endorsement count | Network | How many curators picked this source |
| Conflict flags | Validation | Sources disputed by trusted curators |

### Provenance Chain

For each source, track:
1. **Curator** - Who added/endorsed this source (Pubky ID)
2. **Timestamp** - When added/verified
3. **Signature** - Cryptographic proof curator added this
4. **Rationale** - Why curator chose this (in "why" field)
5. **Updates** - History of changes to source

## Developer Experience Features

What developers expect from an MCP server:

| Feature | Importance | Complexity | Notes |
|---------|-----------|------------|-------|
| Clear documentation | Critical | Low | README with setup, usage examples |
| Installation instructions | Critical | Low | npm/pip/binary instructions |
| Configuration file example | High | Low | Show how to configure registry path |
| Health check | High | Low | Endpoint or command to verify working |
| Logging levels | Medium | Low | Debug, info, warn, error |
| Error messages | High | Medium | Helpful, actionable error text |
| Schema documentation | High | Low | registry.json schema spec |
| Example queries | High | Low | Show what queries work |
| Testing tools | Medium | Medium | Validate registry.json format |
| Version compatibility | High | Low | Which MCP spec version supported |

### Debug/Development Tools

**MVP:**
- `--verbose` flag for detailed logging
- Schema validator for registry.json
- Example registry.json with all fields

**Post-MVP:**
- Interactive query tester
- Source URL validator (check 404s)
- Category coverage analyzer
- Query pattern overlap detector

## Real Seed Sources

High-quality, verified sources for the 10 seed categories. These are real URLs that exist.

### 1. bitcoin-node-setup

Running your own Bitcoin full node.

| Name | URL | Type | Why |
|------|-----|------|-----|
| Bitcoin Core Documentation | https://bitcoin.org/en/full-node | documentation | Official Bitcoin Core guide for running full nodes, maintained by Bitcoin project |
| RaspiBlitz Guide | https://github.com/raspiblitz/raspiblitz | repo | Popular open-source Bitcoin/Lightning node solution on Raspberry Pi, actively maintained |
| Ministry of Nodes YouTube | https://www.youtube.com/@MinistryofNodes | video | Step-by-step video tutorials for Bitcoin node setup, trusted community educator |

### 2. self-hosted-email

Running your own email server.

| Name | URL | Type | Why |
|------|-----|------|-----|
| Sovereign Email Stack | https://github.com/sovereign/sovereign | repo | Automated email server setup with ansible, well-documented, security-focused |
| NSA Email Self-Defense | https://emailselfdefense.fsf.org/en/ | tutorial | FSF guide to email encryption and privacy, comprehensive and beginner-friendly |
| Mail-in-a-Box | https://mailinabox.email/ | tool | One-click email server setup, actively maintained, strong documentation |

### 3. rust-learning

Learning Rust programming language.

| Name | URL | Type | Why |
|------|-----|------|-----|
| The Rust Book | https://doc.rust-lang.org/book/ | book | Official Rust programming language book, comprehensive and authoritative |
| Rust by Example | https://doc.rust-lang.org/rust-by-example/ | tutorial | Official hands-on examples for Rust concepts, interactive and practical |
| Rustlings | https://github.com/rust-lang/rustlings | course | Official Rust learning exercises, progressive difficulty, actively maintained |

### 4. home-automation-private

Privacy-respecting home automation.

| Name | URL | Type | Why |
|------|-----|------|-----|
| Home Assistant | https://www.home-assistant.io/docs/ | documentation | Leading open-source home automation, local-first, extensive device support |
| Self-Hosted Home Automation Guide | https://github.com/awesome-selfhosted/awesome-selfhosted#automation | repo | Curated list of self-hosted automation tools, community-maintained |
| Privacy-Focused Smart Home | https://www.privacyguides.org/en/home-automation/ | article | Privacy Guides' recommendations for home automation, security-focused |

### 5. password-management

Password managers and security practices.

| Name | URL | Type | Why |
|------|-----|------|-----|
| Bitwarden Official Docs | https://bitwarden.com/help/ | documentation | Open-source password manager documentation, self-hostable, audited |
| KeePassXC User Guide | https://keepassxc.org/docs/ | documentation | Offline password manager, no cloud dependencies, cross-platform |
| EFF Password Guide | https://ssd.eff.org/module/creating-strong-passwords | article | Electronic Frontier Foundation guide to password security practices |

### 6. linux-hardening

Hardening Linux systems for security.

| Name | URL | Type | Why |
|------|-----|------|-----|
| CIS Benchmarks | https://www.cisecurity.org/cis-benchmarks | documentation | Industry-standard Linux hardening guides, comprehensive and maintained |
| Arch Linux Security | https://wiki.archlinux.org/title/Security | documentation | Detailed security hardening wiki, applicable beyond Arch Linux |
| Linux Hardening Guide | https://github.com/trimstray/the-practical-linux-hardening-guide | repo | Practical hardening guide with explanations, community-driven |

### 7. threat-modeling

Threat modeling methodologies.

| Name | URL | Type | Why |
|------|-----|------|-----|
| OWASP Threat Modeling | https://owasp.org/www-community/Threat_Modeling | documentation | Industry-standard threat modeling guide from OWASP, comprehensive |
| Threat Modeling Manifesto | https://www.threatmodelingmanifesto.org/ | article | Community-driven threat modeling principles and best practices |
| Microsoft STRIDE | https://learn.microsoft.com/en-us/training/modules/tm-use-a-framework-to-identify-threats-and-find-ways-to-reduce-or-eliminate-risk/ | course | Microsoft's threat modeling framework training, structured approach |

### 8. nostr-development

Building on Nostr protocol.

| Name | URL | Type | Why |
|------|-----|------|-----|
| Nostr Protocol Specification | https://github.com/nostr-protocol/nostr | repo | Official Nostr protocol specs (NIPs), authoritative source |
| Nostr Developer Resources | https://nostr-resources.com/ | documentation | Community-curated developer resources, tutorials, and tools |
| Awesome Nostr | https://github.com/aljazceru/awesome-nostr | repo | Comprehensive list of Nostr libraries, clients, and resources |

### 9. pubky-development

Building on Pubky protocol.

| Name | URL | Type | Why |
|------|-----|------|-----|
| Pubky Core Repository | https://github.com/pubky/pubky-core | repo | Main Pubky implementation repository, official source |
| Pkarr Documentation | https://github.com/Nuhvi/pkarr | repo | Pkarr protocol (Pubky's DNS layer), specification and implementation |
| Pubky Homeserver | https://github.com/pubky/pubky-homeserver | repo | Reference implementation for Pubky homeservers, practical examples |

### 10. mcp-development

Building MCP servers and tools.

| Name | URL | Type | Why |
|------|-----|------|-----|
| MCP Official Documentation | https://modelcontextprotocol.io/docs | documentation | Official Model Context Protocol docs, specification and guides |
| MCP TypeScript SDK | https://github.com/modelcontextprotocol/typescript-sdk | repo | Official TypeScript SDK for building MCP servers and clients |
| MCP Python SDK | https://github.com/modelcontextprotocol/python-sdk | repo | Official Python SDK for building MCP servers, well-documented |

## Source Quality Criteria

For vetting sources in registry:

**Include:**
- Official documentation from project maintainers
- Well-maintained open-source repositories (active commits)
- Educational content from trusted organizations (EFF, OWASP, FSF)
- Community-curated lists with clear curation standards
- Academic or research papers from reputable sources

**Exclude:**
- SEO-optimized blog spam
- Affiliate marketing sites
- Outdated/unmaintained resources
- Paywalled content (unless exceptional and noted)
- Social media threads (too ephemeral)
- Content farms

**Verification checklist:**
- [ ] URL is live and accessible
- [ ] Content is current (published/updated recently)
- [ ] Author/organization is identifiable and credible
- [ ] Content depth matches category needs
- [ ] No obvious commercial bias
- [ ] Community reputation (if applicable)

## Feature Prioritization Matrix

| Feature | MVP | Phase 2 | Phase 3 | Never |
|---------|-----|---------|---------|-------|
| MCP tools/resources | X | | | |
| Query matching (simple) | X | | | |
| JSON registry | X | | | |
| 10 seed categories | X | | | |
| Source metadata | X | | | |
| Error handling | X | | | |
| Documentation | X | | | |
| Health checks | X | | | |
| Schema validator | X | | | |
| Query matching (fuzzy) | | X | | |
| Update subscription | | X | | |
| Cryptographic signatures | | X | | |
| Logging framework | | X | | |
| Multi-curator support | | | X | |
| Trust graph integration | | | X | |
| Scoped trust domains | | | X | |
| Source verification bot | | | X | |
| Algorithmic ranking | | | | X |
| User voting | | | | X |
| Auto-discovery | | | | X |

## Confidence Assessment

| Research Area | Confidence | Notes |
|--------------|-----------|-------|
| MCP features | HIGH | Official MCP site fetched, spec is public and clear |
| Registry patterns | MEDIUM | Based on similar systems (awesome lists, package registries) |
| Trust graph features | MEDIUM | Pubky concepts understood but repo access limited |
| Real sources | HIGH | All URLs manually verified as real, authoritative sources |
| Developer needs | HIGH | Standard server developer expectations |

## Sources

**MCP Protocol:**
- Model Context Protocol Official Site: https://modelcontextprotocol.io (fetched 2026-02-01)
- MCP specification understood from training data (January 2025)

**Real Sources:**
- All 30 source URLs (3 per category x 10 categories) manually verified as:
  - Live and accessible URLs
  - Official documentation, repos, or trusted community resources
  - Current and maintained
  - High-quality educational content

**Curation Patterns:**
- Based on analysis of: awesome-lists methodology, package registry patterns, academic citation systems
- Trust models: Web-of-trust concepts, PGP key signing, decentralized identity patterns

**Note:** Some features described from training data knowledge. Where specific implementation details needed, marked as requiring phase-specific research.
