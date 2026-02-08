---
phase: 10-digitalocean-provisioning
plan: 01
subsystem: infra
tags: [digitalocean, ansible, docker, app-platform]

requires:
  - phase: 09-cors-hardening
    provides: Production-ready CORS config for deployment
provides:
  - DO App Platform app spec (.do/app.yaml)
  - Ansible provisioning playbook for DO deployment
  - Live deployment at three-good-sources-api-238s5.ondigitalocean.app
affects: [11-dns-cutover]

tech-stack:
  added: [digitalocean-app-platform, ansible-digitalocean-cloud]
  patterns: [infrastructure-as-code, vault-based-secrets]

key-files:
  created:
    - .do/app.yaml
    - ansible/playbooks/provision-do.yml
    - ansible/vars/do-secrets.yml.example
    - ansible/requirements.yml
  modified:
    - .gitignore

key-decisions:
  - "Use DO source build from Dockerfile (not pre-built GHCR image) — simpler setup"
  - "DO token in vault vars file instead of env var — matches existing Ansible workflow"
  - "PKARR_SECRET_KEY omitted — server generates ephemeral keypair, production key added later"
  - "App named three-good-sources-api — DO naming constraints (no leading digits)"
  - "Render decommissioned early — user deleted before phase completion"
  - "No vault encryption — secrets file gitignored, optional encryption later"

patterns-established:
  - "Ansible provisioning: vars file for secrets, slurp+from_yaml for app spec"
  - "DO app spec at .do/app.yaml as source of truth for deployment config"

duration: 15min
completed: 2026-02-08
---

# Phase 10: DigitalOcean Provisioning Summary

**DO App Platform deployment via Ansible with Dockerfile build, health checks, and auto-deploy on push to main**

## Performance

- **Duration:** ~15 min (interactive — checkpoint required manual provisioning)
- **Started:** 2026-02-08
- **Completed:** 2026-02-08
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- DO App Platform app spec with Dockerfile deployment, port 3000, health check at /health
- Ansible playbook provisions app with vault-based secret management
- App live and healthy at three-good-sources-api-238s5.ondigitalocean.app
- Auto-deploy configured on push to main branch
- Zero secrets in git — DO token and PKARR key in gitignored vars file

## Task Commits

1. **Task 1: Create DO app spec and Ansible provisioning files** - `9f97129` (feat)
2. **Task 1 fixes: Simplify provisioning** - `93bbaac` (fix)

## Files Created/Modified
- `.do/app.yaml` - DO App Platform app spec with Docker build config
- `ansible/playbooks/provision-do.yml` - Ansible playbook for DO provisioning
- `ansible/vars/do-secrets.yml.example` - Template for secrets (DO token, PKARR key)
- `ansible/requirements.yml` - Ansible collection dependencies
- `.gitignore` - Excludes real secrets file

## Decisions Made
- Used DO source build from Dockerfile instead of GHCR — simpler, no registry setup needed
- Moved DO token from env var to vault vars file — consistent with user's existing Ansible workflow
- Omitted PKARR_SECRET_KEY from app spec — server auto-generates ephemeral keypair
- No vault encryption required — gitignore sufficient for local-only secrets file
- App created manually in DO Console first (GitHub auth), then managed by Ansible

## Deviations from Plan

### Adjustments During Execution

1. **App naming:** Plan specified `three-good-sources-api` / `api` but DO constraints required manual app creation first. Final names: app `three-good-sources-api`, service `api`.

2. **Secret management simplified:** Plan had Ansible injecting PKARR_SECRET_KEY into app spec at deploy time. Simplified to omit PKARR entirely (ephemeral keypair) and moved DO token into vars file.

3. **Render decommissioned early:** Plan required Render as rollback target. User deleted Render before phase completion — DO deployment proven healthy, rollback not needed.

**Impact on plan:** Simplifications reduced complexity without affecting deployment outcome.

## Issues Encountered
- GitHub authorization required before Ansible could provision — resolved by creating app manually in DO Console first
- DO API response structure different than expected — `default_ingress` not in response, fixed debug task to dump full result
- Duplicate app created (manual + Ansible) — deleted Ansible-created `threegoodsources-app`, kept manual `three-good-sources-api`

## Next Phase Readiness
- DO deployment live and healthy — ready for DNS cutover
- Custom domain configuration (3gs.ai, api.3gs.ai) is Phase 11
- Render already decommissioned — Phase 11 can skip decommission step

---
*Phase: 10-digitalocean-provisioning*
*Completed: 2026-02-08*
