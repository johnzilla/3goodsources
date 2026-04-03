# Phase 16: Core Federation - Context

**Gathered:** 2026-04-03
**Status:** Ready for planning

<domain>
## Phase Boundary

Wire the federation networking layer and MCP tools. PeerCache gains reqwest::Client and a background refresh loop. MCP handler becomes async. New `get_federated_sources` tool queries local + peer registries with trust tagging. `get_endorsements` updated with real data. DRY helper for all tool responses. Graceful shutdown for background task.

</domain>

<decisions>
## Implementation Decisions

### Peer Cache Networking (NET-01, NET-02, NET-03)
- **D-01:** PeerCache gains `client: reqwest::Client` field, created once in `PeerCache::new()` and shared across all fetches
- **D-02:** `refresh_loop(interval: Duration, shutdown: tokio::sync::watch::Receiver<bool>)` runs as background tokio task via `tokio::select!` on interval tick vs shutdown signal
- **D-03:** Per-peer fetch: `GET {url}/registry` with 10-second reqwest timeout. Parse response as `PeerRegistry` (lax types from Phase 15)
- **D-04:** On success: update CachedPeer.registry, set last_success to now, status to Fresh
- **D-05:** On failure (timeout, HTTP error, parse error): log WARN, update last_attempt, keep existing cache. Mark Stale if last_success > 1 hour ago, Unreachable if never succeeded
- **D-06:** Peers fetched sequentially in refresh loop (not parallel). For v3.0 with 1-5 peers, sequential is simpler and sufficient
- **D-07:** `get_all_cached_peers()` method returns read-only snapshot of all peers with registries, including stale flag

### Graceful Shutdown (NET-04)
- **D-08:** Use `tokio::sync::watch` channel for shutdown signaling (simpler than CancellationToken, no new dep)
- **D-09:** main.rs creates `let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false)` before spawning refresh loop
- **D-10:** On axum server completion (after `axum::serve(listener, app).await`), send `shutdown_tx.send(true)` to stop refresh loop
- **D-11:** Refresh loop checks shutdown between peer fetches, not mid-fetch. Clean termination within 10s (one fetch timeout)

### Async MCP Refactor (MCP-04)
- **D-12:** `McpHandler::handle_json()` becomes `async fn handle_json()`
- **D-13:** `handle_tools_call()` becomes `async fn handle_tools_call()`
- **D-14:** All tool functions in tools.rs gain `async` keyword (no behavior change for existing tools, enables RwLock reads for new tool)
- **D-15:** `handle_tool_call()` gains `peer_cache: &PeerCache` parameter
- **D-16:** McpHandler struct gains `peer_cache: Arc<PeerCache>` field
- **D-17:** server.rs `mcp_endpoint` handler: `.await` on `handle_json()`
- **D-18:** Existing sync unit tests in handler.rs need `#[tokio::test]` and `.await` on `handle_json` calls

### Federated Sources Tool (MCP-01)
- **D-19:** New `get_federated_sources` MCP tool with same params as `get_sources` (query, threshold)
- **D-20:** Query flow: run match_query() on local registry, then on each cached peer's registry
- **D-21:** Output format: local results first (trust: "direct"), then peer results (trust: "endorsed") with curator info and stale flag
- **D-22:** If no peers cached or all unreachable: behaves identically to get_sources (local only)
- **D-23:** Multiple curators matching same category slug: all returned, no merging. Agent sees "John recommends X, Alice recommends Y"

### Updated Endorsements Tool (MCP-02)
- **D-24:** `get_endorsements` shows real endorsement data: pubkey, url, name (if set), since
- **D-25:** Empty endorsements: keep existing "future feature" message
- **D-26:** Non-empty: list each endorsement with all fields

### DRY Tool Response Helper (MCP-03)
- **D-27:** `fn tool_response(text: &str, is_error: bool) -> Value` helper in tools.rs
- **D-28:** Refactor all 8 existing tools + 1 new tool to use it
- **D-29:** Replace `json!({"content": [{"type": "text", "text": ...}], "isError": ...})` pattern throughout

### Server Wiring
- **D-30:** AppState gains `pub peer_cache: Arc<PeerCache>`
- **D-31:** main.rs: create PeerCache after registry load, spawn refresh loop, pass to McpHandler and AppState
- **D-32:** Tool count changes from 8 to 9. Update `get_tools_list()` and test assertion

### Claude's Discretion
- Exact text formatting of federated source results (line layout, separators)
- Whether to show peer cache stats in health endpoint
- Test mocking strategy for peer HTTP fetches (mock server vs mock client)
- Order of refactoring steps within each plan

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Federation Types (from Phase 15)
- `src/federation/types.rs` — PeerRegistry, PeerCurator, PeerEndorsement, FederatedMatch, TrustLevel, PeerStatus, CachedPeer
- `src/federation/cache.rs` — PeerCache::new() with self-endorsement guard (to be extended)
- `src/federation/error.rs` — FederationError enum

### MCP Handler (to be refactored)
- `src/mcp/handler.rs` — McpHandler with handle_json(), handle_tools_call() (sync → async)
- `src/mcp/tools.rs` — All 8 tool functions, handle_tool_call() dispatch, get_tools_list()

### Server Wiring
- `src/server.rs` — AppState struct, build_router(), mcp_endpoint handler
- `src/main.rs` — Server startup, config loading, state initialization

### Eng Review Plan
- `~/.claude/plans/purring-marinating-taco.md` — Steps 2-4 cover this phase's architecture

### Design Doc
- `~/.gstack/projects/johnzilla-3goodsources/john-main-design-20260402-212653.md` — get_federated_sources tool contract, caching strategy, error handling

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/federation/cache.rs:PeerCache` — Already has RwLock, self-endorsement guard, peer_count(). Extend with reqwest::Client, refresh_loop(), get_all_cached_peers()
- `src/matcher/match_query()` — Reuse for querying peer registries (takes &Registry, works with PeerRegistry's categories since they reuse Category/Source types)
- `src/mcp/tools.rs` — All 8 tools follow identical json!() response pattern, ready for DRY extraction

### Established Patterns
- `tokio::spawn` for background tasks (not used yet but standard for axum/tokio)
- `Arc<T>` for shared state across handlers (Registry, AuditLog, etc all use this pattern)
- Tool dispatch: match on name string, call function with shared state references
- `#[tokio::test]` used in federation/cache.rs tests (async test pattern established)

### Integration Points
- `src/main.rs:94` — AppState construction, add peer_cache field
- `src/main.rs:104` — After build_router, before serve: spawn refresh loop
- `src/server.rs:63` — mcp_endpoint: add .await to handle_json()
- `src/mcp/handler.rs:152` — handle_tool_call dispatch: add "get_federated_sources" arm
- `src/mcp/tools.rs:93` — get_tools_list(): add 9th tool definition
- `src/mcp/handler.rs:14` — McpHandler struct: add peer_cache field
- `src/lib.rs:5` — federation module already declared (Phase 15)

</code_context>

<specifics>
## Specific Ideas

No specific requirements — follow eng-reviewed plan architecture. The design doc specifies the exact tool contract for get_federated_sources (input params, output format, trust tagging, error handling).

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 16-core-federation*
*Context gathered: 2026-04-03*
