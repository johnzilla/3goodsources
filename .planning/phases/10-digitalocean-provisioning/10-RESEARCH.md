# Phase 10: DigitalOcean Provisioning - Research

**Researched:** 2026-02-08
**Domain:** DigitalOcean App Platform provisioning, Docker deployment, Ansible automation
**Confidence:** MEDIUM-HIGH

## Summary

DigitalOcean App Platform provides PaaS deployment capabilities through app spec YAML files (conventionally located at `.do/app.yaml`). The platform supports both Dockerfile-based deployments (matching our existing Render setup) and managed buildpacks. For Rust applications like ours, Dockerfile-based deployment provides better control and consistency with our existing working configuration.

Provisioning can be automated via Ansible using the `digitalocean.cloud.app` module, which directly accepts app spec definitions. The module supports idempotent create/update/delete operations. Secret environment variables use `type: SECRET` in the app spec but must be provided as plaintext on initial creation, then encrypted by the platform.

Key migration consideration: App Platform defaults to port 8080 (vs Render's 3000), requires explicit `http_port` configuration, and has specific Dockerfile constraints around `/var/run` paths and build arguments for environment variables.

**Primary recommendation:** Use Dockerfile deployment (not buildpack), leverage Ansible's `digitalocean.cloud.app` module for provisioning, store app spec at `.do/app.yaml` following DO conventions, and provision secrets via Ansible variables (never commit plaintext secrets to git).

## Standard Stack

### Core
| Library/Tool | Version | Purpose | Why Standard |
|--------------|---------|---------|--------------|
| DigitalOcean App Platform | N/A (PaaS) | Managed container hosting | DO-native PaaS, consistent with existing DO infrastructure |
| Ansible Collection `digitalocean.cloud` | Latest | Infrastructure automation | Official DO collection, replaces deprecated community.digitalocean |
| `pydo` | >= 0.1.7 | Python DO API client | Required dependency for digitalocean.cloud collection |
| `doctl` | Latest | DO CLI tool | Useful for local testing and manual operations |

### Supporting
| Library/Tool | Version | Purpose | When to Use |
|--------------|---------|---------|-------------|
| `azure-core` | >= 1.26.1 | Ansible collection dependency | Required by digitalocean.cloud |
| GitHub Actions `digitalocean/app_action` | v2 | CI/CD deployment | If using GitHub Actions instead of Ansible |
| DO API v2 | N/A | Direct API access | Fallback if Ansible module insufficient |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Ansible provisioning | doctl CLI | Less idempotent, harder to integrate with existing Ansible playbooks |
| Dockerfile deployment | Rust buildpack | Less control, harder to match Render config exactly, buildpack is newer |
| App spec YAML | UI-only configuration | Not version-controlled, can't automate |

**Installation:**
```bash
# Ansible collection and dependencies
pip3 install --user azure-core==1.26.1 pydo==0.1.7
ansible-galaxy collection install digitalocean.cloud

# Optional: doctl for local testing
# See: https://docs.digitalocean.com/reference/doctl/how-to/install/
```

## Architecture Patterns

### Recommended Project Structure
```
.do/
├── app.yaml              # App spec (DO convention)
ansible/
├── playbooks/
│   └── provision-do.yml  # DO provisioning playbook
├── inventory/
│   └── hosts.yml         # Inventory with DO API token
└── vars/
    └── do-secrets.yml    # Encrypted secrets (ansible-vault)
```

### Pattern 1: App Spec with Dockerfile Deployment
**What:** Define app configuration in `.do/app.yaml` pointing to existing Dockerfile
**When to use:** When you need consistent deployment config between Render and DO, have working Dockerfile
**Example:**
```yaml
# Source: https://docs.digitalocean.com/products/app-platform/reference/app-spec/
# and https://github.com/digitalocean/sample-dockerfile/blob/main/.do/app.yaml
name: three-good-sources-api
region: nyc
services:
  - name: api
    dockerfile_path: Dockerfile
    github:
      repo: owner/3goodsources
      branch: main
      deploy_on_push: true
    http_port: 3000  # CRITICAL: Explicitly set to match our app, DO default is 8080
    instance_count: 1
    instance_size_slug: basic-xxs
    health_check:
      http_path: /health
      initial_delay_seconds: 5
      period_seconds: 10
      timeout_seconds: 1
      success_threshold: 1
      failure_threshold: 9
    envs:
      - key: REGISTRY_PATH
        value: /app/registry.json
        scope: RUN_TIME
      - key: LOG_FORMAT
        value: json
        scope: RUN_TIME
      - key: PORT
        value: "3000"
        scope: RUN_TIME
      - key: RUST_LOG
        value: info
        scope: RUN_TIME
      - key: PKARR_SECRET_KEY
        value: "{{ will_be_encrypted_by_platform }}"
        type: SECRET
        scope: RUN_TIME
```

### Pattern 2: Ansible Provisioning with App Module
**What:** Use `digitalocean.cloud.app` module to create/update apps from spec
**When to use:** Automating DO provisioning, consistent with existing DO project's Ansible usage
**Example:**
```yaml
# Source: https://docs.digitalocean.com/reference/ansible/reference/modules/app/
---
- name: Provision DigitalOcean App Platform application
  hosts: localhost
  connection: local
  vars_files:
    - ../vars/do-secrets.yml  # Contains pkarr_secret_key
  tasks:
    - name: Load app spec from file
      slurp:
        src: "{{ playbook_dir }}/../../.do/app.yaml"
      register: app_spec_file

    - name: Parse app spec YAML
      set_fact:
        app_spec: "{{ app_spec_file.content | b64decode | from_yaml }}"

    - name: Inject secret environment variables
      set_fact:
        app_spec_with_secrets: "{{ app_spec | combine({'services': [app_spec.services[0] | combine({'envs': updated_envs})]}, recursive=true) }}"
      vars:
        updated_envs: "{{ app_spec.services[0].envs | map('combine', {'value': pkarr_secret_key}) | list if item.key == 'PKARR_SECRET_KEY' else app_spec.services[0].envs }}"

    - name: Create or update App Platform application
      digitalocean.cloud.app:
        token: "{{ lookup('env', 'DIGITALOCEAN_TOKEN') }}"
        state: present
        spec: "{{ app_spec_with_secrets }}"
        timeout: 600
      register: app_result

    - name: Display app URL
      debug:
        msg: "App deployed at: {{ app_result.app.default_ingress }}"
```

### Pattern 3: Secret Management Workflow
**What:** Provide plaintext secrets on initial create, use encrypted values on updates
**When to use:** Managing SECRET type environment variables
**Flow:**
1. Initial creation: Ansible provides plaintext secret value
2. Platform encrypts value, returns `EV[1:encrypted_string]` format
3. Subsequent updates: Use encrypted value from platform (fetch via API/doctl)
4. Never commit plaintext secrets to git

### Anti-Patterns to Avoid
- **Committing secrets to git:** SECRET type vars still need plaintext initially, never commit `.do/app.yaml` with real secret values
- **Using buildpack without testing:** Rust buildpack is newer (2026), Dockerfile is safer for migration
- **Hardcoding port 8080:** Our app listens on 3000, must explicitly set `http_port: 3000`
- **Skipping health checks:** App Platform requires health checks for proper zero-downtime deployment
- **Using latest Docker tags:** Breaks reproducibility, pin specific versions

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| DO API authentication | Custom token management | `DIGITALOCEAN_TOKEN` env var with Ansible | Official pattern, avoids plaintext tokens in playbooks |
| App spec validation | Manual YAML checking | `doctl apps spec validate .do/app.yaml` | Catches errors before deployment |
| Secret encryption | Custom encryption | App Platform's type: SECRET | Platform-native, integrates with DO secrets management |
| Zero-downtime deploys | Custom health check polling | App Platform health_check config | Built-in, handles container orchestration |
| Build caching | Custom Docker layer caching | App Platform automatic caching | Platform-optimized, faster builds |

**Key insight:** App Platform is a fully managed PaaS - let it handle container orchestration, health checks, and build caching rather than implementing custom solutions. Our existing Dockerfile already handles multi-stage builds and size optimization correctly.

## Common Pitfalls

### Pitfall 1: Secret Value Encryption Confusion
**What goes wrong:** Error "secret env value must not be encrypted before app is created" when using `EV[1:...]` format on first deployment
**Why it happens:** Platform expects plaintext on initial creation, encrypted values on subsequent updates
**How to avoid:**
- Initial provision: Pass plaintext secret via Ansible variable, mark as `type: SECRET`
- Platform encrypts value automatically
- Fetch encrypted value for subsequent updates (if needed)
**Warning signs:** Deployment fails with encryption-related error message

### Pitfall 2: Port Mismatch (8080 vs 3000)
**What goes wrong:** App Platform routes traffic to port 8080 by default, but our app listens on 3000
**Why it happens:** DO's default `http_port` is 8080, different from Render
**How to avoid:** Explicitly set `http_port: 3000` in app spec service configuration
**Warning signs:** Health checks fail, app appears unreachable despite successful deployment

### Pitfall 3: /var/run Path Extraction Issue
**What goes wrong:** App Platform fails to start containers based on Alpine Linux
**Why it happens:** Platform treats `/var/run` as special path and doesn't extract it from base images
**How to avoid:** Add to Dockerfile: `RUN test -e /var/run || ln -s /run /var/run`
**Warning signs:** Container starts but crashes immediately with file system errors
**Note:** Our current Dockerfile uses `debian:bookworm-slim`, not Alpine, so this doesn't apply to us

### Pitfall 4: Dockerfile BUILD_TIME Environment Variables
**What goes wrong:** Build-time env vars aren't available during Docker build
**Why it happens:** With Dockerfiles, vars need `docker build --build-arg`, but App Platform's bindable vars aren't passed as build args by default
**How to avoid:** Only use `scope: RUN_TIME` for our app (we don't need build-time vars), or add `ARG VAR_NAME` directives in Dockerfile
**Warning signs:** Build fails with "undefined variable" errors
**Note:** Our app only needs runtime vars, so this is not a concern

### Pitfall 5: GitHub Repository Connection During Ansible Provision
**What goes wrong:** App created via Ansible API lacks GitHub auto-deploy integration
**Why it happens:** GitHub OAuth flow requires interactive UI authorization the first time
**How to avoid:**
- Option A: First-time setup via DO console UI to authorize GitHub, then Ansible manages updates
- Option B: Use app spec with `github:` section, may require manual GitHub app installation
- Option C: Deploy from Docker registry instead of GitHub source
**Warning signs:** App deploys but doesn't auto-redeploy on git push
**Recommendation:** Use Option A - manual UI setup for GitHub connection, Ansible for everything else

### Pitfall 6: doctl Version Incompatibility
**What goes wrong:** Newer app spec fields like `health_check.port` or CORS policies fail validation
**Why it happens:** Using outdated doctl version that doesn't recognize new schema fields
**How to avoid:** Keep doctl updated (v1.61.0+ for CORS, v1.52.0+ for image sources)
**Warning signs:** `doctl apps spec validate` returns "unknown field" errors
**Note:** Only matters if using doctl for validation/deployment, Ansible uses API directly

## Code Examples

Verified patterns from official sources:

### Minimal App Spec for Dockerfile Deployment
```yaml
# Source: https://github.com/digitalocean/sample-dockerfile/blob/main/.do/app.yaml
name: three-good-sources-api
services:
  - name: api
    dockerfile_path: Dockerfile
    github:
      repo: owner/3goodsources
      branch: main
      deploy_on_push: true
```

### Complete Service Configuration with All Options
```yaml
# Source: https://docs.digitalocean.com/products/app-platform/reference/app-spec/
services:
  - name: api
    dockerfile_path: Dockerfile
    http_port: 3000
    instance_count: 1
    instance_size_slug: basic-xxs
    github:
      repo: owner/3goodsources
      branch: main
      deploy_on_push: true
    health_check:
      http_path: /health
      initial_delay_seconds: 5
      period_seconds: 10
      timeout_seconds: 1
      success_threshold: 1
      failure_threshold: 9
    envs:
      - key: REGISTRY_PATH
        value: /app/registry.json
        scope: RUN_TIME
        type: GENERAL
      - key: PKARR_SECRET_KEY
        value: "plaintext_value_on_first_create"
        scope: RUN_TIME
        type: SECRET
```

### Ansible Task: Create App
```yaml
# Source: https://docs.digitalocean.com/reference/ansible/reference/modules/app/
- name: Create App Platform application
  digitalocean.cloud.app:
    token: "{{ lookup('env', 'DIGITALOCEAN_TOKEN') }}"
    state: present
    spec:
      name: three-good-sources-api
      region: nyc
      services:
        - name: api
          dockerfile_path: Dockerfile
          github:
            repo: owner/3goodsources
            branch: main
            deploy_on_push: true
          http_port: 3000
          instance_count: 1
          instance_size_slug: basic-xxs
          health_check:
            http_path: /health
          envs:
            - key: PORT
              value: "3000"
              scope: RUN_TIME
            - key: PKARR_SECRET_KEY
              value: "{{ pkarr_secret_key }}"
              scope: RUN_TIME
              type: SECRET
    timeout: 600
  register: app_result

- name: Save app ID for future operations
  set_fact:
    do_app_id: "{{ app_result.app.id }}"
```

### Ansible Task: Update Existing App
```yaml
# Source: https://docs.digitalocean.com/reference/ansible/reference/modules/app/
- name: Update existing app by ID
  digitalocean.cloud.app:
    token: "{{ lookup('env', 'DIGITALOCEAN_TOKEN') }}"
    state: present
    id: "{{ do_app_id }}"
    spec:
      # ... updated spec ...
  register: app_update_result
```

### Ansible Task: Delete App (Rollback/Cleanup)
```yaml
# Source: https://docs.digitalocean.com/reference/ansible/reference/modules/app/
- name: Delete app by name
  digitalocean.cloud.app:
    token: "{{ lookup('env', 'DIGITALOCEAN_TOKEN') }}"
    state: absent
    name: three-good-sources-api
```

### Validation Before Deploy
```bash
# Source: https://docs.digitalocean.com/reference/doctl/reference/apps/spec/
doctl apps spec validate .do/app.yaml

# Create app from validated spec (alternative to Ansible)
doctl apps create --spec .do/app.yaml --wait

# Check app status
doctl apps list
doctl apps get <app-id>
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| community.digitalocean Ansible collection | digitalocean.cloud collection | 2024-2025 | Old collection deprecated, new one uses pydo library, more maintained |
| Manual UI-only app creation | App spec YAML (.do/app.yaml) | 2020-present | Infrastructure as code, version control, automation |
| Buildpack-only deployment | Dockerfile or buildpack | 2019-present | More deployment flexibility |
| No Rust buildpack | Native Rust buildpack | Feb 2026 | Can use buildpack instead of Dockerfile (but Dockerfile still recommended for our migration) |
| Image sources: DOCR only | DOCR, GHCR, Docker Hub | 2024-2025 | More registry flexibility |

**Deprecated/outdated:**
- **community.digitalocean collection:** Replaced by digitalocean.cloud, no longer actively maintained
- **App Platform v1 API:** Now using v2 API endpoints (https://api.digitalocean.com/v2/apps)
- **Manual secret management in UI:** Now type: SECRET in app spec (but still needs Ansible for automation)

## Open Questions

1. **GitHub Auto-Deploy Setup via API**
   - What we know: GitHub connection requires OAuth authorization
   - What's unclear: Whether Ansible can fully automate GitHub connection on first setup, or requires manual UI step
   - Recommendation: Plan for manual GitHub connection via UI first time, then Ansible manages app updates. Document this as pre-requisite step.

2. **Ansible Module Secret Handling**
   - What we know: Platform needs plaintext on first create, encrypted on updates
   - What's unclear: Whether digitalocean.cloud.app module automatically handles encrypted values on subsequent runs, or if we need to manually fetch encrypted values
   - Recommendation: Test with ansible-vault encrypted var for plaintext secret, verify module behavior on second run. May need to fetch encrypted value from platform after first provision.

3. **DO Project Assignment**
   - What we know: App can be assigned to DO project (consistent with existing infrastructure)
   - What's unclear: Whether digitalocean.cloud.app supports project assignment directly, or requires separate digitalocean.cloud.project_resource module
   - Recommendation: Check module docs for project parameter, otherwise use project_resource module as second step.

4. **Build Cache Persistence**
   - What we know: App Platform automatically caches builds
   - What's unclear: Whether Docker build cache persists between deployments, or if multi-stage build runs fully each time
   - Recommendation: Monitor first few deployment times, optimize Dockerfile if needed, but likely already optimal.

5. **Resource Scaling**
   - What we know: Can set instance_count and instance_size_slug in app spec
   - What's unclear: Whether basic-xxs matches Render's starter plan performance
   - Recommendation: Start with basic-xxs (cheapest), monitor performance, scale if needed.

## Sources

### Primary (HIGH confidence)
- [DigitalOcean App Platform App Spec Reference](https://docs.digitalocean.com/products/app-platform/reference/app-spec/) - Complete app.yaml schema
- [DigitalOcean Ansible Collection Reference](https://docs.digitalocean.com/reference/ansible/reference/) - Module documentation
- [digitalocean.cloud.app Module](https://docs.digitalocean.com/reference/ansible/reference/modules/app/) - Ansible provisioning
- [App Platform Environment Variables](https://docs.digitalocean.com/products/app-platform/how-to/use-environment-variables/) - Secret management
- [App Platform Dockerfile Reference](https://docs.digitalocean.com/products/app-platform/reference/dockerfile/) - Docker-specific configuration
- [Sample Dockerfile App Spec](https://github.com/digitalocean/sample-dockerfile/blob/main/.do/app.yaml) - Reference implementation

### Secondary (MEDIUM confidence)
- [DigitalOcean App Platform vs Render comparison](https://www.digitalocean.com/resources/articles/render-alternatives) - Platform comparison
- [App Platform Multi-Environment Best Practices](https://www.digitalocean.com/community/conceptual-articles/best-practices-app-platform-multi-environment) - Deployment patterns
- [doctl apps create documentation](https://docs.digitalocean.com/reference/doctl/reference/apps/create/) - CLI alternative to Ansible
- [App Platform Secret Best Practices](https://www.digitalocean.com/community/questions/what-are-app-spec-best-practices-for-keeping-env-secrets-secret) - Security guidance
- [Rust Buildpack Documentation](https://docs.digitalocean.com/products/app-platform/reference/buildpacks/rust/) - Alternative to Dockerfile

### Secondary (MEDIUM confidence - WebSearch verified with official sources)
- [GitHub doctl issue #1071](https://github.com/digitalocean/doctl/issues/1071) - Error handling pitfalls
- [GitHub doctl issue #1217](https://github.com/digitalocean/doctl/issues/1217) - health_check.port validation issues
- [DigitalOcean Community: Secret encryption error](https://www.digitalocean.com/community/questions/error-with-dolt-secret-env-value-must-not-be-encrypted-before-app-is-created) - Common secret pitfall

### Tertiary (LOW confidence - general Docker best practices, not DO-specific)
- [Common Dockerfile Mistakes - Atlassian](https://www.atlassian.com/blog/developer/common-dockerfile-mistakes) - General Docker pitfalls
- [Docker Container Mistakes - Virtualization Howto](https://www.virtualizationhowto.com/2025/08/10-common-docker-container-mistakes-and-how-to-avoid-them/) - Security and version pinning

## Metadata

**Confidence breakdown:**
- Standard stack: MEDIUM-HIGH - Official Ansible collection and doctl documented, but Ansible module is less commonly used than UI/CLI
- Architecture: HIGH - App spec format is well-documented with official examples, Dockerfile approach matches existing setup
- Pitfalls: MEDIUM - Most from official docs and GitHub issues, but some require validation (especially secret handling workflow)
- Ansible provisioning: MEDIUM - Module exists and is documented, but less community usage/examples than other methods

**Research date:** 2026-02-08
**Valid until:** 2026-03-08 (30 days - App Platform is stable, but Rust buildpack is very new)

**Key unknowns requiring validation during implementation:**
1. Ansible module behavior with encrypted secrets on subsequent runs
2. GitHub auto-deploy setup via API vs requiring UI step
3. Actual DO project assignment via Ansible
4. Performance comparison of basic-xxs vs Render starter
