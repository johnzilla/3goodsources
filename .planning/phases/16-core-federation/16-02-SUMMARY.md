---
phase: 16-core-federation
plan: "02"
subsystem: mcp
tags: [async, refactor, dry, endorsements]
dependency_graph:
  requires: [15-01, 15-02]
  provides: [async-mcp-handler, tool-response-helper, real-endorsements]
  affects: [src/mcp/tools.rs, src/mcp/handler.rs, src/server.rs]
tech_stack:
  added: [tokio::test]
  patterns: [tool_response DRY helper, async fn with .await dispatch]
key_files:
  created: []
  modified:
    - src/mcp/tools.rs
    - src/mcp/handler.rs
    - src/server.rs
decisions:
  - All 8 tool functions made async even though none await anything yet — prerequisite for Plan 03 federated tool
  - tool_response() helper replaces 12 json!() boilerplate occurrences across 8 tools
  - handle_initialize and handle_tools_list stay sync internally; only handle_tools_call needs async
metrics:
  duration_minutes: 8
  completed_date: "2026-04-02"
  tasks_completed: 2
  files_modified: 3
---

# Phase 16 Plan 02: Async MCP Handler + DRY Tool Response Summary

Async MCP handler refactor with tool_response() DRY helper and real endorsement data display.

## What Was Built

- `tool_response(text, is_error) -> Value` helper replaces 12 repetitive `json!({"content": [...]})` blocks across all 8 tool functions
- `handle_tool_call`, all 8 `tool_*` functions made `async fn` (no actual awaits yet, but required for Plan 03's federated tool)
- `handle_json` and `handle_tools_call` in McpHandler made `async fn` with `.await` on tool dispatch
- `mcp_endpoint` in server.rs updated to `.await` handle_json
- All 16 handler tests converted from `#[test]` to `#[tokio::test]` with `async fn` and `.await`
- `init_handler` test helper made async
- `tool_get_endorsements` updated to show real pubkey/url/name/since per endorsement when non-empty

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | DRY tool_response + async refactor | 4bf3dee | src/mcp/tools.rs, src/mcp/handler.rs, src/server.rs |
| 2 | Update get_endorsements real data | 4bf3dee | src/mcp/tools.rs (included in Task 1) |

## Verification

- `cargo build` passes with no errors (7 pre-existing warnings, not new)
- `cargo test --lib mcp::` passes all 24 tests
- `grep -c "#\[test\]" src/mcp/handler.rs` returns 0 (all converted)
- `grep -q "pub async fn handle_json" src/mcp/handler.rs` passes
- `grep -q "pub async fn handle_tool_call" src/mcp/tools.rs` passes
- `grep -q ".handle_json(&body).await" src/server.rs` passes

## Deviations from Plan

None - plan executed exactly as written. Task 2's endorsement update was folded into the Task 1 file write since both modified tools.rs; the single commit covers both tasks cleanly.

## Known Stubs

None - endorsements non-empty branch now shows real data. Empty branch shows the proper placeholder message per spec.

## Self-Check: PASSED
