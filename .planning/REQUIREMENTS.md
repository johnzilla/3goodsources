# Requirements — v1.1 Migrate to DigitalOcean + Tech Debt

## Deployment

- [ ] **DEPLOY-01**: App deployed to DO App Platform via Docker from app spec YAML
- [ ] **DEPLOY-02**: Environment variables configured (PORT, RUST_LOG, REGISTRY_PATH, PKARR_SECRET_KEY as encrypted SECRET)
- [ ] **DEPLOY-03**: Health check configured at /health in app spec
- [ ] **DEPLOY-04**: Auto-deploy triggered on push to main branch
- [ ] **DEPLOY-05**: App provisioned via Ansible playbook using DO API (consistent with existing DO project)

## Domains & SSL

- [ ] **DNS-01**: Custom domain 3gs.ai served from DigitalOcean App Platform
- [ ] **DNS-02**: Custom domain api.3gs.ai served from DigitalOcean App Platform
- [ ] **DNS-03**: SSL certificates auto-provisioned via Let's Encrypt
- [ ] **DNS-04**: Render deployment decommissioned after successful cutover

## CORS Hardening

- [ ] **CORS-01**: CORS configured with specific origin allowlist (3gs.ai, api.3gs.ai) instead of permissive
- [ ] **CORS-02**: Cross-origin POST /mcp requests work for MCP agents after CORS tightening

## Dependency Cleanup

- [ ] **DEPS-01**: curve25519-dalek `[patch.crates-io]` section removed from Cargo.toml
- [ ] **DEPS-02**: Project builds and all tests pass without the patch

## Dead Code Removal

- [ ] **CLEAN-01**: Unused McpError enum removed from codebase

## Secrets Safety

- [ ] **SEC-01**: No secrets (PKARR_SECRET_KEY, DO API tokens) present in committed code, app spec YAML, or CI config
- [ ] **SEC-02**: App spec YAML uses `type: SECRET` for sensitive env vars (values set via Ansible, never in file)
- [ ] **SEC-03**: Git history verified clean of secrets before any public push

## Future Requirements

None deferred from this milestone.

## Out of Scope

- stdio/SSE MCP transport
- Agent feedback loop
- Community voting on sources
- doctl CLI (using Ansible instead for consistency)
- DO Container Registry (builds from Dockerfile)
- Multi-region deployment
- Custom buildpacks

## Traceability

| REQ-ID | Phase | Plan | Status |
|--------|-------|------|--------|
| DEPLOY-01 | — | — | pending |
| DEPLOY-02 | — | — | pending |
| DEPLOY-03 | — | — | pending |
| DEPLOY-04 | — | — | pending |
| DEPLOY-05 | — | — | pending |
| DNS-01 | — | — | pending |
| DNS-02 | — | — | pending |
| DNS-03 | — | — | pending |
| DNS-04 | — | — | pending |
| CORS-01 | — | — | pending |
| CORS-02 | — | — | pending |
| DEPS-01 | — | — | pending |
| DEPS-02 | — | — | pending |
| CLEAN-01 | — | — | pending |
| SEC-01 | — | — | pending |
| SEC-02 | — | — | pending |
| SEC-03 | — | — | pending |
