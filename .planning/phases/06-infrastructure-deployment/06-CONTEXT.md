# Phase 6: Infrastructure & Deployment - Context

**Gathered:** 2026-02-02
**Status:** Ready for planning

<domain>
## Phase Boundary

Deploy the Rust MCP server to production via Docker on Render, with a static landing page at 3gs.ai (GitHub Pages) and API at api.3gs.ai (Render). Includes Dockerfile, render.yaml, landing page HTML, and DNS configuration documentation.

</domain>

<decisions>
## Implementation Decisions

### Landing page content
- Audience: both general visitors and developers/agents
- Split content: explain what 3GS is AND show how to connect
- Minimal plain HTML — no CSS framework, fast loading, like a well-formatted README
- Static HTML with links to /health and /registry for live data inspection
- Include MCP client config JSON snippet AND a curl example for testing the endpoint
- Hosted on GitHub Pages at 3gs.ai root domain

### Docker build strategy
- Multi-stage build with debian:bookworm-slim as final runtime image
- registry.json mounted as volume at runtime — not baked into image, can update without rebuild
- Use cargo-chef for dependency caching layer — faster rebuilds when only source changes
- Standard .dockerignore (target/, .git/, .planning/, .env)
- Run as dedicated non-root user inside the container

### Render configuration
- Paid tier (not free) — no spin-down or cold start concerns
- render.yaml declares all environment variables with defaults; secrets overridden in Render dashboard
- Auto-deploy on push to main branch
- Health check: GET /health endpoint (returns 200 with version and pubkey)

### Domain & TLS setup
- 3gs.ai domain purchased and DNS accessible
- API lives at subdomain: api.3gs.ai (Render service)
- Landing page at root: 3gs.ai (GitHub Pages)
- Render handles TLS for api.3gs.ai automatically
- GitHub Pages handles TLS for 3gs.ai

### Claude's Discretion
- Exact Dockerfile layer ordering and optimization
- render.yaml structure details
- Landing page HTML structure and copy
- DNS record documentation format
- cargo-chef recipe generation approach

</decisions>

<specifics>
## Specific Ideas

- Landing page should link to /health and /registry as live endpoints visitors can check
- Connection docs should include both MCP client JSON config and curl command
- registry.json as volume mount allows updating sources without redeploying

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 06-infrastructure-deployment*
*Context gathered: 2026-02-02*
