# Phase 18: Docker Distribution - Context

**Gathered:** 2026-04-03
**Status:** Ready for planning

<domain>
## Phase Boundary

Create a GitHub Actions workflow that builds and publishes a multi-platform Docker image to GHCR. Any curator should be able to `docker pull ghcr.io/johnzilla/3goodsources:latest` and run a 3GS node. This is the distribution mechanism for the federation demand test.

</domain>

<decisions>
## Implementation Decisions

### GitHub Actions Workflow (DIST-03)
- **D-01:** New file: `.github/workflows/docker.yml`
- **D-02:** Triggers: push to `main` branch, tags matching `v*`
- **D-03:** Build multi-platform: `linux/amd64` and `linux/arm64`
- **D-04:** Use `docker/build-push-action` with QEMU for cross-platform builds
- **D-05:** Login to GHCR via `docker/login-action` with `GITHUB_TOKEN`
- **D-06:** Image name: `ghcr.io/johnzilla/3goodsources`

### Tagging Strategy
- **D-07:** Tags: `latest` (on main push), git SHA short hash, semver tag (on `v*` tag push)
- **D-08:** Use `docker/metadata-action` for automated tag generation

### Dockerfile
- **D-09:** Existing Dockerfile at repo root works as-is (multi-stage: cargo-chef, debian:bookworm-slim)
- **D-10:** No Dockerfile modifications needed — it already produces a correct runtime image
- **D-11:** The image copies registry.json as fallback; curators override via volume mount or env var

### Package Visibility
- **D-12:** Set package to public visibility so anyone can pull without authentication

### Claude's Discretion
- Whether to add a `workflow_dispatch` trigger for manual builds
- Whether to add build caching (cargo-chef already handles dep caching in the Dockerfile)
- Whether to add a README badge for the GHCR package

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Dockerfile
- `Dockerfile` — Existing multi-stage build (cargo-chef planner, builder, debian:bookworm-slim runtime)

### Deployment Config
- `.do/app.yaml` — Current DigitalOcean App Platform config (for reference, not modified)

### Eng Review Plan
- `~/.claude/plans/purring-marinating-taco.md` — Step 6 covers Docker/GHCR

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `Dockerfile` — Complete multi-stage build, no changes needed
- `.do/app.yaml` — Shows current deployment pattern (for reference)

### Established Patterns
- No existing GitHub Actions workflows — this is the first one
- Ansible used for DigitalOcean provisioning (separate concern)

### Integration Points
- `.github/workflows/docker.yml` — new file, no conflicts
- GHCR package settings may need manual visibility toggle after first push

</code_context>

<specifics>
## Specific Ideas

No specific requirements — standard GitHub Actions Docker workflow pattern. The Dockerfile is already production-ready.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 18-docker-distribution*
*Context gathered: 2026-04-03*
