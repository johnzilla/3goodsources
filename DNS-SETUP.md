# DNS Setup for 3GS Deployment

This document provides the exact DNS records needed to deploy 3GS with:
- **3gs.ai** - Static landing page hosted on GitHub Pages
- **api.3gs.ai** - Rust MCP server hosted on Render

## Overview

Two domains to configure:
1. **Apex domain (3gs.ai)** - Routes to GitHub Pages for the static landing page
2. **API subdomain (api.3gs.ai)** - Routes to Render for the live MCP server

## DNS Records

### Apex Domain: 3gs.ai → GitHub Pages

Create **four A records** pointing to GitHub Pages IP addresses:

| Type | Name   | Value             | TTL  |
|------|--------|-------------------|------|
| A    | 3gs.ai | 185.199.108.153   | 3600 |
| A    | 3gs.ai | 185.199.109.153   | 3600 |
| A    | 3gs.ai | 185.199.110.153   | 3600 |
| A    | 3gs.ai | 185.199.111.153   | 3600 |

These are the official GitHub Pages IPv4 addresses. All four records are required for high availability and load distribution.

### API Subdomain: api.3gs.ai → Render

Create **one CNAME record** pointing to the Render hostname:

| Type  | Name        | Value                                  | TTL  |
|-------|-------------|----------------------------------------|------|
| CNAME | api.3gs.ai  | three-good-sources-api.onrender.com    | 3600 |

Replace `three-good-sources-api.onrender.com` with your actual Render service hostname if different. You can find this in your Render dashboard under your service settings.

## Verification Steps

After creating DNS records, wait 30-60 minutes for propagation, then verify:

### 1. Verify apex domain resolves to GitHub Pages

```bash
dig @8.8.8.8 3gs.ai A
```

Expected output should include all four GitHub Pages IPs:
- 185.199.108.153
- 185.199.109.153
- 185.199.110.153
- 185.199.111.153

### 2. Verify API subdomain resolves to Render

```bash
dig @8.8.8.8 api.3gs.ai CNAME
```

Expected output should show `three-good-sources-api.onrender.com` (or your Render hostname).

### 3. Check HTTPS access

Once DNS propagates and certificates are issued:

```bash
curl https://3gs.ai
curl https://api.3gs.ai/health
```

Both should return content without TLS errors.

## GitHub Pages Setup

After DNS records are created and propagated:

1. Go to your GitHub repository settings: **Settings → Pages**
2. Configure source:
   - **Source:** Deploy from a branch
   - **Branch:** main
   - **Folder:** /docs
3. Set custom domain:
   - **Custom domain:** 3gs.ai
   - Click **Save**
4. Wait for DNS check to pass (GitHub verifies DNS records)
5. Enable **Enforce HTTPS** checkbox (after DNS verification completes)

GitHub will automatically provision a Let's Encrypt TLS certificate for 3gs.ai.

## Render Custom Domain Setup

After DNS records are created and propagated:

1. Go to your Render dashboard
2. Select your service (three-good-sources-api)
3. Go to **Settings → Custom Domains**
4. Click **Add Custom Domain**
5. Enter: `api.3gs.ai`
6. Click **Save**

Render will automatically:
- Verify DNS CNAME record
- Provision a Let's Encrypt TLS certificate
- Enable HTTPS for api.3gs.ai

This process typically takes 5-15 minutes after DNS propagation.

## Troubleshooting

### DNS propagation delays

DNS changes can take up to 48 hours to propagate globally, though it's typically much faster (30-60 minutes). You can check propagation status with:

```bash
# Check from Google DNS
dig @8.8.8.8 3gs.ai A
dig @8.8.8.8 api.3gs.ai CNAME

# Check from Cloudflare DNS
dig @1.1.1.1 3gs.ai A
dig @1.1.1.1 api.3gs.ai CNAME
```

If different DNS servers return different results, propagation is still in progress.

### GitHub Pages verification fails

If GitHub Pages shows "DNS check unsuccessful":

1. Verify all four A records exist with correct IPs
2. Wait 10-15 minutes and try again
3. Remove and re-add the custom domain in GitHub settings
4. Check for CNAME conflicts (only one CNAME record should exist in docs/CNAME)

### Render domain verification fails

If Render shows "DNS verification failed":

1. Verify CNAME record exists: `dig api.3gs.ai CNAME`
2. Ensure CNAME value matches your Render hostname exactly
3. Wait for DNS propagation (can take up to 1 hour)
4. Remove and re-add the custom domain in Render dashboard

### HTTPS not working

If HTTPS shows certificate errors after setup:

- **GitHub Pages:** Can take 1-2 hours to provision certificate after DNS verification
- **Render:** Can take 5-15 minutes to provision certificate after domain verification
- Both use Let's Encrypt with automated renewal
- Check certificate status in respective dashboards

## DNS Provider Notes

### Common providers

The exact interface varies by provider, but all DNS providers support A and CNAME records:

- **Cloudflare:** DNS → Records → Add record
- **Namecheap:** Advanced DNS → Add New Record
- **Google Domains:** DNS → Custom records
- **Route 53 (AWS):** Hosted zones → Create record

### Cloudflare proxy warning

If using Cloudflare, ensure the DNS records are **DNS only** (gray cloud), not proxied (orange cloud), during initial setup. You can enable Cloudflare proxy after GitHub Pages and Render TLS certificates are issued.

## Summary Checklist

- [ ] Create 4 A records for 3gs.ai → GitHub Pages IPs
- [ ] Create 1 CNAME record for api.3gs.ai → Render hostname
- [ ] Wait 30-60 minutes for DNS propagation
- [ ] Verify DNS with dig commands
- [ ] Enable GitHub Pages with source: main branch, /docs folder
- [ ] Set GitHub Pages custom domain to 3gs.ai
- [ ] Wait for GitHub DNS verification to pass
- [ ] Enable Enforce HTTPS on GitHub Pages
- [ ] Add api.3gs.ai as custom domain in Render dashboard
- [ ] Wait for Render DNS verification and TLS provisioning
- [ ] Test HTTPS access to both 3gs.ai and api.3gs.ai

Once all steps complete, the landing page will be live at https://3gs.ai and the API will be accessible at https://api.3gs.ai.
