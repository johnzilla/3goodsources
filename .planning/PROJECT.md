# Three Good Sources (3GS)

## What This Is

An MCP server in Rust that serves as a curated trust registry for AI agents. When queried via HTTP POST JSON-RPC, it returns three vetted sources for a given topic, with fuzzy query matching and cryptographic curator identity via PKARR. Includes a static landing page at 3gs.ai and deployment infrastructure for Render.

## Core Value

Agents get curated, high-quality sources instead of SEO-gamed search results — three good sources per topic, human-vetted, cryptographically signed, served via open protocol.

## Requirements

### Validated

- MCP server responds to JSON-RPC 2.0 over HTTP POST at `/mcp` — v1
- Four MCP tools: `get_sources`, `list_categories`, `get_provenance`, `get_endorsements` — v1
- Registry JSON schema with intent patterns, ranked sources, curator identity, and endorsements — v1
- Fuzzy query matching (normalized Levenshtein + keyword boosting) against category query_patterns — v1
- 10 seed categories with 3 real, researched sources each — v1
- PKARR keypair generation/loading for curator identity — v1
- Local `registry.json` as primary data store with in-memory loading on startup — v1
- `/health` endpoint returning version and pubkey — v1
- `/registry` endpoint returning raw registry JSON — v1
- Static landing page at 3gs.ai explaining 3GS, how to connect, how to verify — v1
- Docker multi-stage build (rust:1.85 builder, debian:bookworm-slim runtime) — v1
- Render deployment via render.yaml (starter plan) — v1
- Documentation: README.md, SCHEMA.md, METHODOLOGY.md, PUBKY.md — v1
- 72 tests (43 unit + 29 integration) with full E2E coverage — v1

### Active

## Current Milestone: v1.1 Migrate to DigitalOcean + Tech Debt

**Goal:** Migrate live deployment from Render to DigitalOcean App Platform via DO API, clean up v1 tech debt (CORS, dependency patch, dead code).

**Target features:**
- DigitalOcean App Platform deployment via DO API (replace Render)
- Live migration with DNS cutover for 3gs.ai and api.3gs.ai
- Production-grade CORS configuration (replace permissive)
- Fix curve25519-dalek fragile git patch dependency
- Remove McpError dead code

### Out of Scope

- stdio/SSE MCP transport — HTTP POST only for MVP, add transports later
- Agent feedback loop — future feature, not MVP
- Community voting on sources — requires Nostr/Pubky event infrastructure
- Routstr integration — future pricing transparency layer
- Automated compliance checking — future feature
- Multiple federated registries with trust graph traversal — future
- Domain-specific forks (3gs-woodworking, etc.) — future
- Mobile app or rich frontend — landing page is static only
- Pubky SDK integration for registry storage and trust graph — SDK immature, local-first for now

## Context

- **Protocol**: MCP (Model Context Protocol) defines the interface. 3GS implements an HTTP POST JSON-RPC server. Agents call it as an API.
- **Identity layer**: PKARR keypairs provide cryptographic curator identity. Pubky homeserver integration and trust graph deferred to v2 (SDK immature). Local registry.json is the primary data store.
- **The insight**: Traditional search is gamed by SEO. Traditional curation doesn't scale. 3GS maps intent patterns (how humans actually ask questions) to curated answers (three sources, human-vetted), served via open protocol with cryptographic provenance.
- **Domains**: threegoodsources.com, 3goodsources.com, 3gs.ai — all acquired. Landing page live at 3gs.ai.
- **Curator**: John Turner. Domains of expertise: security, bitcoin, maker, self-hosting.
- **Source types**: documentation, tutorial, video, article, tool, repo, forum, book, course, api.
- **Seed categories**: bitcoin-node-setup, self-hosted-email, rust-learning, home-automation-private, password-management, linux-hardening, threat-modeling, nostr-development, pubky-development, mcp-development.
- **Current state**: v1 shipped and live. 3,016 lines of Rust. 72 tests passing. 10 categories, 30 sources. Live at 3gs.ai (landing) and api.3gs.ai (API), currently on Render.

## Constraints

- **Language**: Rust — non-negotiable, this is a Rust project
- **Framework**: axum 0.8 for HTTP, tokio for async runtime
- **Deployment**: DigitalOcean App Platform via Docker (migrating from Render)
- **MCP Transport**: HTTP POST only (no stdio, no SSE for MVP)
- **Sources per category**: Always exactly 3 — quality over quantity, this is a hard rule
- **Rust version**: 1.85+ required for edition 2024

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| HTTP POST for MCP transport | Standalone API for any agent to call over HTTP, not tied to Claude Desktop | ✓ Good — works for all agents |
| Local-first architecture (no Pubky SDK) | SDK maturity unknown; local registry.json ensures MVP works regardless | ✓ Good — shipped without blocking dependency |
| Research real sources for seed data | Curated quality matters; sources validated against real URLs | ✓ Good — 30 real sources curated |
| Static landing page in MVP | Domains acquired, need presence explaining what 3GS is | ✓ Good — live at 3gs.ai |
| axum 0.8 + tokio | Standard Rust async web stack, well-supported | ✓ Good — clean implementation |
| Weighted sum for score combination | (fuzzy_weight * fuzzy) + (keyword_weight * keyword) allows independent tuning | ✓ Good — reliable matching |
| pkarr with curve25519-dalek git patch | Pre-release dependency issue; fragile but functional | ⚠️ Revisit — should track upstream fix |
| debian:bookworm-slim over Alpine | 133MB vs ~25MB but better glibc compatibility | ✓ Good — stability over size |
| Plain text MCP tool responses | Better agent parsing than markdown | ✓ Good — works well |
| Permissive CORS for MVP | CorsLayer::permissive() allows all origins | ⚠️ Revisit — tighten for production |

| Migrate from Render to DigitalOcean App Platform | Consolidate infrastructure — already running another project on DO | — Pending |

---
*Last updated: 2026-02-08 after v1.1 milestone start*
