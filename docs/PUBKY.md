# Identity & Verification (PKARR/Pubky)

## Overview

3GS uses cryptographic identity to prove who operates a registry server. This enables trust, verification, and future federation where multiple curators vouch for each other's quality.

This document explains:
- What PKARR is and why it matters
- How 3GS uses PKARR for server identity
- Step-by-step verification guide with curl commands
- The broader Pubky ecosystem
- Future vision for federated trust and endorsements

## What Is PKARR?

**PKARR** stands for **Public Key Addressable Resource Records**. It's a protocol for self-sovereign identity using public key cryptography.

### Core Concepts

At its heart, PKARR uses **Ed25519 keypairs** for identity:
- A **secret key** (32 bytes, kept private) proves you control the identity
- A **public key** (32 bytes, shared publicly) is the identity itself
- No central authority - the keypair IS the identity
- Anyone can verify signatures made with the corresponding secret key

### Key Format

PKARR public keys are encoded in **z-base-32**, a human-readable base-32 encoding:
- Produces a 52-character string
- Uses only lowercase letters and numbers
- Easy to share, copy-paste, and display
- Example: `o4dksfbqk85ogzdb5osziw6befigbuxmuxkuxq8434q89uj56uyy`

### Why PKARR for 3GS?

Traditional identity relies on:
- **Domain names** - Controlled by registrars, subject to seizure/expiration
- **Certificate authorities** - Centralized trust, expensive, bureaucratic
- **Usernames on platforms** - Controlled by the platform, can be banned

PKARR identity is:
- **Self-sovereign** - Only the keypair holder can sign as that identity
- **Permanent** - The public key never expires or gets revoked by a third party
- **Verifiable** - Anyone can verify signatures without asking permission
- **Portable** - Same identity works across protocols and servers

For 3GS, this means a curator's identity is **independent of infrastructure**. Even if the domain changes, the cryptographic identity persists.

## How 3GS Uses PKARR

The 3GS server generates or loads a PKARR keypair on startup and uses it to prove operator identity.

### Startup Sequence

1. **Check for PKARR_SECRET_KEY environment variable**
   - If set: Decode 64-character hex string → Load keypair from secret key
   - If not set: Generate random ephemeral keypair (logs warning)

2. **Store public key in application state**
   - Public key is extracted and stored as z-base-32 string
   - Shared across HTTP handlers for provenance responses

3. **Expose public key via /health endpoint**
   - JSON response includes `pubkey` field
   - Quick verification that server is running with expected identity

4. **Include public key in MCP get_provenance responses**
   - Agents querying provenance learn the curator's identity
   - Future: Could verify endorsements, trust graphs, signature chains

### Persistent vs. Ephemeral Identity

**Without PKARR_SECRET_KEY:**
- Server generates random keypair each time it starts
- Public key changes on every restart
- Fine for testing, **bad for production** (identity not stable)
- Server logs warning: "PKARR_SECRET_KEY not set, generating ephemeral keypair"

**With PKARR_SECRET_KEY:**
- Server loads the same keypair deterministically from the secret
- Public key stays consistent across restarts
- **Required for production** where identity continuity matters
- Secret must be kept secure (environment variable, not committed to git)

### Generating a Secret Key

To create a persistent identity:

1. Generate a random 32-byte secret key (use a cryptographically secure random source)
2. Hex-encode it (produces 64-character string)
3. Set `PKARR_SECRET_KEY=<hex-string>` in your environment

**Example (Linux/macOS):**
```bash
# Generate 32 random bytes and hex-encode
openssl rand -hex 32

# Output: a1b2c3d4e5f6...0123456789abcdef (64 chars)
```

**Store securely:**
- In `.env` file (excluded from git via .gitignore)
- In environment variables on production platform (e.g., DigitalOcean App Platform)
- Never commit to version control

## Verification Guide

Here's how to verify the cryptographic identity of a 3GS server using curl commands.

### Step 1: Get the Server's Public Key

Query the `/health` endpoint:

```bash
curl -s http://localhost:3000/health | jq .
```

**Expected response:**
```json
{
  "status": "ok",
  "version": "0.1.0",
  "pubkey": "o4dksfbqk85ogzdb5osziw6befigbuxmuxkuxq8434q89uj56uyy"
}
```

The `pubkey` field is the server's PKARR public key in z-base-32 format. **Save this value** for comparison in Step 3.

### Step 2: Call get_provenance via MCP

The MCP protocol requires initialization before tool calls. Here's the full sequence:

**Initialize MCP session:**
```bash
curl -s -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "2025-11-25",
      "capabilities": {},
      "clientInfo": {"name": "curl", "version": "1.0"}
    }
  }' | jq .
```

**Call get_provenance tool:**
```bash
curl -s -X POST http://localhost:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
      "name": "get_provenance",
      "arguments": {}
    }
  }' | jq .
```

**Expected response:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Curator: 3GS Curator\nPubkey: o4dksfbqk85ogzdb5osziw6befigbuxmuxkuxq8434q89uj56uyy\nRegistry Version: 0.1.0\nLast Updated: 2026-02-01"
      }
    ]
  }
}
```

The `Pubkey:` line contains the server's public key.

### Step 3: Verify Consistency

Compare the pubkey from `/health` (Step 1) with the pubkey from `get_provenance` (Step 2):

```bash
# They should match exactly
```

**If they match:**
- The server is consistently reporting the same identity
- The server operator holds the corresponding private key
- The identity is stable (assuming PKARR_SECRET_KEY is set)

**If they don't match:**
- Server configuration error (very unlikely)
- Or you're querying different server instances (load balancer?)

### What This Proves

✅ **The server holds the private key** corresponding to the advertised public key

✅ **If the same pubkey appears across sessions**, it's the same operator (persistent identity)

✅ **The pubkey can be shared out-of-band** (e.g., on a website, in a trust list) for independent verification

❌ **Does NOT prove:** The curator is trustworthy, the sources are high-quality, or anything about reputation - only identity

### Future: Signature Verification

Currently, 3GS doesn't sign data with the PKARR key. Future versions could:

1. **Sign registry.json** with the secret key
2. **Include signature in responses**
3. **Clients verify signature** matches the advertised public key
4. **Proof of authenticity:** Registry hasn't been tampered with by a MITM

This would require:
- Canonicalization of JSON (consistent serialization)
- Signature generation in server startup
- Signature field in MCP responses
- Client-side Ed25519 verification logic

Not implemented in v1, but the keypair infrastructure is in place.

## What Is Pubky?

**Pubky** is a broader ecosystem built on top of PKARR for decentralized web applications.

### Beyond Just Keypairs

While 3GS v1 uses only the PKARR **cryptographic primitives** (Ed25519 keypairs via the `pkarr` Rust crate), the full Pubky ecosystem includes:

- **Homeservers:** Personal data servers keyed by PKARR public keys
- **Pubky URIs:** Addressing scheme `pubky://<pubkey>/path/to/data`
- **Pkarr DHT:** Mainline DHT for publishing DNS records keyed by public key
- **Decentralized DNS:** No ICANN, no registrars - your pubkey IS your domain
- **Portable data:** Your data lives at your homeserver, independent of applications

### Why 3GS Doesn't Use Full Pubky (Yet)

**Design decision:** Keep v1 simple and local-first.

- **No dependency on homeservers:** Registry is just a local JSON file
- **No DHT queries:** PKARR keypair is local-only, not published
- **No network calls for identity:** Just Ed25519 crypto, no infrastructure
- **Easier to understand:** Cryptographic identity without distributed systems complexity

**Future (v2+):** Could leverage Pubky homeservers for:
- Publishing registry.json at `pubky://<curator-pubkey>/registry.json`
- Fetching other curators' registries via Pubky URIs
- Building a decentralized network of curators

### Learn More

- **Pubky Core Repository:** https://github.com/pubky/pubky-core
- **PKARR Specification:** https://github.com/Nuhvi/pkarr
- **Pubky Developer Docs:** https://pubky.github.io/pubky-core/

## Future: Federated Trust

The long-term vision for 3GS is a **network of curators**, each maintaining their own registry, vouching for each other's quality through cryptographic endorsements.

### The Endorsement Concept

An **endorsement** is a statement of trust:

```
Curator A (pubkey: abc123...) endorses Curator B (pubkey: xyz789...)
Signed by: Curator A's secret key
Timestamp: 2026-02-03T12:00:00Z
```

This is a **verifiable claim**: Anyone can check the signature and confirm that Curator A really endorsed Curator B.

### How It Would Work

1. **Each curator maintains their own registry.json**
   - Different expertise, different source selections
   - Each signed with their own PKARR key

2. **Curators endorse each other**
   - Security expert endorses another security curator
   - Rust expert endorses Rust learning resources curator
   - Endorsements stored in `endorsements` array in registry.json

3. **Agents traverse the trust graph**
   - User trusts Curator A
   - Curator A endorses Curator B
   - Agent queries both A and B for sources
   - Transitive trust with decay (2nd-degree endorsements weaker than 1st-degree)

4. **MCP tools expose endorsements**
   - `get_endorsements` returns endorsement list (empty in v1)
   - Agents check endorsements before trusting sources
   - Cross-registry queries aggregate results

### What's Not in v1

❌ Endorsement signing and verification
❌ get_endorsements tool implementation (scaffolded but returns empty)
❌ Trust graph traversal
❌ Cross-registry queries
❌ Pubky homeserver storage
❌ Reputation scoring

### Why This Matters

**Decentralized curation** at scale requires trust mechanisms:
- Not everyone trusts the same curator
- Different domains require different expertise
- A web-of-trust allows specialization and discovery
- Cryptographic signatures prevent impersonation

The PKARR foundation in v1 enables this future without committing to the full complexity yet.

## Implementation Details

### Code References

- **Keypair generation:** `src/pubky/identity.rs`
- **Public key storage:** `src/main.rs` (AppState)
- **Health endpoint:** `src/http/routes.rs`
- **MCP provenance tool:** `src/mcp/tools.rs`
- **Environment config:** `src/config/mod.rs`

### Dependencies

- **pkarr crate:** Provides Ed25519 keypair and z-base-32 encoding
- **hex crate:** Decodes 64-char hex secret keys
- **curve25519-dalek:** Underlying elliptic curve cryptography (patched from git)

### Testing Identity

**Local development:**
1. Run server without `PKARR_SECRET_KEY` → ephemeral identity
2. Check `/health` for pubkey
3. Restart server → pubkey changes (expected)

**Production:**
1. Generate secret key: `openssl rand -hex 32`
2. Set `PKARR_SECRET_KEY=<output>` in production environment
3. Deploy server
4. Check `/health` → pubkey should be stable across deploys

## Conclusion

PKARR provides 3GS with **self-sovereign cryptographic identity**:
- Curator identity independent of domains or platforms
- Verifiable via public key checks (no trusted third party)
- Foundation for future federated trust and endorsements

While v1 uses only the keypair primitives, the infrastructure is in place for:
- Signature verification
- Cross-registry trust graphs
- Decentralized curator networks
- Pubky homeserver integration

**Verify it yourself:** Use the curl commands above to confirm your 3GS server's identity.
