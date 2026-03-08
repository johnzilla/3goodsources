# Requirements: Three Good Sources (3GS)

**Defined:** 2026-03-07
**Core Value:** Agents get curated, high-quality sources instead of SEO-gamed search results — three good sources per topic, human-vetted, cryptographically signed, served via open protocol.

## v2.0 Requirements

Requirements for community curation milestone. Each maps to roadmap phases.

### Audit Log

- [x] **AUDIT-01**: Every registry change creates an append-only audit log entry with timestamp, action, category, data, and actor
- [x] **AUDIT-02**: Each audit entry is signed by the actor's Ed25519 key using a defined canonical format
- [x] **AUDIT-03**: Each audit entry includes a previous_hash field linking to the prior entry (hash chain)
- [x] **AUDIT-04**: GET /audit endpoint returns audit entries filterable by since, category, and action
- [x] **AUDIT-05**: get_audit_log MCP tool returns audit entries with the same filtering as the REST endpoint
- [x] **AUDIT-06**: Retroactive audit entries exist for all 30 existing sources from v1.0

### Identity

- [x] **IDENT-01**: identities.json maps PKARR pubkeys to platform handles (X, Nostr, GitHub) with human/bot type classification
- [x] **IDENT-02**: Every platform claim includes a proof URL for independent verification
- [x] **IDENT-03**: Bot identities link to a human operator's pubkey
- [ ] **IDENT-04**: GET /identities endpoint returns all registered identities
- [ ] **IDENT-05**: GET /identities/{pubkey} endpoint returns a single identity with all linked platforms
- [ ] **IDENT-06**: get_identity MCP tool returns identity info for a given pubkey
- [x] **IDENT-07**: Curator's own identity (John Turner) is registered with real platform proofs

### Contributions

- [ ] **CONTRIB-01**: contributions.json holds proposals with defined status lifecycle (pending, approved, rejected, withdrawn)
- [ ] **CONTRIB-02**: Proposals support actions: add_source, update_source, remove_source, add_category, update_category
- [ ] **CONTRIB-03**: Human and bot votes are tracked separately per proposal, classified by voter's identity type
- [ ] **CONTRIB-04**: GET /proposals endpoint returns proposals filterable by status and category
- [ ] **CONTRIB-05**: GET /proposals/{id} endpoint returns a single proposal with vote details
- [ ] **CONTRIB-06**: list_proposals and get_proposal MCP tools expose proposal data to agents

## Future Requirements

### Submission API

- **SUBMIT-01**: Verified identities can submit proposals via signed POST request
- **SUBMIT-02**: Verified identities can cast signed votes via POST request
- **SUBMIT-03**: Identity claims can be submitted via signed POST with proof URL

### Automated Verification

- **VERIFY-01**: Server automatically checks proof URLs for identity claims
- **VERIFY-02**: Automated merge when proposal reaches vote threshold

### AI Note Writer

- **NOTER-01**: Bot monitors X for posts matching 3GS categories
- **NOTER-02**: Bot queries 3GS and drafts Community Notes citing sources
- **NOTER-03**: Bot submits notes via X API and tracks outcomes

## Out of Scope

| Feature | Reason |
|---------|--------|
| Write API for proposals/votes | Read-only server for v2.0; curator manages JSON manually |
| OAuth identity verification | Manual curation sufficient for v2.0; automate later |
| Automated bot detection | Unsolved research problem; classify by identity type instead |
| Real-time notifications | No write API means no real-time events |
| AI Note Writer | Separate project, not part of 3GS server |
| Nostr relay infrastructure | Out of scope for server-side work |
| Hash chain validation at runtime | Include previous_hash field but defer chain integrity checks |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| AUDIT-01 | Phase 12 | Complete |
| AUDIT-02 | Phase 12 | Complete |
| AUDIT-03 | Phase 12 | Complete |
| AUDIT-04 | Phase 12 | Complete |
| AUDIT-05 | Phase 12 | Complete |
| AUDIT-06 | Phase 12 | Complete |
| IDENT-01 | Phase 13 | Complete |
| IDENT-02 | Phase 13 | Complete |
| IDENT-03 | Phase 13 | Complete |
| IDENT-04 | Phase 13 | Pending |
| IDENT-05 | Phase 13 | Pending |
| IDENT-06 | Phase 13 | Pending |
| IDENT-07 | Phase 13 | Complete |
| CONTRIB-01 | Phase 14 | Pending |
| CONTRIB-02 | Phase 14 | Pending |
| CONTRIB-03 | Phase 14 | Pending |
| CONTRIB-04 | Phase 14 | Pending |
| CONTRIB-05 | Phase 14 | Pending |
| CONTRIB-06 | Phase 14 | Pending |

**Coverage:**
- v2.0 requirements: 19 total
- Mapped to phases: 19
- Unmapped: 0

---
*Requirements defined: 2026-03-07*
*Last updated: 2026-03-07 after roadmap creation*
