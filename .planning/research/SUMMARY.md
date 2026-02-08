# Project Research Summary

**Project:** Three Good Sources (3GS) — v1.1 DigitalOcean Migration + Tech Debt
**Domain:** Platform migration (Render → DigitalOcean App Platform)
**Researched:** 2026-02-08
**Confidence:** HIGH

## Executive Summary

Migrating 3GS from Render to DigitalOcean App Platform is primarily an infrastructure configuration change with minimal code modifications. The existing Docker multi-stage build (cargo-chef, rust:1.85, debian:bookworm-slim) is fully compatible with DO App Platform. Migration requires: (1) creating a DO app spec YAML replacing render.yaml, (2) provisioning via doctl CLI or REST API, (3) DNS cutover for live domains, and (4) tech debt cleanup.

The migration is low-medium complexity. Biggest risks are DNS cutover timing and CORS misconfiguration. Recommended approach: fix tech debt first (validates on existing Render), then migrate infrastructure, then DNS cutover last.

DO App Platform costs ~$5/mo (vs Render ~$7/mo) and adds automatic rollback, API-driven deployment, and encrypted secrets.

## Key Findings

### Stack Additions

**No new Rust dependencies needed.** Only CLI tooling for deployment automation.

- **doctl CLI** (v1.146.0+): Official DO CLI for app provisioning, DNS management, deployment monitoring
- **App spec YAML** (`.do/app.yaml`): Declarative config replacing render.yaml
- **DO REST API v2** (`https://api.digitalocean.com/v2`): Bearer token auth, key endpoints: `/v2/apps`, `/v2/domains`

**Dependency fixes (tech debt):**
- **curve25519-dalek**: v4.1.3 released. Test removing `[patch.crates-io]` — may resolve pkarr compatibility. MUST verify by building locally.
- **tower-http CORS**: Already at 0.6. Change `CorsLayer::permissive()` to `CorsLayer::new().allow_origin(...)` with specific origins.
- **McpError**: Dead code removal, no dependency changes needed.

### Expected Features

**Table stakes (must have for go-live):**
- Docker deployment on DO App Platform
- App spec YAML (convert render.yaml)
- Environment variables (PORT, RUST_LOG, REGISTRY_PATH, PKARR_SECRET_KEY as encrypted SECRET)
- Health checks (/health endpoint in app spec)
- Custom domains (3gs.ai, api.3gs.ai)
- Auto-SSL (Let's Encrypt, automatic)
- Auto-deploy from GitHub on push to main

**Differentiators (DO advantages):**
- API-driven deployment (doctl + app spec = infrastructure as code)
- Automatic rollback on failed deployments
- Encrypted secrets (type: SECRET hides from logs)
- $5/mo vs $7/mo

**Anti-features (avoid):**
- DO Container Registry (unnecessary — builds from Dockerfile)
- Multi-region, premature scaling, platform-level CORS
- Custom buildpacks (Dockerfile works fine)

### Architecture Approach

Migration is infrastructure-only. Existing Docker build fully compatible.

**App spec YAML** replaces render.yaml:
```yaml
name: three-good-sources
region: nyc
services:
  - name: api
    dockerfile_path: Dockerfile
    github:
      repo: johnzilla/3goodsources
      branch: main
      deploy_on_push: true
    http_port: 3000
    health_check:
      http_path: /health
      period_seconds: 10
      timeout_seconds: 5
      failure_threshold: 3
    envs:
      - key: PKARR_SECRET_KEY
        type: SECRET
      - key: REGISTRY_PATH
        value: /app/registry.json
      - key: RUST_LOG
        value: info
    instance_size_slug: basic-xxs
    instance_count: 1
domains:
  - domain: 3gs.ai
    type: PRIMARY
  - domain: api.3gs.ai
    type: ALIAS
```

**Region:** `nyc` (New York) — closest to Michigan. Render's `ohio` not available on DO.

**CORS tightening:**
```rust
let cors = CorsLayer::new()
    .allow_origin(AllowOrigin::list([
        "https://3gs.ai".parse().unwrap(),
        "https://api.3gs.ai".parse().unwrap(),
    ]))
    .allow_methods([Method::GET, Method::POST])
    .allow_headers([header::CONTENT_TYPE]);
```

### Critical Pitfalls

1. **Mixing migration with refactoring** — Fix code first, migrate second. One risky change per deployment.
2. **DNS propagation delays** — Lower TTL to 300s 24 hours before cutover. Both platforms serve during propagation.
3. **CORS misconfiguration** — MCP agents call POST /mcp cross-origin. Overly strict CORS blocks them. Test with real agent before go-live.
4. **curve25519-dalek removal risk** — Build may break if pkarr still needs patch. Test locally first.
5. **Apex domain CNAME** — Some DNS providers can't CNAME apex domains. May need A/AAAA or ALIAS.
6. **Environment variable mapping** — PKARR_SECRET_KEY must be manually copied from Render to DO.
7. **BuildKit cache mounts** — DO may not support `--mount=type=cache`. Test Dockerfile builds on DO.

## Implications for Roadmap

### Phase 8: Tech Debt Cleanup
**Rationale:** Fix code issues on existing Render first — validates changes don't break anything
**Delivers:** Clean dependency tree, secure CORS, no dead code
**Addresses:** curve25519-dalek patch, CORS hardening, McpError removal
**Avoids:** Mixing migration risk with refactoring

### Phase 9: DO App Platform Provisioning
**Rationale:** Create DO deployment alongside Render (parallel running)
**Delivers:** Working DO deployment at `{app}.ondigitalocean.app`
**Addresses:** App spec creation, DO API provisioning, env var config, health checks
**Avoids:** DNS changes (test on DO subdomain first)

### Phase 10: DNS Cutover & Render Decommission
**Rationale:** Once DO deployment verified, switch live traffic
**Delivers:** 3gs.ai and api.3gs.ai served from DigitalOcean
**Addresses:** DNS CNAME updates, SSL verification, render.yaml removal, Render cleanup
**Avoids:** Extended dual-platform costs

### Phase Ordering Rationale

- Tech debt first: code changes deploy to Render, validating they work
- Provisioning second: creates DO app in parallel, no risk to live traffic
- DNS last: only after DO deployment verified stable
- Keep Render alive during entire process as rollback target

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | No new Rust deps. doctl well-documented. |
| Features | HIGH | DO App Platform verified against official docs. |
| Architecture | HIGH | Migration is infrastructure-only. Docker compatible. |
| Pitfalls | MEDIUM-HIGH | DNS/CORS pitfalls well-known. curve25519 fix needs testing. |

**Overall confidence:** HIGH

### Gaps to Address

- **curve25519-dalek**: Must test build after removing patch — can't verify from research
- **DNS provider**: Confirm where 3gs.ai DNS is managed for cutover instructions
- **BuildKit cache**: Verify DO supports `--mount=type=cache` in Dockerfile
- **GitHub repo format**: Confirm app spec `github.repo` field format

## Sources

### Primary (HIGH confidence)
- [DO App Platform App Spec Reference](https://docs.digitalocean.com/products/app-platform/reference/app-spec/)
- [DO Docker Deployment](https://docs.digitalocean.com/products/app-platform/how-to/deploy-from-container-images/)
- [DO Dockerfile Reference](https://docs.digitalocean.com/products/app-platform/reference/dockerfile/)
- [DO Environment Variables](https://docs.digitalocean.com/products/app-platform/how-to/use-environment-variables/)
- [DO Custom Domains](https://docs.digitalocean.com/products/app-platform/how-to/manage-domains/)
- [DO Health Checks](https://docs.digitalocean.com/products/app-platform/how-to/manage-health-checks/)
- [doctl CLI Reference](https://docs.digitalocean.com/reference/doctl/reference/apps/)
- [DO API v2](https://docs.digitalocean.com/reference/api/)
- [tower_http CORS docs](https://docs.rs/tower-http/latest/tower_http/cors/index.html)
- [DO App Platform Pricing](https://docs.digitalocean.com/products/app-platform/details/pricing/)

---
*Research completed: 2026-02-08*
*Ready for roadmap: yes*
