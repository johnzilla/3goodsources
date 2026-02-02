---
phase: 03-mcp-protocol-implementation
plan: 02
subsystem: api
tags: [mcp, json-rpc, tools, schemars, fuzzy-matching, tdd]

# Dependency graph
requires:
  - phase: 03-01
    provides: MCP protocol foundation with initialize handshake and JSON-RPC infrastructure
  - phase: 02-02
    provides: Query matching engine with fuzzy scoring and threshold-based matching
  - phase: 01-03
    provides: Seeded registry.json with 10 categories and curated sources
provides:
  - tools/list method returning 4 tool definitions with JSON Schema inputSchema
  - get_sources tool for natural language query matching with threshold tuning
  - list_categories tool for browsing available topics
  - get_provenance tool for curator identity and verification
  - get_endorsements tool for trust relationships (v1: empty)
  - Error handling: unknown tools return -32601, invalid params return -32602
  - Match error formatting: no-match returns isError:true with available categories
affects: [04-http-transport, 05-pkarr-identity]

# Tech tracking
tech-stack:
  added: [schemars (JSON Schema generation)]
  patterns:
    - "TDD with RED-GREEN-REFACTOR commits"
    - "Tool parameter structs with JsonSchema + deny_unknown_fields"
    - "Result<Value, ToolCallError> for tool dispatch error handling"
    - "Plain text responses (no markdown) for agent consumption"
    - "Optional threshold parameter for match sensitivity tuning"

key-files:
  created:
    - src/mcp/tools.rs
  modified:
    - src/mcp/handler.rs
    - src/mcp/mod.rs
    - src/matcher/config.rs

key-decisions:
  - "JSON Schema generation via schemars::schema_for! macro for MCP inputSchema compliance"
  - "Plain text tool responses instead of markdown for better agent parsing"
  - "Match errors return MCP success with isError:true (not JSON-RPC errors) per user decision"
  - "Unknown tool name returns JSON-RPC -32601, invalid params return -32602"
  - "Clone derive on MatchConfig to support threshold parameter customization"
  - "Sorted category listing for deterministic output in list_categories"

patterns-established:
  - "Tool param structs: #[derive(JsonSchema, Deserialize)] with deny_unknown_fields"
  - "Tool handlers return Result<Value, ToolCallError> for error type discrimination"
  - "MCP content format: {content: [{type: 'text', text: string}], isError: bool}"
  - "Threshold parameter allows per-query match sensitivity override"

# Metrics
duration: 5min
completed: 2026-02-02
---

# Phase 3 Plan 2: MCP Tool Implementations Summary

**Four MCP tools with JSON Schema generation: get_sources with threshold tuning, list_categories, get_provenance, and get_endorsements**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-02T13:47:50Z
- **Completed:** 2026-02-02T13:52:39Z
- **Tasks:** 1 TDD task (3 commits: RED-GREEN-REFACTOR)
- **Files modified:** 4

## Accomplishments

- All 4 MCP tools implemented with proper JSON Schema input validation
- get_sources supports optional threshold parameter for match sensitivity tuning
- Match errors return user-friendly messages with available categories (isError: true)
- Unknown tools and invalid params return correct JSON-RPC error codes (-32601, -32602)
- 13 new comprehensive tests added (24 total MCP tests passing)
- TDD workflow with atomic commits: test (RED) → feat (GREEN) → refactor

## Task Commits

Each TDD phase was committed atomically:

1. **RED: Failing tests** - `739d3ba` (test)
   - Created tools.rs with stub implementations
   - Added 13 comprehensive tool tests to handler.rs
   - All tests fail with unimplemented!()

2. **GREEN: Implementation** - `e38c954` (feat)
   - Implemented get_tools_list() with schemars JSON Schema generation
   - Implemented handle_tool_call() dispatcher with error handling
   - Implemented all 4 tool handlers (get_sources, list_categories, get_provenance, get_endorsements)
   - Wired tools into handler methods
   - Added Clone derive to MatchConfig
   - Fixed test bug: corrected curator name assertion

3. **REFACTOR: Documentation** - `1550f0b` (refactor)
   - Added comprehensive doc comments to all tool handlers
   - Documented error behavior and threshold support
   - All tests passing, clippy clean

## Files Created/Modified

- `src/mcp/tools.rs` - Tool parameter types with JsonSchema derives, tool definitions for tools/list, and 4 tool handler implementations
- `src/mcp/handler.rs` - Updated handle_tools_list and handle_tools_call with dispatch to tools module, added 13 new tests
- `src/mcp/mod.rs` - Added tools module export
- `src/matcher/config.rs` - Added Clone derive to MatchConfig for threshold customization

## Decisions Made

**JSON Schema generation with schemars:** Used schemars::schema_for! macro to generate MCP-compliant inputSchema for each tool. This provides automatic type-to-schema conversion and keeps schemas in sync with Rust types.

**Plain text responses:** All tool responses return plain text (no markdown formatting) for better agent parsing. Labels like "Category:", "URL:", "Why:" are used for structure without markdown syntax.

**Match error handling:** Per user decision, no-match scenarios return MCP success responses with isError: true and helpful error messages (available categories, closest match score). This distinguishes business logic errors (no match) from protocol errors (unknown tool).

**Error code mapping:** Unknown tool names return JSON-RPC error -32601 (Method not found), invalid/extra tool parameters return -32602 (Invalid params). This follows JSON-RPC spec while providing clear error feedback.

**Threshold parameter:** get_sources accepts optional threshold parameter (0.0-1.0) to override default match_threshold from config. Enables per-query sensitivity tuning without modifying global config.

**Sorted category output:** list_categories sorts categories by slug for deterministic, reproducible output across runs.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed curator name assertion in test**
- **Found during:** GREEN phase test execution
- **Issue:** Test checked for "John Turner" but registry.json contains "3GS Curator"
- **Fix:** Updated test assertion to match actual registry curator name
- **Files modified:** src/mcp/handler.rs
- **Verification:** Test passes with correct curator name
- **Committed in:** e38c954 (GREEN phase commit)

**2. [Rule 1 - Bug] Fixed clippy useless_format warning**
- **Found during:** GREEN phase clippy check
- **Issue:** Using format!() with string literal (no interpolation) instead of .to_string()
- **Fix:** Changed format!("...") to "...".to_string() in get_endorsements
- **Files modified:** src/mcp/tools.rs
- **Verification:** Clippy passes with -D warnings
- **Committed in:** e38c954 (GREEN phase commit)

---

**Total deviations:** 2 auto-fixed (2 bugs)
**Impact on plan:** Both auto-fixes were correctness issues (wrong test data, unnecessary format call). No scope changes.

## Issues Encountered

None - TDD workflow executed smoothly with failing tests → implementation → refactor.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

**Ready for Phase 4 (HTTP Transport):**
- MCP protocol fully implemented (initialize + tools/list + tools/call)
- All 4 tools functional and tested
- Error handling complete (protocol errors + business logic errors)
- Plain text responses ready for HTTP streaming

**Phase 3 Complete:**
- Plan 01: MCP protocol foundation ✓
- Plan 02: Tool implementations ✓

**Next phase can:**
- Wrap McpHandler in axum HTTP endpoints
- Stream JSON-RPC requests/responses over HTTP
- Add CORS and health check endpoints

---
*Phase: 03-mcp-protocol-implementation*
*Completed: 2026-02-02*
