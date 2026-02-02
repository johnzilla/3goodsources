# Three Good Sources (3GS)

## What This Is

An MCP server in Rust that serves as a curated trust registry for AI agents. When queried, it returns three vetted sources for a given topic, along with cryptographic provenance via Pubky for verification. Includes a static landing page at 3gs.ai explaining the project and how to connect.

## Core Value

Agents get curated, high-quality sources instead of SEO-gamed search results — three good sources per topic, human-vetted, cryptographically signed, served via open protocol.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] MCP server responds to JSON-RPC over HTTP POST at `/mcp`
- [ ] Four MCP tools: `get_sources`, `list_categories`, `get_provenance`, `get_endorsements`
- [ ] Registry JSON schema with intent patterns, ranked sources, curator identity, and endorsements
- [ ] Fuzzy query matching (normalized Levenshtein + keyword boosting) against category query_patterns
- [ ] 10 seed categories with 3 real, researched sources each
- [ ] PKARR keypair generation/loading for curator identity
- [ ] Pubky SDK integration for registry storage and trust graph (with local fallback if SDK immature)
- [ ] Local `registry.json` fallback when Pubky homeserver unreachable
- [ ] `/health` endpoint returning version and pubkey
- [ ] `/registry` endpoint returning raw registry JSON
- [ ] Static landing page explaining 3GS, how to connect, how to verify
- [ ] Docker multi-stage build (rust:1.84-slim builder, debian:bookworm-slim runtime)
- [ ] Render deployment via render.yaml
- [ ] Documentation: README.md, SCHEMA.md, METHODOLOGY.md, PUBKY.md

### Out of Scope

- stdio/SSE MCP transport — HTTP POST only for MVP, add transports later
- Agent feedback loop — future feature, not MVP
- Community voting on sources — requires Nostr/Pubky event infrastructure
- Routstr integration — future pricing transparency layer
- Automated compliance checking — future feature
- Multiple federated registries with trust graph traversal — future
- Domain-specific forks (3gs-woodworking, etc.) — future
- Mobile app or rich frontend — landing page is static only

## Context

- **Protocol**: MCP (Model Context Protocol) defines the interface. 3GS implements an HTTP POST JSON-RPC server, not the standard stdio transport. Agents call it as an API.
- **Identity layer**: Pubky provides decentralized identity via PKARR keypairs, homeserver storage for the registry, and a trust graph for curator endorsements. SDK maturity is unknown — architecture must gracefully fall back to local registry with manual signing.
- **The insight**: Traditional search is gamed by SEO. Traditional curation doesn't scale. 3GS maps intent patterns (how humans actually ask questions) to curated answers (three sources, human-vetted), served via open protocol with cryptographic provenance. The JSON schema is the protocol. The MCP server is a reference implementation.
- **Domains**: threegoodsources.com, 3goodsources.com, 3gs.ai — all acquired.
- **Curator**: John Turner. Domains of expertise: security, bitcoin, maker, self-hosting.
- **Source types**: documentation, tutorial, video, article, tool, repo, forum, book, course, api.
- **Seed categories**: bitcoin-node-setup, self-hosted-email, rust-learning, home-automation-private, password-management, linux-hardening, threat-modeling, nostr-development, pubky-development, mcp-development.

## Constraints

- **Language**: Rust — non-negotiable, this is a Rust project
- **Framework**: axum 0.8 for HTTP, tokio for async runtime
- **Deployment**: Render free tier via Docker
- **MCP Transport**: HTTP POST only (no stdio, no SSE for MVP)
- **Sources per category**: Always exactly 3 — quality over quantity, this is a hard rule
- **Pubky dependency**: SDK may be immature; must research current state and build fallback-first if needed

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| HTTP POST for MCP transport | Standalone API for any agent to call over HTTP, not tied to Claude Desktop | — Pending |
| Fallback-first Pubky architecture | SDK maturity unknown; local registry.json ensures MVP works regardless | — Pending |
| Research real sources for seed data | Curated quality matters; AI-researched sources validated against real URLs | — Pending |
| Static landing page in MVP | Domains are acquired, need a presence explaining what 3GS is | — Pending |
| axum 0.8 + tokio | Standard Rust async web stack, well-supported | — Pending |

---
*Last updated: 2026-02-01 after initialization*
