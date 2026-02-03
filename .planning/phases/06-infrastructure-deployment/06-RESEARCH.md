# Phase 6: Infrastructure & Deployment - Research

**Researched:** 2026-02-02
**Domain:** Docker containerization, Render deployment, GitHub Pages static hosting
**Confidence:** HIGH

## Summary

Phase 6 deploys the Rust MCP server to production via Docker on Render's paid tier, with a static landing page at 3gs.ai (GitHub Pages) and API at api.3gs.ai (Render). Research confirms the standard approach uses multi-stage Dockerfile with cargo-chef for dependency caching, debian:bookworm-slim runtime for compatibility, and Render's render.yaml Blueprint for declarative infrastructure. GitHub Pages supports branch-based publishing for static HTML with automatic TLS. All user decisions from CONTEXT.md align with current best practices.

**Key findings:**
- cargo-chef is the standard for Rust Docker builds, providing 5x faster rebuilds via dependency layer caching
- Render's Blueprint specification supports Docker runtime with auto-deploy, health checks, and environment variable injection
- GitHub Pages apex domain requires A/AAAA records pointing to GitHub IPs; Render subdomain uses CNAME
- Volume mounts for registry.json enable runtime updates without image rebuilds
- Paid tier eliminates free tier spin-down delays and 750-hour monthly limits

**Primary recommendation:** Use three-stage Dockerfile (planner/builder/runtime) with cargo-chef, mount registry.json as read-only volume, configure health check at /health, and deploy via render.yaml with auto-deploy on main branch push.

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| cargo-chef | Latest | Dependency caching for Docker builds | De facto standard for Rust Docker builds; automatically detects workspace crates, leverages layer caching for 5x speedups |
| debian:bookworm-slim | Latest | Runtime base image | Better glibc compatibility than Alpine, minimal footprint (~150MB vs 20MB for Alpine), recommended for Rust binaries with standard dependencies |
| Docker BuildKit | Default | Build engine with cache mounts | Native Docker feature; enables --mount=type=cache for persistent caching across builds |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| lukemathwalker/cargo-chef | rust:1.84 | Pre-built cargo-chef images | Use as base for planner/builder stages to ensure consistent Rust versions (required for caching) |
| .dockerignore | N/A | Build context filtering | Always use; excludes target/, .git/, .env to prevent bloat and secret leaks |
| render.yaml | Blueprint v1 | Infrastructure as Code | Declarative service configuration; version-controlled deployment settings |
| GitHub Actions | Default | Automated deployments | Automatic for GitHub Pages; workflow runs on push to trigger deploys |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| debian:bookworm-slim | alpine:latest | Alpine: smaller image (~25MB final) but musl libc incompatibilities, less standard tooling. Use Debian for compatibility unless size is critical. |
| cargo-chef | Manual COPY Cargo.toml | Manual approach rebuilds all dependencies on any file change; cargo-chef isolates dependency changes for smarter caching. |
| render.yaml | Render Dashboard config | Dashboard works but isn't version-controlled; Blueprint enables GitOps and reproducible infrastructure. |
| Branch publishing (Pages) | GitHub Actions workflow | Actions required for custom build steps (Jekyll alternatives); branch publishing sufficient for plain HTML. |

**Installation:**
```bash
# Installed via Dockerfile FROM directives, not local installation
# cargo-chef installed in build stage: cargo install cargo-chef --locked
# .dockerignore created manually in project root
# render.yaml created in project root
```

## Architecture Patterns

### Recommended Project Structure
```
/
├── Dockerfile           # Multi-stage build definition
├── .dockerignore        # Build context exclusions
├── render.yaml          # Render Blueprint specification
├── docs/
│   └── index.html       # Landing page for GitHub Pages
├── registry.json        # Runtime-mounted configuration
└── src/                 # Application code
```

### Pattern 1: Multi-Stage Dockerfile with cargo-chef

**What:** Three-stage build separating dependency caching (planner), compilation (builder), and runtime (runtime). Planner generates recipe.json from Cargo.toml/Cargo.lock, builder cooks dependencies and compiles app, runtime copies only the binary.

**When to use:** All Rust Docker builds where dependency changes are less frequent than code changes (standard case).

**Example:**
```dockerfile
# Source: https://depot.dev/docs/languages/rust-dockerfile (verified official depot.dev docs)
# Stage 1: Planner - analyze dependencies
FROM lukemathwalker/cargo-chef:latest-rust-1.84 AS planner
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Builder - cache dependencies and build
FROM lukemathwalker/cargo-chef:latest-rust-1.84 AS builder
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
# Cook dependencies (cached layer)
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json
# Copy source and build application
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cargo build --release --bin three-good-sources

# Stage 3: Runtime - minimal production image
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
# Create non-root user for security
RUN groupadd -g 1001 appgroup && \
    useradd -u 1001 -g appgroup -m -d /home/appuser appuser
COPY --from=builder --chown=appuser:appgroup /app/target/release/three-good-sources /usr/local/bin/app
USER appuser
EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/app"]
```

### Pattern 2: Read-Only Volume Mount for Configuration

**What:** Mount registry.json as read-only volume at runtime instead of baking into image. Allows updating curated sources without rebuilding/redeploying container.

**When to use:** Configuration files that change more frequently than code (registry.json updates vs application releases).

**Example:**
```yaml
# Source: https://render.com/docs/blueprint-spec (verified official Render docs)
# In render.yaml:
services:
  - type: web
    name: three-good-sources-api
    runtime: docker
    # ... other config ...
    disk:
      name: registry-data
      mountPath: /data
      sizeGB: 1
# In Dockerfile, application reads from /data/registry.json
# Update registry.json via Render shell or disk management without redeploy
```

### Pattern 3: Health Check Endpoint for Zero-Downtime Deploys

**What:** Implement GET /health endpoint returning 200 OK with version/pubkey. Render polls this during deployment to verify new instance is ready before routing traffic.

**When to use:** All production web services on Render; enables zero-downtime rolling deploys.

**Example:**
```yaml
# Source: https://render.com/docs/health-checks (verified official Render docs)
# In render.yaml:
services:
  - type: web
    name: three-good-sources-api
    healthCheckPath: /health
    # Render sends GET /health every few seconds
    # 2xx/3xx = healthy, can receive traffic
    # 15 consecutive minutes of failures = deploy cancelled, rollback
```

### Pattern 4: GitHub Pages Apex Domain with A Records

**What:** Configure DNS apex domain (3gs.ai) with four A records pointing to GitHub IPs, configure custom domain in repo settings, enforce HTTPS. Subdomain (api.3gs.ai) uses CNAME to Render.

**When to use:** Hosting static landing page on GitHub Pages with custom apex domain.

**Example:**
```bash
# Source: https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site/managing-a-custom-domain-for-your-github-pages-site (verified official GitHub docs)
# DNS configuration at domain provider:
# A records for 3gs.ai (apex):
3gs.ai.  A  185.199.108.153
3gs.ai.  A  185.199.109.153
3gs.ai.  A  185.199.110.153
3gs.ai.  A  185.199.111.153

# CNAME for api.3gs.ai (Render subdomain):
api.3gs.ai.  CNAME  three-good-sources-api.onrender.com

# In GitHub repo Settings > Pages:
# - Custom domain: 3gs.ai
# - Enforce HTTPS: ✓ (after DNS propagates, ~24 hours)
```

### Anti-Patterns to Avoid

- **Baking registry.json into Docker image:** Forces rebuild/redeploy for content updates. Use volume mount for separation of code and data.
- **Using different Rust versions across stages:** Breaks cargo-chef caching. Use same lukemathwalker/cargo-chef:latest-rust-X.YZ image for planner and builder.
- **Omitting .dockerignore:** Sends target/ (gigabytes), .git/ (history), .env (secrets) to build context. Slows builds, risks secret exposure.
- **Hardcoding environment variables in Dockerfile:** Secrets baked into image layers are insecure. Use Render environment variables injected at runtime.
- **Using EXPOSE without documenting PORT:** EXPOSE is documentation; doesn't publish port. Document PORT=3000 in README and use env var for runtime flexibility.
- **Free tier assumptions:** Free tier spins down after 15min inactivity, has 750hr/month limit. User is on paid tier; no spin-down concerns.

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Docker dependency caching | Manual COPY Cargo.toml && cargo build approach | cargo-chef | Handles workspace detection, file reorganization, and recipe generation automatically. Manual approaches miss edge cases. |
| TLS certificate management | Custom Let's Encrypt automation | Render auto TLS | Render automatically provisions and renews certificates for custom domains. No configuration needed. |
| Health check logic | Custom readiness probes | Existing /health endpoint | Already implemented in Phase 4 (04-02); returns 200 with version/pubkey. Just wire into render.yaml. |
| Static site deployment | Custom S3/CloudFront setup | GitHub Pages branch publishing | Free, automatic TLS, integrated with GitHub workflow. Zero-config for static HTML. |
| Environment variable injection | Custom config file parsing | Render's envVars in render.yaml | Platform handles build-time and runtime injection securely. Supports secrets, sync: false prompting, generateValue. |
| DNS management complexity | Manual record updates | Domain provider's UI + Render guidance | Render docs provide provider-specific guides for Cloudflare, Namecheap, etc. Follow official patterns. |

**Key insight:** Infrastructure automation and deployment tooling have matured significantly. Render and GitHub Pages provide opinionated, zero-config defaults for common patterns (TLS, health checks, deployments). Custom solutions add maintenance burden without feature benefits.

## Common Pitfalls

### Pitfall 1: Rust Version Mismatch Across Build Stages

**What goes wrong:** Using rust:1.84 for planner and rust:1.85 for builder stage. cargo-chef cache invalidates on every build despite no dependency changes.

**Why it happens:** Assumption that "latest stable" is fine for all stages. cargo-chef's caching is fingerprint-based on Rust version; version mismatch = different fingerprint = cache miss.

**How to avoid:** Use explicit, matching versions from lukemathwalker/cargo-chef:latest-rust-1.84 for both planner and builder stages. Pin the version, don't rely on latest.

**Warning signs:** Docker cache always rebuilding dependencies despite no Cargo.toml/Cargo.lock changes. Build times stay slow (several minutes) instead of dropping to seconds after first build.

### Pitfall 2: Secrets in Dockerfile ENV/ARG

**What goes wrong:** Setting PKARR_SECRET_KEY via ENV or ARG in Dockerfile. Secret embedded in image layers, visible via docker history, shipped to registry.

**Why it happens:** Confusion between build-time arguments and runtime environment variables. ENV/ARG in Dockerfile = baked into image. Runtime injection needed for secrets.

**How to avoid:** Declare secrets in render.yaml envVars with sync: false (prompted once) or set in Render Dashboard environment variables. Never reference secrets in Dockerfile.

**Warning signs:** docker history shows sensitive values. Image pushed to registry contains secrets accessible to anyone with image access.

### Pitfall 3: Missing .dockerignore Causes Slow Builds

**What goes wrong:** No .dockerignore file. Docker sends target/ directory (gigabytes of build artifacts) to build context on every docker build. Build startup takes minutes, cache invalidates unnecessarily.

**Why it happens:** .dockerignore is optional; builds work without it. Impact invisible until project grows and target/ accumulates artifacts.

**How to avoid:** Create .dockerignore on day one with: target/, .git/, .planning/, .env, Cargo.lock (copied explicitly when needed).

**Warning signs:** "Sending build context to Docker daemon" shows hundreds of MB or GB. Builds slow down over time as target/ grows.

### Pitfall 4: Free Tier Spin-Down Delays

**What goes wrong:** Deploying to Render free tier. Service spins down after 15min inactivity, takes 30-60 seconds to spin up on next request. Users see timeouts or extreme latency.

**Why it happens:** Roadmap says "free tier" but user has clarified they're on paid tier. Free tier has 750hr/month limit and spin-down behavior.

**How to avoid:** User is on paid tier per CONTEXT.md; no action needed. Verify paid tier configuration in Render Dashboard to ensure always-on behavior.

**Warning signs:** First request after idle period takes 30+ seconds. Render Dashboard shows "Free" instance type instead of paid tier.

### Pitfall 5: GitHub Pages Jekyll Processing Breaks Plain HTML

**What goes wrong:** Pushing index.html to GitHub Pages. Jekyll processes HTML, potentially breaking custom structure or adding unwanted layouts.

**Why it happens:** GitHub Pages defaults to Jekyll processing for all sites, even plain HTML.

**How to avoid:** Add empty .nojekyll file to docs/ directory (or repo root if publishing from root). Disables Jekyll processing, serves HTML as-is.

**Warning signs:** HTML renders differently on GitHub Pages than locally. Unexpected layouts or formatting appear. Liquid template syntax ({{ }}) in HTML gets processed.

### Pitfall 6: Health Check Path Mismatch

**What goes wrong:** Setting healthCheckPath: /healthz in render.yaml but application implements GET /health. Render sends requests to /healthz, gets 404, cancels deploy, rolls back.

**Why it happens:** Copy-paste from other projects with different conventions (/healthz, /ready, /live are all common).

**How to avoid:** Match healthCheckPath in render.yaml to actual endpoint path in application. Verify with curl http://localhost:3000/health locally before deploying.

**Warning signs:** Render deploys fail with "health check timeout" errors. New instance never becomes healthy, deploy rolls back to previous version.

### Pitfall 7: DNS Propagation Delays Break Verification

**What goes wrong:** Adding custom domain to GitHub Pages/Render immediately after creating DNS records. Verification fails because DNS hasn't propagated globally (up to 24 hours).

**Why it happens:** DNS changes aren't instant. Local dig may show records (cached at ISP) while GitHub/Render verification uses different DNS servers that haven't updated.

**How to avoid:** Create DNS records, wait 30-60 minutes, verify with dig @8.8.8.8 3gs.ai (Google's DNS) before adding domain to platform settings.

**Warning signs:** Domain verification fails with "DNS record not found" errors. dig locally shows records but platforms can't see them.

## Code Examples

Verified patterns from official sources:

### Complete Dockerfile for Rust MCP Server

```dockerfile
# Source: https://depot.dev/docs/languages/rust-dockerfile (verified pattern)
# Adapted for three-good-sources project

# syntax=docker/dockerfile:1

# Stage 1: Planner - generate dependency recipe
FROM lukemathwalker/cargo-chef:latest-rust-1.84 AS planner
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Builder - cache dependencies and compile
FROM lukemathwalker/cargo-chef:latest-rust-1.84 AS builder
WORKDIR /app

# Copy recipe from planner
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies (cached layer)
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json

# Copy source code
COPY . .

# Build application with size optimizations
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cargo build --release --bin three-good-sources

# Stage 3: Runtime - minimal production image
FROM debian:bookworm-slim AS runtime

# Install CA certificates for HTTPS
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Create non-root user (security best practice)
RUN groupadd -g 1001 appgroup && \
    useradd -u 1001 -g appgroup -m -d /home/appuser appuser

# Copy binary from builder with correct ownership
COPY --from=builder --chown=appuser:appgroup \
    /app/target/release/three-good-sources \
    /usr/local/bin/app

# Switch to non-root user
USER appuser

# Document exposed port (doesn't publish, just documentation)
EXPOSE 3000

# Run application
ENTRYPOINT ["/usr/local/bin/app"]
```

### .dockerignore File

```bash
# Source: https://marsbased.com/blog/2026/01/09/always-use-a-dockerignore-in-your-projects (verified best practices)
# Exclude build artifacts
target/
Cargo.lock

# Exclude version control
.git/
.gitignore

# Exclude planning documents
.planning/

# Exclude environment files (secrets)
.env
.env.*

# Exclude local development files
*.swp
*.swo
*~

# Exclude documentation
README.md
LICENSE
docs/

# Exclude test directories
tests/
```

### render.yaml Blueprint

```yaml
# Source: https://render.com/docs/blueprint-spec (verified official spec)
# Declarative infrastructure configuration for Render

services:
  - type: web
    name: three-good-sources-api
    runtime: docker
    region: oregon # or closest to target users
    plan: starter # paid tier, no spin-down
    branch: main

    # Health check for zero-downtime deploys
    healthCheckPath: /health

    # Auto-deploy on push to main
    autoDeploy: true

    # Environment variables
    envVars:
      - key: REGISTRY_PATH
        value: /data/registry.json
      - key: LOG_FORMAT
        value: json
      - key: PORT
        value: 3000
      - key: RUST_LOG
        value: info
      - key: PKARR_SECRET_KEY
        sync: false # Prompt once, stored securely

    # Persistent disk for registry.json
    disk:
      name: registry-data
      mountPath: /data
      sizeGB: 1
```

### Landing Page HTML (docs/index.html)

```html
<!-- Source: Landing page best practices from https://www.involve.me/blog/landing-page-best-practices -->
<!-- Minimal HTML with no CSS framework, fast loading, clear information hierarchy -->
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Three Good Sources - Curated Sources for AI Agents</title>
    <meta name="description" content="Get three curated, high-quality sources per topic instead of SEO-gamed search results. Cryptographically signed, agent-ready, open protocol.">
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            line-height: 1.6;
            max-width: 800px;
            margin: 0 auto;
            padding: 2rem 1rem;
            color: #333;
        }
        h1 { font-size: 2rem; margin-bottom: 0.5rem; }
        h2 { font-size: 1.5rem; margin-top: 2rem; border-bottom: 1px solid #ddd; padding-bottom: 0.5rem; }
        code { background: #f4f4f4; padding: 0.2rem 0.4rem; border-radius: 3px; font-size: 0.9em; }
        pre { background: #f4f4f4; padding: 1rem; border-radius: 5px; overflow-x: auto; }
        a { color: #0066cc; text-decoration: none; }
        a:hover { text-decoration: underline; }
        .status { display: inline-block; padding: 0.2rem 0.6rem; background: #28a745; color: white; border-radius: 3px; font-size: 0.85em; }
        ul { padding-left: 1.5rem; }
        li { margin-bottom: 0.5rem; }
    </style>
</head>
<body>
    <h1>Three Good Sources (3GS)</h1>
    <p><span class="status">LIVE</span> <a href="https://api.3gs.ai/health">Health Check</a> | <a href="https://api.3gs.ai/registry">Registry Data</a></p>

    <h2>What is 3GS?</h2>
    <p>AI agents get curated, high-quality sources instead of SEO-gamed search results. For every topic, we provide exactly three good sources: primary documentation, practical guides, and essential tools.</p>

    <p><strong>Why three?</strong> Enough to triangulate truth, few enough to stay focused. Human-curated, cryptographically signed, served via open protocol.</p>

    <h2>How to Connect</h2>

    <h3>For AI Agents (MCP Clients)</h3>
    <p>Add this configuration to your MCP client (Claude Desktop, etc.):</p>

    <pre><code>{
  "mcpServers": {
    "three-good-sources": {
      "command": "node",
      "args": ["-e", "require('node-fetch')('https://api.3gs.ai/mcp', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ jsonrpc: '2.0', method: arguments[0], params: arguments[1], id: 1 }) }).then(r => r.json()).then(console.log)", process.argv[2], process.argv[3]]
    }
  }
}</code></pre>

    <h3>For Developers (HTTP API)</h3>
    <p>Test the endpoint with curl:</p>

    <pre><code>curl -X POST https://api.3gs.ai/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "get_sources",
      "arguments": {
        "query": "learn rust"
      }
    },
    "id": 1
  }'</code></pre>

    <h2>Available Tools</h2>
    <ul>
        <li><strong>get_sources(query)</strong> - Find three curated sources for a topic</li>
        <li><strong>list_categories()</strong> - Browse all available categories</li>
        <li><strong>get_provenance()</strong> - Verify curator identity and signature</li>
        <li><strong>get_endorsements(category_slug)</strong> - See community endorsements (v2 feature)</li>
    </ul>

    <h2>Live Data</h2>
    <ul>
        <li><a href="https://api.3gs.ai/health">/health</a> - Server version and public key</li>
        <li><a href="https://api.3gs.ai/registry">/registry</a> - Full registry data (transparency)</li>
    </ul>

    <h2>Vision</h2>
    <p>3GS is the first node in a decentralized knowledge graph. Curators run their own servers with their own taste. Agents query multiple curators and triangulate consensus. Cryptographic signatures prevent tampering. Open protocol prevents platform lock-in.</p>

    <p><em>Built with Rust, powered by <a href="https://pkarr.org">Pubky</a> identity, served via MCP.</em></p>
</body>
</html>
```

### DNS Configuration Documentation

```markdown
# Source: https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site/managing-a-custom-domain-for-your-github-pages-site

# DNS Setup for 3gs.ai Domain

## 1. Configure Apex Domain for GitHub Pages (3gs.ai)

At your DNS provider, create these A records:

| Type | Name | Value           | TTL  |
|------|------|-----------------|------|
| A    | @    | 185.199.108.153 | 3600 |
| A    | @    | 185.199.109.153 | 3600 |
| A    | @    | 185.199.110.153 | 3600 |
| A    | @    | 185.199.111.153 | 3600 |

## 2. Configure Subdomain for Render (api.3gs.ai)

At your DNS provider, create this CNAME record:

| Type  | Name | Value                              | TTL  |
|-------|------|------------------------------------|------|
| CNAME | api  | three-good-sources-api.onrender.com | 3600 |

## 3. Verify DNS Propagation

Wait 30-60 minutes, then check:

```bash
# Check apex domain
dig @8.8.8.8 3gs.ai A

# Check subdomain
dig @8.8.8.8 api.3gs.ai CNAME
```

## 4. Configure GitHub Pages

1. Navigate to repository Settings > Pages
2. Under "Custom domain", enter: 3gs.ai
3. Click Save (creates CNAME file in docs/ directory)
4. Wait for DNS check to pass (green checkmark)
5. Enable "Enforce HTTPS" (may take up to 24 hours)

## 5. Configure Render Custom Domain

1. Open Render Dashboard > three-good-sources-api service
2. Navigate to Settings > Custom Domains
3. Click "Add Custom Domain"
4. Enter: api.3gs.ai
5. Wait for verification (Render checks CNAME record)
6. TLS certificate auto-provisions (usually < 5 minutes)

## Troubleshooting

- DNS propagation can take up to 24 hours globally
- Use dig @8.8.8.8 to bypass local DNS caching
- GitHub Pages verification requires all 4 A records present
- Render requires CNAME pointing to *.onrender.com subdomain
```

### Build Size Optimization (Cargo.toml)

```toml
# Source: https://github.com/johnthagen/min-sized-rust (verified best practices)
# Add to existing Cargo.toml

[profile.release]
opt-level = "z"        # Optimize for smallest code size
lto = "fat"            # Enable full Link Time Optimization
codegen-units = 1      # Reduce parallel codegen for better optimization
panic = "abort"        # Use abort instead of unwind (smaller binary)
strip = "symbols"      # Strip debug symbols from binary

# This configuration reduces binary size from ~15MB to ~5MB
# Trade-off: slightly slower compilation, smaller runtime footprint
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual Cargo.toml copying | cargo-chef recipe generation | 2020 | Automatic workspace detection, 5x faster rebuilds |
| Alpine base images | debian:bookworm-slim | 2023-2024 | Better glibc compatibility, standard tooling, minor size increase (25MB→150MB) acceptable |
| Manual TLS via Certbot | Platform-native auto TLS | 2021+ | Zero configuration, automatic renewal, applies to Render and GitHub Pages |
| docker-compose for deploys | render.yaml Blueprint | 2022 | Render doesn't support Compose; Blueprint is declarative IaC |
| Free tier (750hr limit) | Paid tier always-on | User choice | No spin-down delays, persistent disk, static IPs available |
| Jekyll for GitHub Pages | Branch publishing with .nojekyll | 2024+ | Plain HTML no longer processed, faster deploys, no Ruby dependencies |

**Deprecated/outdated:**
- **Render free tier for production:** Free tier has 15min spin-down, 750hr/month limit, no persistent disk. Paid tier required for production reliability (user is already on paid tier).
- **Alpine for Rust web services:** Alpine's musl libc causes compatibility issues with Rust's glibc assumptions. Use debian:bookworm-slim unless size is absolutely critical (<50MB target).
- **Hardcoded secrets in Dockerfile:** Modern platforms inject environment variables at runtime. Never use ENV/ARG for secrets in Dockerfile (security risk).

## Open Questions

Things that couldn't be fully resolved:

1. **Render disk mount behavior with Docker volumes**
   - What we know: render.yaml disk field creates persistent volume at mountPath. Render docs confirm this works with Docker runtime.
   - What's unclear: Whether registry.json should be in project root (baked into image) AND mounted at /data, or ONLY mounted. Safer to mount and update manually via Render shell initially.
   - Recommendation: Start with registry.json baked into image (COPY registry.json /app/registry.json) for initial deploy. Phase 7 can explore disk mount updates for dynamic registry updates without rebuild.

2. **cargo-chef with workspace projects**
   - What we know: cargo-chef automatically detects workspace crates. Project uses edition = "2024" (workspace feature).
   - What's unclear: Whether single-crate project benefits from workspace detection (probably no-op).
   - Recommendation: Current Cargo.toml has single package; cargo-chef still recommended for future-proofing if workspace added later.

3. **GitHub Pages build time after push**
   - What we know: GitHub docs say "up to 10 minutes" for changes to publish. Branch publishing triggers automatic workflow.
   - What's unclear: Whether .nojekyll affects build time (probably yes, faster since no Jekyll processing).
   - Recommendation: Expect 1-3 minute publish time for plain HTML with .nojekyll. Monitor Actions tab for actual workflow duration.

## Sources

### Primary (HIGH confidence)

- [cargo-chef GitHub Repository](https://github.com/LukeMathWalker/cargo-chef) - Installation, workflow, limitations (official)
- [Render Blueprint YAML Reference](https://render.com/docs/blueprint-spec) - Service configuration, environment variables, health checks (official)
- [Render Docker Documentation](https://render.com/docs/docker) - Docker runtime, build process, environment variable injection (official)
- [GitHub Pages Custom Domain Management](https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site/managing-a-custom-domain-for-your-github-pages-site) - DNS configuration, A/AAAA records (official)
- [GitHub Pages Site Creation](https://docs.github.com/en/pages/getting-started-with-github-pages/creating-a-github-pages-site) - Publishing methods, branch vs Actions (official)
- [Depot Rust Dockerfile Guide](https://depot.dev/docs/languages/rust-dockerfile) - Complete multi-stage example with cargo-chef and sccache (verified official depot.dev)
- [Render Health Checks Documentation](https://render.com/docs/health-checks) - Zero-downtime deploy process, timeout behavior (official)
- [Render Custom Domains Documentation](https://render.com/docs/custom-domains) - CNAME configuration, TLS auto-provisioning (official)

### Secondary (MEDIUM confidence)

- [Fast Rust Docker Builds with cargo-chef (Luca Palmieri)](https://www.lpalmieri.com/posts/fast-rust-docker-builds/) - Performance benchmarks, 5x speedup claims (verified author is cargo-chef creator)
- [Always Use .dockerignore (MarsBased, 2026)](https://marsbased.com/blog/2026/01/09/always-use-a-dockerignore-in-your-projects) - Security risks, performance impact (recent, credible source)
- [Securing Docker Non-Root User Best Practices (Medium, 2024)](https://medium.com/@Kfir-G/securing-docker-non-root-user-best-practices-5784ac25e755) - Security rationale, implementation patterns (multiple sources confirm)
- [Docker Volume Management Best Practices (DevOps Training Institute, 2025)](https://www.devopstraininginstitute.com/blog/12-best-practices-for-docker-volume-management) - Named volumes, read-only mounts, security (cross-verified with Docker docs)
- [Rust Binary Size Optimization (johnthagen/min-sized-rust)](https://github.com/johnthagen/min-sized-rust) - Profile.release configuration, strip option (community standard)
- [Landing Page Best Practices 2026 (involve.me)](https://www.involve.me/blog/landing-page-best-practices) - Minimal design, single CTA, white space usage (verified with multiple sources)
- [How to Create Minimal Docker Images for Rust Binaries (OneUpTime, 2026)](https://oneuptime.com/blog/post/2026-01-07-rust-minimal-docker-images/view) - Alpine vs Debian comparison (recent, technical depth)

### Tertiary (LOW confidence)

- [Render Free Tier Limitations (Community Discussion)](https://community.render.com/t/confused-about-the-free-tier/19092) - Spin-down behavior, hour limits (community-reported, cross-verified with official docs)
- [MCP Client Configuration Examples (adhikasp/mcp-client-cli)](https://github.com/adhikasp/mcp-client-cli/blob/master/mcp-server-config-example.json) - JSON structure for mcpServers (unofficial example, pattern confirmed across multiple clients)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - cargo-chef, debian:bookworm-slim, render.yaml, GitHub Pages all confirmed via official documentation
- Architecture: HIGH - Multi-stage Dockerfile pattern verified in Depot official docs; Render Blueprint spec confirmed in official Render docs; GitHub Pages A records confirmed in GitHub docs
- Pitfalls: MEDIUM - Rust version mismatch, secrets in Dockerfile, .dockerignore omission verified in multiple sources; free tier spin-down confirmed in official Render docs; DNS propagation delays are standard behavior

**Research date:** 2026-02-02
**Valid until:** 2026-03-02 (30 days - stable domain, best practices evolve slowly)
