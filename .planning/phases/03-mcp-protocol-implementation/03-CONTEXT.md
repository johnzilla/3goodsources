# Phase 3: MCP Protocol Implementation - Context

**Gathered:** 2026-02-02
**Status:** Ready for planning

<domain>
## Phase Boundary

Handle MCP JSON-RPC 2.0 protocol with four tools: `get_sources`, `list_categories`, `get_provenance`, `get_endorsements`. This phase implements protocol handling and tool dispatch only — HTTP transport is Phase 4.

</domain>

<decisions>
## Implementation Decisions

### Tool response shape
- `get_sources`: Return full category (name, slug, description) AND all 3 ranked sources (title, URL, description, rank)
- `list_categories`: Return slug, display name, and domain tags for each category
- `get_provenance`: Return curator name, public key (when available), and verification instructions
- `get_endorsements`: Return endorsements list (empty for v1)
- All response content in plain text — no markdown in field values

### Error behavior
- No match: Return success with empty sources array and "no match" message (not a JSON-RPC error)
- Unknown tool: Standard JSON-RPC method not found (-32601), no listing of valid tool names
- Invalid params: Minimal JSON-RPC invalid params (-32602) error, no extra detail
- Tool-level errors use MCP content result with `isError: true` — JSON-RPC errors only for protocol-level issues

### Protocol strictness
- No batch request support — single requests only, batch returns error
- Notifications (no id field) ignored silently per JSON-RPC spec
- Initialize handshake enforced strictly — error if client calls tools/list or tools/call before initialize
- Protocol version: research latest stable MCP version at planning time and use that

### Tool schemas & naming
- Tool names: snake_case as defined — `get_sources`, `list_categories`, `get_provenance`, `get_endorsements`
- Strict input validation: reject requests with extra/unknown parameters (consistent with project's `deny_unknown_fields` pattern)
- `get_sources` accepts required `query` parameter + optional `threshold` parameter for match sensitivity tuning
- Tool descriptions in `tools/list` should be detailed with examples to help agents use tools correctly

### Claude's Discretion
- Exact JSON-RPC parsing implementation
- Internal dispatch architecture
- MCP capability flags in initialize response
- How to structure the protocol handler module

</decisions>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 03-mcp-protocol-implementation*
*Context gathered: 2026-02-02*
