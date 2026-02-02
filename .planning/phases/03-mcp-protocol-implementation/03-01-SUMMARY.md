---
phase: 03-mcp-protocol-implementation
plan: 01
subsystem: protocol
tags: [mcp, json-rpc, protocol, handler, initialization]

requires:
  - 01-01: Registry types for handler dependency
  - 02-01: MatchConfig for handler construction

provides:
  - JSON-RPC 2.0 types (JsonRpcRequest, JsonRpcResponse, JsonRpcError)
  - MCP protocol types (InitializeParams, CallToolParams)
  - McpHandler with initialize handshake and method dispatch
  - Protocol compliance (batch rejection, notification handling, error codes)

affects:
  - 03-02: Tool implementations will use McpHandler::handle_tools_list/call stubs
  - 04-01: HTTP transport will integrate McpHandler::handle_json as entry point

tech-stack:
  added:
    - schemars: JSON Schema generation support (with derive feature)
  patterns:
    - JSON-RPC 2.0 envelope handling with strict protocol compliance
    - Initialization handshake with AtomicBool state tracking
    - Option<String> return type for notification vs response distinction

key-files:
  created:
    - src/mcp/types.rs: JSON-RPC 2.0 and MCP protocol types
    - src/mcp/handler.rs: McpHandler with init handshake and dispatch
  modified:
    - src/mcp/error.rs: ParseError and SerializationError variants
    - src/mcp/mod.rs: Module exports for mcp subsystem
    - Cargo.toml: Added schemars dependency

decisions:
  - id: 03-01-init-gate
    what: Enforce initialize-before-tools handshake with AtomicBool
    why: MCP spec requires initialization before tool methods
    impact: All tool methods check initialized flag before executing
    alternatives: Could use Option<Instant> to track init time, but bool is simpler

  - id: 03-01-batch-reject
    what: Reject batch requests at raw JSON level (before parsing)
    why: Server doesn't support batch operations per spec
    impact: Array detection happens first, returns -32600 immediately
    alternatives: Could parse first then reject, but early rejection is cleaner

  - id: 03-01-notification-none
    what: Return None from handle_json for notifications
    why: Notifications have no id field and expect no response
    impact: Caller must handle Option<String> return type
    alternatives: Could return empty string, but Option is more explicit

  - id: 03-01-forward-compat
    what: InitializeParams without deny_unknown_fields
    why: MCP spec may add fields in future versions
    impact: Server accepts unknown fields in initialize params
    alternatives: Strict validation would break on spec changes

metrics:
  duration: 2 minutes
  tasks: 2
  commits: 2
  tests_added: 11
  files_created: 2
  files_modified: 3
  completed: 2026-02-02
---

# Phase 03 Plan 01: MCP Protocol Foundation Summary

**One-liner:** JSON-RPC 2.0 handler with MCP initialize handshake, batch rejection, notification handling, and protocol compliance.

## Objective Achieved

Created the MCP protocol foundation layer that handles JSON-RPC 2.0 envelope processing, initialization handshake, and method dispatch routing. This provides the protocol scaffolding that Plan 02's tool implementations will build upon.

## Tasks Completed

| Task | Description | Commit | Files |
|------|-------------|--------|-------|
| 1 | Add schemars dependency and create MCP types and error enum | 1c11ea6 | Cargo.toml, src/mcp/types.rs, src/mcp/error.rs |
| 2 | Create handler with init handshake, dispatch, and protocol tests | b3a6bd5 | src/mcp/handler.rs, src/mcp/mod.rs |

## What Was Built

### 1. JSON-RPC 2.0 Types (`src/mcp/types.rs`)

Complete type system for JSON-RPC 2.0 protocol:

- **JsonRpcRequest**: Request message with jsonrpc, id, method, params
- **JsonRpcResponse**: Response message with result or error
- **JsonRpcError**: Error object with code, message, data

**Convenience constructors** on JsonRpcResponse:
- `success(id, result)` - Wrap result in success response
- `error(id, code, message)` - Wrap in error response
- `parse_error()` - Code -32700
- `invalid_request()` - Code -32600
- `method_not_found(id)` - Code -32601
- `invalid_params(id)` - Code -32602
- `not_initialized(id)` - Code -32002
- `tool_result(id, text, is_error)` - MCP content format

**Error code constants**: PARSE_ERROR, INVALID_REQUEST, METHOD_NOT_FOUND, INVALID_PARAMS, NOT_INITIALIZED

### 2. MCP Protocol Types

- **InitializeParams**: protocolVersion, capabilities, clientInfo (WITHOUT deny_unknown_fields for forward-compatibility)
- **CallToolParams**: name, arguments (WITH deny_unknown_fields for strict validation)

### 3. Protocol Handler (`src/mcp/handler.rs`)

**McpHandler** struct with:
- `Arc<AtomicBool>` for initialization state tracking
- `Arc<Registry>` for tool access (used in Plan 02)
- `MatchConfig` for query matching (used in Plan 02)

**Main entry point**: `handle_json(&self, raw_json: &str) -> Option<String>`

Returns `None` for notifications (no response needed), `Some(String)` for requests.

**Protocol flow**:
1. Parse raw JSON (return parse_error if invalid)
2. Reject batch requests (arrays) with -32600
3. Deserialize into JsonRpcRequest (return parse_error if invalid)
4. Validate jsonrpc field is "2.0" (return invalid_request if not)
5. Check if notification (no id field) - return None if so
6. Enforce init gate: non-initialize methods require initialized flag
7. Dispatch to method handlers

**Method handlers**:
- `initialize` - Sets initialized flag, returns protocol response
- `notifications/initialized` - Client notification, silently ignored
- `tools/list` - Stub returning method_not_found (Plan 02 implements)
- `tools/call` - Stub returning method_not_found (Plan 02 implements)
- Unknown methods - Return method_not_found

### 4. Protocol Compliance Tests

11 comprehensive tests covering all protocol requirements:

1. **test_initialize_returns_protocol_version** - Verify MCP response structure
2. **test_initialize_sets_initialized_flag** - Confirm state transition
3. **test_pre_init_tools_list_rejected** - Init gate enforcement
4. **test_pre_init_tools_call_rejected** - Init gate enforcement
5. **test_batch_request_rejected** - Batch rejection with correct error code
6. **test_notification_returns_none** - Notification handling
7. **test_unknown_method_returns_error** - Method dispatch error handling
8. **test_invalid_json_returns_parse_error** - JSON parsing error handling
9. **test_all_responses_have_jsonrpc_field** - Protocol compliance
10. **test_invalid_params_on_initialize** - Parameter validation
11. **test_invalid_jsonrpc_version** - Version validation

All tests pass. Cargo check and clippy clean (lib only - bin has expected unused import warnings for Plan 02).

## Deviations from Plan

None - plan executed exactly as written.

## Decisions Made

### 1. Initialization Handshake with AtomicBool

**Context**: MCP spec requires clients call initialize before tool methods.

**Decision**: Use `Arc<AtomicBool>` to track initialized state across calls.

**Why**: Simple, thread-safe, minimal overhead. AtomicBool is perfect for binary state tracking.

**Alternatives considered**: `Option<Instant>` to track initialization time, but we don't need timestamps.

**Impact**: All tool methods check `initialized.load(Ordering::SeqCst)` before executing. Returns -32002 if not initialized.

### 2. Batch Rejection at Raw JSON Level

**Context**: Server doesn't support batch requests per MCP spec.

**Decision**: Check if parsed JSON is array BEFORE deserializing into JsonRpcRequest.

**Why**: Early rejection is cleaner and more efficient. No need to parse individual requests in batch.

**Alternatives considered**: Parse each request in batch then reject, but wastes cycles.

**Impact**: Any JSON array input immediately returns -32600 with "Batch requests not supported" message.

### 3. Notification Handling with Option Return

**Context**: Notifications (requests without id field) expect no response.

**Decision**: Return `Option<String>` from handle_json - None for notifications, Some for requests.

**Why**: Makes notification vs request distinction explicit in type system.

**Alternatives considered**: Return empty string for notifications, but Option is more idiomatic Rust.

**Impact**: HTTP layer (Plan 04) must handle `Option<String>` - send response only if Some.

### 4. Forward-Compatible InitializeParams

**Context**: MCP spec is evolving, may add new fields to initialize request.

**Decision**: Do NOT use `#[serde(deny_unknown_fields)]` on InitializeParams.

**Why**: Server should accept new spec fields without breaking. We only read fields we need.

**Alternatives considered**: Strict validation with deny_unknown_fields, but would break on spec updates.

**Impact**: Future MCP spec versions can add initialize params without server code changes. Use deny_unknown_fields only on OUR input types (CallToolParams).

## Next Phase Readiness

**Phase 03 Plan 02 blockers**: None - all stubs ready.

**What's ready**:
- McpHandler::handle_tools_list stub exists
- McpHandler::handle_tools_call stub exists
- Registry and MatchConfig are in handler struct
- JsonRpcResponse::tool_result constructor ready for MCP content format

**What Plan 02 needs to do**:
1. Implement handle_tools_list - return registry categories as MCP tools
2. Implement handle_tools_call - dispatch to get_sources query matcher
3. Use match_query from scorer module (Phase 02)
4. Return results in MCP content format with tool_result constructor

**Known issues**: None.

**Technical debt**: None.

## Testing Coverage

- Protocol compliance: 11/11 tests pass
- Init handshake: Covered
- Batch rejection: Covered
- Notification handling: Covered
- Error codes: All spec codes tested (-32700, -32600, -32601, -32602, -32002)
- Method dispatch: Covered

## Files Modified

**Created**:
- `src/mcp/types.rs` (136 lines) - JSON-RPC 2.0 and MCP types
- `src/mcp/handler.rs` (370 lines) - Protocol handler and tests

**Modified**:
- `src/mcp/error.rs` - Added ParseError and SerializationError
- `src/mcp/mod.rs` - Added types and handler module exports
- `Cargo.toml` - Added schemars = { version = "1", features = ["derive"] }

## Performance Notes

- Execution time: 2 minutes
- All cargo check/test/clippy operations passed
- Handler uses Arc for zero-copy registry sharing
- AtomicBool for lock-free initialization checking

## Key Learnings

1. **Early validation saves work**: Checking for array at raw JSON level avoids parsing overhead.

2. **Option types communicate intent**: `Option<String>` return makes notification vs request explicit in type system.

3. **Forward-compatibility is critical**: MCP spec is evolving - avoid deny_unknown_fields on spec-owned types.

4. **Convenience constructors reduce boilerplate**: JsonRpcResponse::parse_error() is cleaner than manual error object construction.

5. **AtomicBool perfect for init gates**: Simple, thread-safe, no lock overhead for binary state.

---

**Status**: ✅ Complete - Ready for Phase 03 Plan 02 (Tool Implementations)
