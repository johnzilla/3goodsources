# Phase 11: DNS Cutover & Decommission - Research

**Researched:** 2026-02-08
**Domain:** DNS migration, custom domain configuration, infrastructure decommissioning
**Confidence:** HIGH

## Summary

Phase 11 involves transitioning DNS records from the current configuration (which pointed api.3gs.ai to Render) to DigitalOcean App Platform, while ensuring the apex domain (3gs.ai) continues pointing to GitHub Pages. The DO app is already deployed and healthy at three-good-sources-api-238s5.ondigitalocean.app, making this primarily a DNS reconfiguration and cleanup phase rather than an infrastructure deployment.

The critical technical challenge is handling apex domain (3gs.ai) vs subdomain (api.3gs.ai) DNS configuration differences. Apex domains cannot use CNAME records due to RFC specifications, while subdomains can. DigitalOcean App Platform provides automatic SSL certificate provisioning via Let's Encrypt once custom domains are added, requiring no manual certificate management.

**Primary recommendation:** Use DigitalOcean's app spec domains configuration with doctl/API to add custom domains, update DNS provider CNAME for api.3gs.ai to point to DO's ondigitalocean.app hostname, lower TTLs 24-48 hours before cutover for fast propagation, verify with dig/curl before considering Render decommissioned, and remove render.yaml only after successful cutover verification.

## Standard Stack

### Core DNS Tools

| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| doctl | latest | DigitalOcean CLI for app management | Official DO CLI, used for app spec updates |
| dig | system | DNS query tool for verification | Pre-installed on Linux/macOS, standard DNS debugging |
| curl | system | HTTP client for endpoint testing | Universal tool for API/endpoint verification |
| whois | system | Domain registrar/DNS provider detection | Standard tool for domain ownership lookup |

### DigitalOcean Integration

| Tool | Version | Purpose | When to Use |
|------|---------|---------|-------------|
| digitalocean.cloud (Ansible) | latest | Ansible collection for DO automation | When automating via Ansible (project uses this) |
| digitalocean.cloud.app | module | App Platform provisioning | Already used in ansible/playbooks/provision-do.yml |
| doctl apps update | CLI | Manual app spec updates | Alternative to Ansible for one-off changes |

### DNS Verification Services

| Service | Purpose | When to Use |
|---------|---------|-------------|
| dnschecker.org | Global DNS propagation check | Verify cutover propagated worldwide |
| whatsmydns.net | Visual global DNS map | User-friendly propagation visualization |
| Google DNS (8.8.8.8) | Reliable DNS resolver for dig | Primary verification resolver |
| Cloudflare DNS (1.1.1.1) | Alternative resolver | Secondary verification |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| doctl CLI | DigitalOcean API directly | API requires manual curl/http client, doctl is simpler |
| App spec domains | Manual UI configuration | UI doesn't version control, app spec is IaC |
| Ansible automation | Manual doctl commands | Manual is faster for one-off, Ansible is repeatable |

**Installation:**
```bash
# doctl (if not already installed)
brew install doctl  # macOS
snap install doctl  # Linux

# Ansible collection (already in ansible/requirements.yml)
ansible-galaxy collection install digitalocean.cloud
```

## Architecture Patterns

### Recommended DNS Cutover Flow

```
1. Pre-Cutover (24-48 hours before)
   ├── Lower TTLs on existing DNS records to 300-600s
   ├── Verify DO app health at ondigitalocean.app URL
   └── Document current DNS state

2. Domain Configuration in DO
   ├── Update .do/app.yaml with domains section
   ├── Apply via ansible-playbook or doctl apps update
   └── DigitalOcean provisions Let's Encrypt certs automatically

3. DNS Updates at Provider
   ├── api.3gs.ai: Update CNAME to point to DO app hostname
   ├── 3gs.ai: No change (continues pointing to GitHub Pages)
   └── Monitor DNS propagation globally

4. Verification Phase
   ├── Check DNS resolution: dig @8.8.8.8 api.3gs.ai
   ├── Test HTTPS endpoint: curl https://api.3gs.ai/health
   ├── Verify SSL cert from Let's Encrypt
   └── Monitor for 24-48 hours

5. Decommission
   ├── Remove render.yaml from git
   ├── Update DNS-SETUP.md to remove Render references
   ├── Verify git history clean of secrets
   └── Restore TTLs to normal values (3600s+)
```

### Pattern 1: App Spec Domain Configuration

**What:** Declarative domain specification in .do/app.yaml

**When to use:** All DigitalOcean App Platform custom domain setup

**Example:**
```yaml
# .do/app.yaml
name: three-good-sources-api
region: nyc
domains:
  - domain: api.3gs.ai
    type: PRIMARY
    minimum_tls_version: "1.2"
  - domain: 3gs.ai
    type: ALIAS
    minimum_tls_version: "1.2"
services:
  - name: api
    dockerfile_path: Dockerfile
    # ... rest of service config
```

**Domain types:**
- `DEFAULT`: The auto-assigned .ondigitalocean.app domain
- `PRIMARY`: The primary custom domain shown in DO control panel
- `ALIAS`: Additional custom domains (non-primary)

**Source:** [DigitalOcean App Spec Reference](https://docs.digitalocean.com/products/app-platform/reference/app-spec/)

### Pattern 2: Ansible Automation for Domain Updates

**What:** Use existing Ansible playbook to apply app spec changes

**When to use:** When maintaining infrastructure-as-code consistency

**Example:**
```yaml
# ansible/playbooks/provision-do.yml (already exists)
- name: Provision DigitalOcean App Platform application
  hosts: localhost
  connection: local
  vars_files:
    - ../vars/do-secrets.yml
  tasks:
    - name: Load app spec from file
      ansible.builtin.slurp:
        src: "{{ playbook_dir }}/../../.do/app.yaml"
      register: app_spec_file

    - name: Parse app spec YAML
      ansible.builtin.set_fact:
        app_spec: "{{ app_spec_file.content | b64decode | from_yaml }}"

    - name: Create or update App Platform application
      digitalocean.cloud.app:
        token: "{{ digitalocean_token }}"
        state: present
        spec: "{{ app_spec }}"
        timeout: 600
      register: app_result
```

**Advantage:** This pattern is already established in the project. Adding domains is just updating .do/app.yaml and re-running the playbook.

### Pattern 3: TTL Reduction Strategy

**What:** Gradually lower DNS TTL before cutover for fast propagation

**When to use:** All production DNS cutovers to minimize downtime/propagation delays

**Timeline:**
```
7 days before:   Audit current TTL values (baseline)
3 days before:   Lower TTL to 3600s (1 hour) if higher
24 hours before: Lower TTL to 300s (5 minutes)
During cutover:  Make DNS changes (old TTL expired, new TTL active)
7 days after:    Raise TTL back to 3600s or higher
```

**Why it works:** DNS resolvers can only respect the new TTL after querying again. You must lower TTL at least one full previous-TTL period before cutover. If TTL is 24 hours, lower it 24+ hours in advance.

**Source:** [DNS TTL Best Practices](https://www.digicert.com/blog/long-short-ttls)

### Pattern 4: Apex Domain Handling

**What:** Apex domains (3gs.ai) cannot use CNAME records per RFC specification

**When to use:** Any apex domain DNS configuration

**Options:**
1. **A records** (current for GitHub Pages): Four A records pointing to GitHub IPs
2. **ALIAS/ANAME records**: Provider-specific CNAME-like behavior (Cloudflare, Route53)
3. **CNAME flattening**: Provider resolves CNAME behind the scenes, serves A record

**Current project state:**
- `3gs.ai` → GitHub Pages via A records (UNCHANGED in this phase)
- `api.3gs.ai` → Currently Render CNAME, will become DO CNAME

**Source:** [CNAME at Apex Guide](https://www.dchost.com/blog/en/cname-at-the-apex-the-friendly-guide-to-alias-aname-and-cloudflare-cname-flattening/)

### Pattern 5: Rollback Strategy

**What:** DNS-based rollback by reverting CNAME changes

**When to use:** If cutover encounters issues (SSL fails, app unreachable, errors)

**Implementation:**
```bash
# Rollback: Change CNAME back to Render
# At DNS provider, update:
# api.3gs.ai CNAME three-good-sources-api.onrender.com

# With low TTL (300s), rollback propagates in 5-10 minutes
```

**Critical:** Keep Render deployment running until cutover fully verified (24-48 hours). Only decommission after confidence in DO deployment.

**Source:** [DNS Cutover Rollback Strategies](https://docs.aws.amazon.com/prescriptive-guidance/latest/best-practices-migration-cutover/cutover-stage.html)

### Anti-Patterns to Avoid

- **Lowering TTL immediately before cutover:** Must lower TTL at least one full previous-TTL period before cutover. If TTL is 86400s (24h), lower it 24+ hours in advance.
- **Removing Render before DNS propagation completes:** Keep Render running for 24-48 hours post-cutover for rollback capability
- **Not verifying SSL certificate issuance:** DO provisions Let's Encrypt certs automatically, but takes 5-30 minutes. Verify HTTPS works before considering cutover complete.
- **Committing render.yaml removal before verification:** Git commit removal only after verifying DO deployment stable for 24+ hours
- **Using CNAME at apex domain:** Apex domains cannot use CNAME per RFC. Use A records, ALIAS, or CNAME flattening instead.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| DNS propagation checking | Custom polling scripts | dig with explicit resolver (@8.8.8.8), dnschecker.org | Global DNS has many layers (authoritative, recursive, caching). Standard tools account for this. |
| SSL certificate provisioning | Manual Let's Encrypt setup | DigitalOcean automatic SSL | DO provisions and renews Let's Encrypt certs automatically for custom domains |
| Global DNS testing | Scripts pinging multiple regions | whatsmydns.net, dnschecker.org | These services maintain worldwide DNS resolver networks |
| Git secret scanning | grep for specific patterns | BFG Repo-Cleaner, git-filter-repo | Secrets hide in patches, merge commits, refs. Tools handle edge cases. |
| WHOIS domain lookup | Parsing raw WHOIS output | whois command or whois.com API | WHOIS format varies by TLD. Existing tools normalize output. |

**Key insight:** DNS is globally distributed with caching at multiple levels (authoritative servers, recursive resolvers, local caches). Propagation isn't instant—use low TTLs and standard verification tools to minimize cutover risk.

## Common Pitfalls

### Pitfall 1: DNS Provider Unknown

**What goes wrong:** Cannot configure DNS records without knowing which provider manages 3gs.ai domain

**Why it happens:** Domain registrar and DNS provider can be different services. Registration might be at one provider (Namecheap, GoDaddy) while DNS managed elsewhere (Cloudflare, Route53).

**How to avoid:**
```bash
# Find current DNS provider
whois 3gs.ai | grep "Name Server"

# Example output shows authoritative nameservers:
# Name Server: ns1.digitalocean.com
# Name Server: ns2.digitalocean.com
# (indicates DNS managed by DigitalOcean)
```

**Warning signs:** User doesn't immediately know where to update DNS records. Must investigate before planning specific DNS update steps.

**Source:** [WHOIS Lookup](https://www.whois.com/whois/)

### Pitfall 2: SSL Certificate Provisioning Delay

**What goes wrong:** DNS cutover completes, but HTTPS returns certificate errors for hours

**Why it happens:** Let's Encrypt certificate issuance requires DNS propagation + DNS-01 or HTTP-01 challenge completion. DO automates this, but takes 5-30 minutes after domain added.

**How to avoid:**
1. Add domains to DO app spec BEFORE DNS cutover
2. Let DO attempt cert provisioning (will fail initially, no DNS pointing yet)
3. Update DNS records to point to DO
4. Wait 5-30 minutes for DNS propagation + cert issuance
5. Verify HTTPS works before declaring cutover complete

**Warning signs:**
- `curl https://api.3gs.ai` returns "certificate not valid for this domain"
- Browser shows "Your connection is not private"
- DO dashboard shows domain in "Pending certificate" state

**Source:** [DigitalOcean SSL Documentation](https://docs.digitalocean.com/products/app-platform/how-to/manage-domains/)

### Pitfall 3: CAA Record Blocking Let's Encrypt

**What goes wrong:** DO attempts SSL provisioning but fails silently due to CAA records

**Why it happens:** Certificate Authority Authorization (CAA) DNS records restrict which CAs can issue certificates. If CAA records exist and don't include Let's Encrypt, issuance fails.

**How to avoid:**
```bash
# Check for CAA records
dig @8.8.8.8 3gs.ai CAA

# If CAA records exist, must include both:
# 3gs.ai. CAA 0 issue "letsencrypt.org"
# 3gs.ai. CAA 0 issue "pki.goog"
```

**Warning signs:** DNS and app configuration correct, but SSL cert never provisions. DO dashboard may show "Certificate provisioning failed."

**Source:** [DigitalOcean CAA Requirements](https://docs.digitalocean.com/products/app-platform/how-to/manage-domains/)

### Pitfall 4: DNSSEC Incompatibility

**What goes wrong:** Domain added to DO app, but configuration fails with cryptic error

**Why it happens:** DigitalOcean App Platform does not support DNSSEC-enabled domains

**How to avoid:**
```bash
# Check if DNSSEC enabled
dig @8.8.8.8 3gs.ai +dnssec | grep "ad"
# If "ad" flag present, DNSSEC is active

# Must disable DNSSEC at DNS provider before adding to DO App Platform
```

**Warning signs:** DO returns error when adding domain: "Domain uses DNSSEC which is not supported"

**Source:** [DigitalOcean Domain Management](https://docs.digitalocean.com/products/app-platform/how-to/manage-domains/)

### Pitfall 5: Git History Not Actually Clean

**What goes wrong:** Believe git history is clean of secrets, but secrets still exist in reachable commits

**Why it happens:** Deleting a file or removing content doesn't remove it from git history. Secrets remain in old commits, branches, tags, reflog.

**How to avoid:**
```bash
# Comprehensive secret scan
git log --all --full-history --source --find-copies --find-renames -S "dop_v1_" -- "**/*.yml" "**/*.yaml" "**/*.json"

# Check reflog too
git reflog | grep -i "secret\|token\|password"

# If secrets found, use BFG Repo-Cleaner or git-filter-repo
# Then force push (destructive, coordinate with team)
```

**Warning signs:**
- Secret mentioned in old commit messages
- File existed in past commits before being gitignored
- Environment variable with token value committed in old config

**Source:** [Removing Secrets from Git History](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/removing-sensitive-data-from-a-repository)

### Pitfall 6: Not Testing SSL Certificate Chain

**What goes wrong:** HTTPS works in browser, but API clients fail with "SSL certificate problem"

**Why it happens:** Browser may trust incomplete certificate chains, but curl/API clients are stricter

**How to avoid:**
```bash
# Test SSL certificate chain completeness
curl -v https://api.3gs.ai/health 2>&1 | grep -E "SSL|certificate|verify"

# Should see:
# * SSL connection using TLSv1.3
# * Server certificate:
# *  subject: CN=api.3gs.ai
# *  issuer: C=US; O=Let's Encrypt; CN=R3
# *  SSL certificate verify ok.

# If fails, certificate chain incomplete or wrong cert installed
```

**Warning signs:** Browser works, but `curl` returns error code 60 (SSL certificate problem)

**Source:** [Testing SSL/TLS](https://kb.hosting.com/docs/troubleshooting-dns-with-dig-and-nslookup)

### Pitfall 7: Render Decommissioned Too Early

**What goes wrong:** Render deleted during DNS cutover, rollback becomes impossible

**Why it happens:** DNS propagation takes 24-72 hours globally. Some users still hitting old CNAME for hours/days.

**How to avoid:**
- Keep Render running for 24-48 hours after DNS cutover
- Monitor Render logs for incoming requests (should taper off)
- Only decommission after global DNS propagation verified AND no Render traffic
- Set calendar reminder: "DO NOT delete Render until [date + 48 hours]"

**Warning signs:**
- Some users report API works, others report errors
- Global DNS checkers show mixed results (some old, some new)
- Render still receiving HTTP requests hours after cutover

**Source:** [DNS Migration Best Practices](https://www.dchost.com/blog/en/domain-and-dns-migration-checklist-when-changing-hosting-provider/)

## Code Examples

Verified patterns from official sources:

### DNS Verification Commands

```bash
# Check current DNS resolution for apex domain (should be GitHub Pages)
dig @8.8.8.8 3gs.ai A +short
# Expected: 185.199.108.153, 185.199.109.153, 185.199.110.153, 185.199.111.153

# Check current DNS resolution for API subdomain (currently Render, will be DO)
dig @8.8.8.8 api.3gs.ai CNAME +short
# Before cutover: three-good-sources-api.onrender.com
# After cutover: three-good-sources-api-238s5.ondigitalocean.app

# Verify from multiple resolvers (global propagation check)
for resolver in 8.8.8.8 1.1.1.1 9.9.9.9; do
  echo "Resolver: $resolver"
  dig @$resolver api.3gs.ai CNAME +short
done

# Check TTL (current caching duration)
dig @8.8.8.8 api.3gs.ai | grep "^api.3gs.ai"
# Example: api.3gs.ai.  3600  IN  CNAME  three-good-sources-api.onrender.com.
#                       ^TTL (seconds)
```

### HTTPS Endpoint Verification

```bash
# Test health endpoint with full TLS verification
curl -v https://api.3gs.ai/health

# Expected output includes:
# * SSL connection using TLSv1.3 / TLS_AES_256_GCM_SHA384
# * Server certificate:
# *  subject: CN=api.3gs.ai
# *  issuer: CN=R3,O=Let's Encrypt,C=US
# * SSL certificate verify ok.
# < HTTP/2 200
# {"pubkey":"...","status":"ok","version":"0.1.0"}

# Test MCP endpoint (should also work)
curl -v -X POST https://api.3gs.ai/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}},"id":1}'
```

### Update App Spec with Domains

```bash
# Option 1: Via Ansible (recommended for this project)
cd /home/john/projects/github.com/3goodsources

# Edit .do/app.yaml to add domains section
# Then run:
ansible-playbook ansible/playbooks/provision-do.yml

# Option 2: Via doctl CLI (alternative)
# Get current app spec
doctl apps spec get <app-id> > app-spec.yaml

# Edit app-spec.yaml to add domains
# Then update:
doctl apps update <app-id> --spec app-spec.yaml

# Option 3: Via doctl with direct file reference
doctl apps update <app-id> --spec .do/app.yaml
```

**Source:** [DigitalOcean doctl Reference](https://docs.digitalocean.com/reference/doctl/reference/apps/)

### Verify SSL Certificate Details

```bash
# Extract certificate information
echo | openssl s_client -connect api.3gs.ai:443 -servername api.3gs.ai 2>/dev/null | openssl x509 -noout -text

# Key fields to verify:
# - Issuer: CN=R3, O=Let's Encrypt, C=US
# - Subject: CN=api.3gs.ai
# - Subject Alternative Name: DNS:api.3gs.ai
# - Not After: [expiry date - should be ~90 days from issuance]

# Quick check for Let's Encrypt issuer
echo | openssl s_client -connect api.3gs.ai:443 -servername api.3gs.ai 2>/dev/null | openssl x509 -noout -issuer | grep "Let's Encrypt"
```

### Git Secret Verification

```bash
# Search entire git history for DO API tokens
git log --all --full-history -S "dop_v1_" --source

# Search for other secret patterns
git log --all --full-history -p | grep -E "(secret|token|password|api_key)" -i

# Check if secrets file was ever committed
git log --all --full-history -- ansible/vars/do-secrets.yml

# Verify gitignore is working
git check-ignore -v ansible/vars/do-secrets.yml
# Expected: .gitignore:X:ansible/vars/do-secrets.yml  ansible/vars/do-secrets.yml
```

**Source:** [GitHub Secret Removal Guide](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/removing-sensitive-data-from-a-repository)

### WHOIS Domain Provider Lookup

```bash
# Find DNS provider for 3gs.ai
whois 3gs.ai | grep -i "name server"

# Example outputs and what they mean:
# ns1.digitalocean.com → DNS managed by DigitalOcean
# ns-cloudflare.com → DNS managed by Cloudflare
# ns1.namecheap.com → DNS managed by Namecheap
# ns-aws.amazon.com → DNS managed by Route53

# Alternative: Use DNS query to find authoritative nameservers
dig 3gs.ai NS +short
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual SSL certificate management | Automatic Let's Encrypt via platform | 2020+ (widespread adoption) | Zero manual cert work, automatic renewal |
| High TTL (86400s) for all records | Low TTL (300s) during cutover only | Best practice since ~2015 | Fast DNS changes with minimal query overhead |
| git filter-branch for secret removal | BFG Repo-Cleaner or git-filter-repo | 2018+ (git-filter-branch deprecated) | 10-100x faster, safer |
| Manual DNS record updates | Infrastructure-as-code (app spec YAML) | 2020+ (GitOps era) | Version control, repeatability, audit trail |
| Blue/green deployment at infrastructure level | Blue/green at DNS level | Ongoing (both valid) | DNS-level allows instant rollback, but propagation delay |

**Deprecated/outdated:**
- **git filter-branch**: Deprecated by Git project in favor of git-filter-repo (slower, more error-prone)
- **Manual Let's Encrypt certbot on servers**: Superseded by automatic platform-managed certs (DO, Cloudflare, etc.)
- **Static TTL values**: Modern practice is dynamic TTL (low during changes, high during stability)

## Open Questions

1. **Which DNS provider manages 3gs.ai?**
   - What we know: DNS-SETUP.md was created for 3gs.ai, but doesn't specify provider
   - What's unclear: Namecheap? Cloudflare? Route53? DigitalOcean DNS?
   - Recommendation: Run `whois 3gs.ai` to identify provider, include in plan. Different providers have different UIs/APIs for DNS updates.

2. **Should both 3gs.ai and api.3gs.ai be added to DO app spec?**
   - What we know: 3gs.ai currently points to GitHub Pages (landing page), api.3gs.ai should point to DO (MCP API)
   - What's unclear: Should 3gs.ai also be added as ALIAS in DO app spec, or leave it pointing to GitHub?
   - Recommendation: Add ONLY api.3gs.ai to DO app spec. Keep 3gs.ai pointing to GitHub Pages. If future requirement is to serve landing page from DO, migrate later.

3. **Should Ansible automate DNS record updates or just app spec?**
   - What we know: Project uses Ansible for DO provisioning. digitalocean.cloud collection has domain_record module.
   - What's unclear: If DNS is managed by non-DO provider, Ansible can't automate DNS updates. If DNS is DO-managed, automation possible.
   - Recommendation: If DNS provider is DigitalOcean, add Ansible task for DNS automation. Otherwise, document manual steps at specific provider.

4. **Does render.yaml need to be removed from git history or just current files?**
   - What we know: render.yaml exists in current repo, contains no secrets (only structure)
   - What's unclear: Is presence in git history a problem, or just remove from current state?
   - Recommendation: Remove from current files only (git rm). No need for history rewrite—file contains no secrets. Update .gitignore to prevent re-adding.

5. **Should Render dashboard be accessed for explicit decommission, or is deletion sufficient?**
   - What we know: User already decommissioned Render during Phase 10
   - What's unclear: Was this a UI deletion, or do Render resources still exist?
   - Recommendation: Verify Render dashboard shows zero active services. If any resources remain, explicitly delete via Render UI. Document final state.

## Sources

### Primary (HIGH confidence)

**DigitalOcean Official Documentation:**
- [How to Manage Domains in App Platform](https://docs.digitalocean.com/products/app-platform/how-to/manage-domains/) - Domain setup process, DNS configuration
- [App Spec Reference](https://docs.digitalocean.com/products/app-platform/reference/app-spec/) - YAML domain configuration structure
- [doctl apps Reference](https://docs.digitalocean.com/reference/doctl/reference/apps/) - CLI commands for app updates
- [Ansible domain_record Module](https://docs.digitalocean.com/reference/ansible/reference/modules/domain_record/) - Ansible DNS automation

**Git & GitHub Documentation:**
- [Removing Sensitive Data from Repository](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/removing-sensitive-data-from-a-repository) - Git secret removal

**DNS & TLS Standards:**
- [DNS TTL Best Practices (DigiCert)](https://www.digicert.com/blog/long-short-ttls) - TTL management for cutovers
- [DNS CNAME at Apex Guide](https://www.dchost.com/blog/en/cname-at-the-apex-the-friendly-guide-to-alias-aname-and-cloudflare-cname-flattening/) - Apex domain limitations

### Secondary (MEDIUM confidence)

**Migration Best Practices:**
- [Domain and DNS Migration Checklist (DCHost)](https://www.dchost.com/blog/en/domain-and-dns-migration-checklist-when-changing-hosting-provider/) - Cutover planning
- [AWS Cutover Best Practices](https://docs.aws.amazon.com/prescriptive-guidance/latest/best-practices-migration-cutover/cutover-stage.html) - Rollback strategies
- [Zero-Downtime Migration Guide](https://unihost.com/blog/zero-downtime-migration/) - DNS sync and cutover checklist

**DNS Tools & Verification:**
- [DNSChecker.org](https://dnschecker.org/) - Global DNS propagation testing
- [What's My DNS](https://www.whatsmydns.net/) - Visual DNS propagation map
- [WHOIS Lookup Services](https://www.whois.com/whois/) - Domain provider identification

**Ansible & Automation:**
- [DigitalOcean Ansible Collection](https://github.com/digitalocean/ansible-collection) - Official collection repository
- [Community DigitalOcean Docs](https://docs.ansible.com/ansible/latest/collections/community/digitalocean/index.html) - Alternative collection

### Tertiary (LOW confidence)

**Web Search Results:**
- Various blog posts and tutorials on DNS cutover (useful for patterns, not authoritative)
- Community forum discussions on DigitalOcean App Platform domain setup (anecdotal experiences)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Official DigitalOcean docs, established tools (dig, curl, whois)
- Architecture: HIGH - DO docs provide exact app spec format, TTL practices well-documented
- Pitfalls: MEDIUM - Combination of official docs (CAA, DNSSEC) and community experience (timing, rollback)

**Research date:** 2026-02-08
**Valid until:** 2026-04-08 (60 days - DNS/platform features stable, but verify for major DO platform updates)

**Key unknowns requiring user input:**
1. DNS provider for 3gs.ai (affects implementation details)
2. Whether to add both domains to DO app or only api.3gs.ai
3. Render decommission status verification
