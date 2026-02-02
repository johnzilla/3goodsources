# Phase 3: MCP Protocol Implementation - Research

**Researched:** 2026-02-02
**Domain:** MCP JSON-RPC 2.0 protocol implementation in Rust
**Confidence:** HIGH

## Summary

The Model Context Protocol (MCP) is a standardized protocol built on JSON-RPC 2.0 that enables AI applications to interact with external tools and data sources. The latest stable specification is **2025-11-25** (released November 25, 2025), which is the current version that should be implemented.

MCP requires a strict three-step initialization handshake before functional requests (tools/list, tools/call) can be processed. The protocol distinguishes between protocol-level errors (JSON-RPC errors with standard codes) and tool-level errors (successful responses with `isError: true` flag). Manual implementation is straightforward with Rust's serde ecosystem, though an official Rust SDK (rmcp 0.14.0) is production-ready if needed.

The protocol uses a capability negotiation system during initialization, where clients and servers explicitly declare supported features. For this phase, we'll implement a minimal capability set focused on tools only (no resources, prompts, or sampling).

**Primary recommendation:** Manually implement MCP protocol using serde for JSON-RPC serialization/deserialization, schemars for tool input schema generation, and pattern matching for method dispatch. Use the official JSON-RPC 2.0 and MCP 2025-11-25 specifications as authoritative references. Implement strict handshake enforcement to ensure initialize completes before accepting tool requests.

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde + serde_json | 1.0.228 + 1.0.149 | JSON serialization/deserialization | De facto Rust JSON standard, already in dependencies |
| schemars | 1.2.0 | JSON Schema generation for tool inputs | Standard library for deriving JSON Schema from Rust types, integrates with serde |
| thiserror | 2.0.18 | Error enum derivation | Already used project-wide for per-module error enums |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| anyhow | 1.0.100 | Error context wrapping | Already in dependencies for context-rich error handling |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Manual serde | rmcp 0.14.0 (official Rust MCP SDK) | SDK provides macros and reduces boilerplate, but adds dependency and abstraction layer. Manual implementation gives full control and deeper understanding. Recommended: manual for Phase 3, consider SDK only if complexity grows. |
| schemars | jsonschema crate | schemars integrates better with serde derives and generates schemas at compile time. jsonschema is for validation, not generation. |
| Pattern matching dispatch | jsonrpc-v2 or similar router | Router libraries add dependencies and abstractions. Pattern matching is simple, explicit, and sufficient for 4 methods. |

**Installation:**

Already present in Cargo.toml. Only addition needed:
```bash
cargo add schemars --features derive
```

## Architecture Patterns

### Recommended Module Structure

```
src/mcp/
├── mod.rs              # Public API and exports
├── error.rs            # McpError enum (already exists)
├── types.rs            # JSON-RPC and MCP message types
├── handler.rs          # Request handler and method dispatch
├── initialize.rs       # Initialize handshake logic
└── tools.rs            # Tool implementations (get_sources, list_categories, etc.)
```

### Pattern 1: JSON-RPC Message Envelope

**What:** All MCP communication uses JSON-RPC 2.0 format with strict field requirements.

**When to use:** Every request/response must include `jsonrpc: "2.0"` and appropriate fields.

**Example:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,  // Must be exactly "2.0"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,  // Absent = notification
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,  // Must be exactly "2.0"
    pub id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}
```

**Source:** [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)

### Pattern 2: Protocol vs Tool Error Separation

**What:** Protocol errors use JSON-RPC error codes; tool execution failures use `isError: true` in result content.

**When to use:**
- Protocol errors: Malformed JSON, invalid params, method not found, pre-init requests
- Tool errors: Valid request but tool execution failed (no match, business logic error)

**Example:**
```rust
// Protocol error (invalid params)
JsonRpcResponse {
    jsonrpc: "2.0",
    id: request_id,
    result: None,
    error: Some(JsonRpcError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: None,  // Minimal per user decision
    }),
}

// Tool error (no matching category)
JsonRpcResponse {
    jsonrpc: "2.0",
    id: request_id,
    result: Some(json!({
        "content": [{
            "type": "text",
            "text": "No matching category found for query. Available categories: ..."
        }],
        "isError": true
    })),
    error: None,
}
```

**Source:** [MCP Error Handling Best Practices](https://mcpcat.io/guides/error-handling-custom-mcp-servers/)

### Pattern 3: Initialize Handshake Enforcement

**What:** Track session state to reject tools/list and tools/call before initialize completes.

**When to use:** Every request must check if initialization happened first.

**Example:**
```rust
pub struct McpHandler {
    initialized: Arc<AtomicBool>,
    registry: Arc<Registry>,
    match_config: MatchConfig,
}

impl McpHandler {
    pub async fn handle_request(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        // Notifications (no id) are silently ignored per JSON-RPC spec
        let Some(id) = req.id.clone() else {
            return JsonRpcResponse::empty();  // No response for notifications
        };

        // Check initialization for non-initialize methods
        if req.method != "initialize" && !self.initialized.load(Ordering::SeqCst) {
            return JsonRpcResponse::error(
                id,
                -32002,  // Server error (custom)
                "Server not initialized. Call initialize first.".to_string(),
            );
        }

        match req.method.as_str() {
            "initialize" => self.handle_initialize(id, req.params).await,
            "tools/list" => self.handle_tools_list(id, req.params).await,
            "tools/call" => self.handle_tools_call(id, req.params).await,
            _ => JsonRpcResponse::error(id, -32601, "Method not found".to_string()),
        }
    }
}
```

**Source:** [MCP Architecture - Lifecycle Management](https://modelcontextprotocol.io/docs/learn/architecture)

### Pattern 4: Tool Input Schema with schemars

**What:** Derive JSON Schema from Rust structs for tool parameter validation.

**When to use:** Every tool needs an input schema in tools/list response.

**Example:**
```rust
use schemars::{schema_for, JsonSchema};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetSourcesParams {
    /// Natural language query describing what sources to find
    pub query: String,
    /// Optional match threshold (0.0-1.0) for sensitivity tuning
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f64>,
}

// Generate schema for tools/list response
let schema = schema_for!(GetSourcesParams);
let schema_json = serde_json::to_value(schema).unwrap();

// In tools/list response:
Tool {
    name: "get_sources".to_string(),
    description: "Find curated sources for a topic...".to_string(),
    inputSchema: schema_json,
}
```

**Source:** [schemars Documentation](https://docs.rs/schemars)

### Pattern 5: Method Dispatch with Pattern Matching

**What:** Use explicit match statement to route JSON-RPC method names to handlers.

**When to use:** Central dispatch point for all incoming requests.

**Example:**
```rust
impl McpHandler {
    pub async fn handle_request(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        let Some(id) = req.id.clone() else {
            return JsonRpcResponse::empty();  // Silent ignore per spec
        };

        if req.method != "initialize" && !self.initialized.load(Ordering::SeqCst) {
            return JsonRpcResponse::error(id, -32002,
                "Server not initialized".to_string());
        }

        match req.method.as_str() {
            "initialize" => self.handle_initialize(id, req.params),
            "tools/list" => self.handle_tools_list(id, req.params),
            "tools/call" => self.handle_tools_call(id, req.params),
            "notifications/initialized" => JsonRpcResponse::empty(),  // Notification
            _ => JsonRpcResponse::error(id, -32601, "Method not found".to_string()),
        }
    }

    async fn handle_tools_call(&self, id: Value, params: Option<Value>)
        -> JsonRpcResponse
    {
        let params: CallToolParams = match params {
            Some(p) => match serde_json::from_value(p) {
                Ok(params) => params,
                Err(_) => return JsonRpcResponse::error(
                    id, -32602, "Invalid params".to_string()),
            },
            None => return JsonRpcResponse::error(
                id, -32602, "Invalid params".to_string()),
        };

        match params.name.as_str() {
            "get_sources" => self.tool_get_sources(id, params.arguments).await,
            "list_categories" => self.tool_list_categories(id, params.arguments).await,
            "get_provenance" => self.tool_get_provenance(id, params.arguments).await,
            "get_endorsements" => self.tool_get_endorsements(id, params.arguments).await,
            _ => JsonRpcResponse::tool_error(id,
                format!("Unknown tool: {}", params.name)),
        }
    }
}
```

**Source:** Manual pattern, standard Rust practice

### Anti-Patterns to Avoid

- **Don't use stdout for logging:** MCP servers must write only JSON-RPC messages to stdout. All logging must go to stderr. This is critical for protocol compliance.
- **Don't mix protocol and tool errors:** Protocol violations return JSON-RPC errors; tool execution failures return successful responses with `isError: true`.
- **Don't respond to notifications:** JSON-RPC requests without `id` field are notifications and must never receive responses, even on error.
- **Don't accept batch requests:** Per user decision, single requests only. Batch arrays should return error -32600 (Invalid Request).
- **Don't expose system internals in error messages:** Return minimal, safe error messages to clients per user decision; log full details to stderr.

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON Schema generation | Manual schema building | schemars with #[derive(JsonSchema)] | Handles edge cases (Option, Vec, nested types), stays in sync with struct changes, generates correct 2020-12 dialect |
| JSON-RPC parsing | String parsing and manual field extraction | serde + #[serde(deny_unknown_fields)] | Validates required fields, handles optional fields correctly, provides clear error messages |
| Error enum boilerplate | Manual Display/Error impl | thiserror with #[error] attribute | Already project standard, reduces boilerplate, consistent with existing code |
| Protocol version negotiation | Custom version comparison logic | String comparison "YYYY-MM-DD" | MCP spec uses date strings, simple string equality check is sufficient |

**Key insight:** The JSON-RPC 2.0 and MCP protocols are well-defined specifications. Focus implementation effort on tool logic (matching queries, formatting responses) rather than protocol mechanics. Leverage serde's ecosystem for serialization and validation.

## Common Pitfalls

### Pitfall 1: Notification Response Leakage

**What goes wrong:** Server responds to JSON-RPC notifications (requests with no `id` field), violating spec.

**Why it happens:** Natural instinct to acknowledge every request; easy to forget id check.

**How to avoid:** Check for `id` field at the very start of request handling. Return early with no response if absent.

**Warning signs:** Client receives unexpected responses; JSON parsing errors from clients expecting no response; protocol test failures for notification handling.

```rust
// CORRECT:
let Some(id) = req.id.clone() else {
    return; // Silent ignore, no response
};

// WRONG:
if req.id.is_none() {
    return JsonRpcResponse::error(/* ... */);  // This is a response!
}
```

**Source:** [JSON-RPC 2.0 Specification - Notification](https://www.jsonrpc.org/specification)

### Pitfall 2: Pre-Initialize Request Acceptance

**What goes wrong:** Server processes tools/list or tools/call before initialize handshake completes, violating MCP lifecycle.

**Why it happens:** Forgetting to track initialization state; assuming clients will follow order.

**How to avoid:** Use atomic boolean flag to track initialization; check on every non-initialize request.

**Warning signs:** Clients receive responses without completing handshake; inconsistent capability behavior; state corruption when client assumes uninitialized.

```rust
// Track state
pub struct McpHandler {
    initialized: Arc<AtomicBool>,  // False until initialize completes
    // ...
}

// Check before processing
if req.method != "initialize" && !self.initialized.load(Ordering::SeqCst) {
    return JsonRpcResponse::error(id, -32002, "Not initialized".to_string());
}
```

**Source:** [MCP Architecture - Lifecycle](https://modelcontextprotocol.io/docs/learn/architecture)

### Pitfall 3: Batch Request Silent Acceptance

**What goes wrong:** Server receives array of requests (batch), processes them individually, violates user decision to reject batches.

**Why it happens:** JSON-RPC 2.0 spec supports batches; easy to accidentally accept during parsing.

**How to avoid:** Explicitly check if request is array before parsing; return error -32600 for batch attempts.

**Warning signs:** Unexpected array parsing; multiple responses generated; user decision violated.

```rust
// Check for array at raw JSON level before parsing
pub async fn handle_json(&self, json_str: &str) -> Result<String, McpError> {
    let raw: serde_json::Value = serde_json::from_str(json_str)?;

    if raw.is_array() {
        // Reject batch requests per user decision
        return Ok(serde_json::to_string(&JsonRpcResponse::error(
            Value::Null,
            -32600,
            "Batch requests not supported".to_string(),
        ))?);
    }

    let req: JsonRpcRequest = serde_json::from_value(raw)?;
    // Continue with single request...
}
```

**Source:** User decision in CONTEXT.md

### Pitfall 4: Unknown Field Acceptance

**What goes wrong:** Client sends extra/unknown parameters, server accepts silently instead of rejecting.

**Why it happens:** Default serde behavior ignores unknown fields; inconsistent with project's `deny_unknown_fields` pattern.

**How to avoid:** Apply `#[serde(deny_unknown_fields)]` to ALL MCP protocol structs, consistent with existing registry types.

**Warning signs:** Parameters accepted that shouldn't be; inconsistent validation; silent bugs from typos in parameter names.

```rust
// CORRECT: Consistent with project pattern
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]  // ✓ Project standard
pub struct GetSourcesParams {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f64>,
}

// WRONG: Would accept typos or extra fields
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetSourcesParams {  // Missing deny_unknown_fields
    pub query: String,
    pub threshold: Option<f64>,
}
```

**Source:** Project STATE.md decision: "Apply #[serde(deny_unknown_fields)] to ALL registry structs"

### Pitfall 5: Tool Error as Protocol Error

**What goes wrong:** No matching category returns JSON-RPC error -32602, making LLM think request was malformed.

**Why it happens:** Confusion between protocol validation (params present) and business logic (category exists).

**How to avoid:** Return successful response with `isError: true` and helpful message for business logic failures.

**Warning signs:** LLM retries with same query; error messages not visible to LLM; difficulty debugging from client side.

```rust
// CORRECT: Business logic error with isError: true
JsonRpcResponse::success(id, json!({
    "content": [{
        "type": "text",
        "text": "No matching category found. Available categories: rust-learning, bitcoin-node-setup, ..."
    }],
    "isError": true  // ✓ Signals failure but not protocol error
}))

// WRONG: Protocol error for business logic failure
JsonRpcResponse::error(id, -32602, "No match found".to_string())
```

**Source:** [MCP Error Handling Guide](https://mcpcat.io/guides/error-handling-custom-mcp-servers/)

### Pitfall 6: Markdown in Field Values

**What goes wrong:** Tool responses include markdown formatting in source descriptions or category names.

**Why it happens:** Temptation to add rich formatting for readability; confusion about where markdown belongs.

**How to avoid:** Return plain text in all field values per user decision; LLM or client handles formatting.

**Warning signs:** Escaped markdown characters in responses; formatting issues in LLM output; inconsistent display across clients.

```rust
// CORRECT: Plain text only
Source {
    name: "The Rust Programming Language".to_string(),
    url: "https://doc.rust-lang.org/book/".to_string(),
    why: "Official book covering fundamentals and advanced topics.".to_string(),
    // ... (no markdown!)
}

// WRONG: Markdown formatting
Source {
    name: "**The Rust Book**".to_string(),  // ✗
    url: "[book](https://...)".to_string(),  // ✗
    why: "Official book - *excellent* for beginners".to_string(),  // ✗
}
```

**Source:** User decision in CONTEXT.md: "All response content in plain text — no markdown in field values"

## Code Examples

Verified patterns from specifications and best practices:

### Initialize Request/Response

```rust
// Client sends:
{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
        "protocolVersion": "2025-11-25",
        "capabilities": {
            "roots": { "listChanged": false }
        },
        "clientInfo": {
            "name": "example-client",
            "version": "1.0.0"
        }
    }
}

// Server responds:
{
    "jsonrpc": "2.0",
    "id": 1,
    "result": {
        "protocolVersion": "2025-11-25",
        "capabilities": {
            "tools": {}  // We support tools
        },
        "serverInfo": {
            "name": "three-good-sources",
            "version": "0.1.0"
        }
    }
}

// Rust implementation:
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InitializeParams {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

impl McpHandler {
    fn handle_initialize(&self, id: Value, params: Option<Value>)
        -> JsonRpcResponse
    {
        let params: InitializeParams = match params {
            Some(p) => match serde_json::from_value(p) {
                Ok(p) => p,
                Err(_) => return JsonRpcResponse::error(
                    id, -32602, "Invalid params".to_string()),
            },
            None => return JsonRpcResponse::error(
                id, -32602, "Invalid params".to_string()),
        };

        // Set initialized flag
        self.initialized.store(true, Ordering::SeqCst);

        JsonRpcResponse::success(id, json!({
            "protocolVersion": "2025-11-25",
            "capabilities": {
                "tools": {}  // Minimal: we support tools
            },
            "serverInfo": {
                "name": "three-good-sources",
                "version": env!("CARGO_PKG_VERSION")
            }
        }))
    }
}
```

**Source:** [MCP Specification 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25)

### Tools/List with JSON Schema

```rust
// Client sends:
{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/list"
}

// Server responds:
{
    "jsonrpc": "2.0",
    "id": 2,
    "result": {
        "tools": [
            {
                "name": "get_sources",
                "description": "Find three curated sources for a topic...",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Natural language query..."
                        },
                        "threshold": {
                            "type": "number",
                            "description": "Optional match threshold..."
                        }
                    },
                    "required": ["query"]
                }
            },
            // ... other tools
        ]
    }
}

// Rust implementation:
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetSourcesParams {
    /// Natural language query describing what sources to find
    pub query: String,
    /// Optional match threshold (0.0-1.0) for sensitivity tuning
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f64>,
}

impl McpHandler {
    fn handle_tools_list(&self, id: Value, _params: Option<Value>)
        -> JsonRpcResponse
    {
        let tools = vec![
            json!({
                "name": "get_sources",
                "description": "Find three curated sources for a topic. \
                    Searches across categories using fuzzy matching and keyword \
                    boosting. Returns the matching category name, description, \
                    and all three ranked sources with URLs. \
                    Example query: 'learn rust programming'",
                "inputSchema": schema_for!(GetSourcesParams),
            }),
            json!({
                "name": "list_categories",
                "description": "List all available topic categories with their \
                    slugs and domain tags. Use this to discover what topics \
                    have curated sources. Returns category slug, display name, \
                    and domain tags. No parameters required.",
                "inputSchema": schema_for!(ListCategoriesParams),  // Empty struct
            }),
            json!({
                "name": "get_provenance",
                "description": "Get curator identity and verification instructions \
                    for this registry. Returns curator name, PKARR public key \
                    (when available), and instructions for cryptographic \
                    verification. No parameters required.",
                "inputSchema": schema_for!(GetProvenanceParams),  // Empty struct
            }),
            json!({
                "name": "get_endorsements",
                "description": "Get list of endorsed curators (empty for v1). \
                    Returns endorsements list with curator info. No parameters \
                    required.",
                "inputSchema": schema_for!(GetEndorsementsParams),  // Empty struct
            }),
        ];

        JsonRpcResponse::success(id, json!({ "tools": tools }))
    }
}
```

**Source:** [MCP Tools Documentation](https://modelcontextprotocol.io/specification/2025-11-25)

### Tools/Call with get_sources

```rust
// Client sends:
{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
        "name": "get_sources",
        "arguments": {
            "query": "learn rust"
        }
    }
}

// Server responds (success):
{
    "jsonrpc": "2.0",
    "id": 3,
    "result": {
        "content": [
            {
                "type": "text",
                "text": "Category: Rust Learning\n\n[Full formatted response with category and sources]"
            }
        ],
        "isError": false
    }
}

// Server responds (no match - tool error, not protocol error):
{
    "jsonrpc": "2.0",
    "id": 3,
    "result": {
        "content": [
            {
                "type": "text",
                "text": "No matching category found for query 'xyz'. Available categories: rust-learning, bitcoin-node-setup, ..."
            }
        ],
        "isError": true
    }
}

// Rust implementation:
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CallToolParams {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,
}

impl McpHandler {
    async fn tool_get_sources(&self, id: Value, args: Option<Value>)
        -> JsonRpcResponse
    {
        let params: GetSourcesParams = match args {
            Some(a) => match serde_json::from_value(a) {
                Ok(p) => p,
                Err(_) => return JsonRpcResponse::error(
                    id, -32602, "Invalid params".to_string()),
            },
            None => return JsonRpcResponse::error(
                id, -32602, "Invalid params".to_string()),
        };

        // Use existing matcher from Phase 2
        let result = match_query(
            &params.query,
            &self.registry,
            &self.match_config,
        );

        match result {
            Ok(match_result) => {
                let category = &self.registry.categories[&match_result.slug];

                // Format response per user decision: full category + all 3 sources
                let mut text = format!(
                    "Category: {}\nDescription: {}\n\nSources:\n\n",
                    category.name,
                    category.description
                );

                for source in &category.sources {
                    text.push_str(&format!(
                        "{}. {}\n   URL: {}\n   Why: {}\n\n",
                        source.rank,
                        source.name,
                        source.url,
                        source.why
                    ));
                }

                JsonRpcResponse::success(id, json!({
                    "content": [{
                        "type": "text",
                        "text": text
                    }],
                    "isError": false
                }))
            }
            Err(_) => {
                // Tool error: no match found
                let categories: Vec<&str> = self.registry.categories
                    .keys()
                    .map(|s| s.as_str())
                    .collect();

                JsonRpcResponse::success(id, json!({
                    "content": [{
                        "type": "text",
                        "text": format!(
                            "No matching category found for query '{}'. \
                             Available categories: {}",
                            params.query,
                            categories.join(", ")
                        )
                    }],
                    "isError": true
                }))
            }
        }
    }
}
```

**Source:** [MCP Tools Call Documentation](https://modelcontextprotocol.io/specification/2025-11-25)

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| MCP 2024-11-05 | MCP 2025-11-25 | Nov 2025 | Added Tasks (async ops), enhanced OAuth/PKCE, icons, JSON Schema 2020-12 standard. Removed JSON-RPC batching support. |
| stdio transport only | HTTP/SSE/stdio transports | June 2025 | HTTP POST becomes official transport alongside stdio. 3GS uses HTTP only for MVP. |
| serde_json::Value everywhere | Strongly typed structs with derive | Ongoing best practice | Type safety catches errors at compile time; deny_unknown_fields enforces strict validation |
| Manual JSON Schema writing | schemars #[derive(JsonSchema)] | Library matured ~2023 | Schemas stay in sync with types; compiler enforces correctness |
| Protocol errors for everything | Protocol vs tool error separation | MCP specification from start | LLMs can see and reason about tool failures; retry logic more intelligent |

**Deprecated/outdated:**
- **JSON-RPC batching**: Removed in MCP 2025-06-18, explicitly not supported. Reject batch requests with error -32600.
- **SSE transport without polling**: Now supports server-initiated disconnect in 2025-11-25. Not relevant for Phase 3 (HTTP is Phase 4).
- **Old MCP versions**: 2024-11-05 and earlier are superseded. Use 2025-11-25 as protocol version string.

## Open Questions

Things that couldn't be fully resolved:

1. **Client protocolVersion negotiation behavior**
   - What we know: Clients send requested version in initialize; servers respond with supported version
   - What's unclear: Should server reject if client requests unsupported version, or just respond with what we support and let client decide?
   - Recommendation: Respond with "2025-11-25" regardless of client request; let client handle version incompatibility per spec. Most clients will accept any version string.

2. **Empty capabilities object detail level**
   - What we know: Servers declare `"tools": {}` to indicate tool support
   - What's unclear: Does empty object `{}` vs. detailed nested capabilities matter for tools?
   - Recommendation: Use `"tools": {}` (empty object) for Phase 3 simplicity. Detailed tool capabilities (like parameter constraints) can be added in future if needed.

3. **initialized notification timing enforcement**
   - What we know: Client should send notifications/initialized after initialize response; some clients don't
   - What's unclear: Should we enforce receipt of initialized notification before accepting tool requests?
   - Recommendation: Accept tool requests immediately after initialize completes; ignore initialized notification if received (it's a notification, no response anyway). Focus on initialize completion as the gate.

## Sources

### Primary (HIGH confidence)

- [MCP Specification 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25) - Official protocol specification
- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification) - Official JSON-RPC standard
- [MCP Versioning](https://modelcontextprotocol.io/specification/versioning) - Protocol versioning documentation
- [schemars Documentation](https://docs.rs/schemars) - JSON Schema generation library
- [Model Context Protocol GitHub](https://github.com/modelcontextprotocol/modelcontextprotocol) - Official specification repository
- [Official Rust SDK (rmcp 0.14.0)](https://github.com/modelcontextprotocol/rust-sdk) - Reference implementation

### Secondary (MEDIUM confidence)

- [MCP Error Handling Best Practices](https://mcpcat.io/guides/error-handling-custom-mcp-servers/) - Community guide verified against spec
- [How to Build an MCP Server in Rust](https://oneuptime.com/blog/post/2026-01-07-rust-mcp-server/view) - Recent tutorial (Jan 2026) with manual implementation example
- [MCP Architecture Overview](https://modelcontextprotocol.io/docs/learn/architecture) - Official architecture documentation
- [JSON-RPC Error Codes Reference](https://json-rpc.dev/docs/reference/error-codes) - Standard error code reference
- [MCP Message Types Guide](https://portkey.ai/blog/mcp-message-types-complete-json-rpc-reference-guide/) - Comprehensive message reference

### Tertiary (LOW confidence - validated with primary sources)

- [Common MCP Implementation Mistakes](https://milvus.io/ai-quick-reference/what-are-common-mistakes-developers-make-when-first-using-model-context-protocol-mcp) - Community observations (validated against spec)
- [MCP Protocol Pitfalls](https://nearform.com/digital-community/implementing-model-context-protocol-mcp-tips-tricks-and-pitfalls/) - Developer experiences (verified with official docs)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - serde/schemars are de facto Rust standards, already in project dependencies
- Architecture: HIGH - JSON-RPC 2.0 and MCP 2025-11-25 specs are authoritative and complete
- Pitfalls: MEDIUM-HIGH - Mix of spec requirements (HIGH) and community observations (MEDIUM)

**Research date:** 2026-02-02
**Valid until:** 2026-04-02 (60 days for stable protocol; MCP spec updates follow 14-day RC cycle but 2025-11-25 is current stable)

**Key verification sources:** All recommendations cross-referenced with official JSON-RPC 2.0 spec and MCP 2025-11-25 spec. schemars and serde patterns verified with official library documentation. User decisions from CONTEXT.md incorporated throughout.
