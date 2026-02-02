use crate::matcher::MatchConfig;
use crate::mcp::types::{InitializeParams, JsonRpcRequest, JsonRpcResponse};
use crate::registry::Registry;
use serde_json::Value;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct McpHandler {
    initialized: Arc<AtomicBool>,
    #[allow(dead_code)] // Used in Plan 02
    registry: Arc<Registry>,
    #[allow(dead_code)] // Used in Plan 02
    match_config: MatchConfig,
}

impl McpHandler {
    pub fn new(registry: Arc<Registry>, match_config: MatchConfig) -> Self {
        Self {
            initialized: Arc::new(AtomicBool::new(false)),
            registry,
            match_config,
        }
    }

    /// Main entry point for handling JSON-RPC requests
    /// Returns None for notifications (no response needed)
    pub fn handle_json(&self, raw_json: &str) -> Option<String> {
        // Parse raw JSON
        let parsed: Value = match serde_json::from_str(raw_json) {
            Ok(v) => v,
            Err(_) => {
                return Some(self.serialize_response(JsonRpcResponse::parse_error()));
            }
        };

        // Reject batch requests (arrays)
        if parsed.is_array() {
            let mut response = JsonRpcResponse::invalid_request();
            if let Some(error) = response.error.as_mut() {
                error.message = "Batch requests not supported".to_string();
            }
            return Some(self.serialize_response(response));
        }

        // Deserialize into JsonRpcRequest
        let request: JsonRpcRequest = match serde_json::from_value(parsed) {
            Ok(r) => r,
            Err(_) => {
                return Some(self.serialize_response(JsonRpcResponse::parse_error()));
            }
        };

        // Validate jsonrpc field
        if request.jsonrpc != "2.0" {
            return Some(self.serialize_response(JsonRpcResponse::invalid_request()));
        }

        // Check if this is a notification (no id field)
        let id = request.id?; // Notification - silently ignore

        // Check initialization gate (except for initialize method itself)
        if request.method != "initialize"
            && request.method != "notifications/initialized"
            && !self.initialized.load(Ordering::SeqCst)
        {
            return Some(self.serialize_response(JsonRpcResponse::not_initialized(id)));
        }

        // Dispatch to method handlers
        match request.method.as_str() {
            "initialize" => self.handle_initialize(id, request.params),
            "notifications/initialized" => None, // Client notification - ignore
            "tools/list" => self.handle_tools_list(id, request.params),
            "tools/call" => self.handle_tools_call(id, request.params),
            _ => Some(self.serialize_response(JsonRpcResponse::method_not_found(id))),
        }
    }

    fn handle_initialize(&self, id: Value, params: Option<Value>) -> Option<String> {
        // Deserialize params
        let _init_params: InitializeParams = match params {
            Some(p) => match serde_json::from_value(p) {
                Ok(params) => params,
                Err(_) => {
                    return Some(self.serialize_response(JsonRpcResponse::invalid_params(id)));
                }
            },
            None => {
                return Some(self.serialize_response(JsonRpcResponse::invalid_params(id)));
            }
        };

        // Set initialized flag
        self.initialized.store(true, Ordering::SeqCst);

        // Build response
        let result = serde_json::json!({
            "protocolVersion": "2025-11-25",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "three-good-sources",
                "version": env!("CARGO_PKG_VERSION")
            }
        });

        Some(self.serialize_response(JsonRpcResponse::success(id, result)))
    }

    /// Stub for tools/list - Plan 02 will implement
    fn handle_tools_list(&self, id: Value, _params: Option<Value>) -> Option<String> {
        Some(self.serialize_response(JsonRpcResponse::method_not_found(id)))
    }

    /// Stub for tools/call - Plan 02 will implement
    fn handle_tools_call(&self, id: Value, _params: Option<Value>) -> Option<String> {
        Some(self.serialize_response(JsonRpcResponse::method_not_found(id)))
    }

    fn serialize_response(&self, response: JsonRpcResponse) -> String {
        serde_json::to_string(&response).unwrap_or_else(|_| {
            r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"Internal error"}}"#
                .to_string()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matcher::MatchConfig;
    use crate::registry::Registry;

    fn test_handler() -> McpHandler {
        let registry_json = include_str!("../../registry.json");
        let registry: Registry = serde_json::from_str(registry_json)
            .expect("Failed to parse test registry.json");

        let match_config = MatchConfig {
            match_threshold: 0.4,
            match_fuzzy_weight: 0.7,
            match_keyword_weight: 0.3,
        };

        McpHandler::new(Arc::new(registry), match_config)
    }

    #[test]
    fn test_initialize_returns_protocol_version() {
        let handler = test_handler();

        let request = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": {"name": "test-client", "version": "1.0"}
            }
        }"#;

        let response_str = handler.handle_json(request).expect("Expected response");
        let response: Value = serde_json::from_str(&response_str).expect("Valid JSON");

        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["result"]["protocolVersion"], "2025-11-25");
        assert!(response["result"]["capabilities"]["tools"].is_object());
        assert_eq!(response["result"]["serverInfo"]["name"], "three-good-sources");
        assert!(response["result"]["serverInfo"]["version"].is_string());
    }

    #[test]
    fn test_initialize_sets_initialized_flag() {
        let handler = test_handler();
        assert!(!handler.initialized.load(Ordering::SeqCst));

        let request = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1.0"}
            }
        }"#;

        handler.handle_json(request);
        assert!(handler.initialized.load(Ordering::SeqCst));
    }

    #[test]
    fn test_pre_init_tools_list_rejected() {
        let handler = test_handler();

        let request = r#"{
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list"
        }"#;

        let response_str = handler.handle_json(request).expect("Expected response");
        let response: Value = serde_json::from_str(&response_str).expect("Valid JSON");

        assert_eq!(response["error"]["code"], -32002);
        assert!(response["error"]["message"]
            .as_str()
            .unwrap()
            .contains("not initialized"));
    }

    #[test]
    fn test_pre_init_tools_call_rejected() {
        let handler = test_handler();

        let request = r#"{
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {"name": "get_sources", "arguments": {}}
        }"#;

        let response_str = handler.handle_json(request).expect("Expected response");
        let response: Value = serde_json::from_str(&response_str).expect("Valid JSON");

        assert_eq!(response["error"]["code"], -32002);
    }

    #[test]
    fn test_batch_request_rejected() {
        let handler = test_handler();

        let request = r#"[
            {"jsonrpc": "2.0", "id": 1, "method": "initialize"},
            {"jsonrpc": "2.0", "id": 2, "method": "tools/list"}
        ]"#;

        let response_str = handler.handle_json(request).expect("Expected response");
        let response: Value = serde_json::from_str(&response_str).expect("Valid JSON");

        assert_eq!(response["error"]["code"], -32600);
        assert_eq!(response["error"]["message"], "Batch requests not supported");
    }

    #[test]
    fn test_notification_returns_none() {
        let handler = test_handler();

        // Request without id field is a notification
        let request = r#"{
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        }"#;

        let response = handler.handle_json(request);
        assert!(response.is_none());
    }

    #[test]
    fn test_unknown_method_returns_error() {
        let handler = test_handler();

        // Initialize first
        let init_request = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1.0"}
            }
        }"#;
        handler.handle_json(init_request);

        // Try unknown method
        let request = r#"{
            "jsonrpc": "2.0",
            "id": 2,
            "method": "unknown/method"
        }"#;

        let response_str = handler.handle_json(request).expect("Expected response");
        let response: Value = serde_json::from_str(&response_str).expect("Valid JSON");

        assert_eq!(response["error"]["code"], -32601);
    }

    #[test]
    fn test_invalid_json_returns_parse_error() {
        let handler = test_handler();

        let request = r#"{ invalid json }"#;

        let response_str = handler.handle_json(request).expect("Expected response");
        let response: Value = serde_json::from_str(&response_str).expect("Valid JSON");

        assert_eq!(response["error"]["code"], -32700);
    }

    #[test]
    fn test_all_responses_have_jsonrpc_field() {
        let handler = test_handler();

        let test_cases = vec![
            // Parse error
            r#"{ invalid }"#,
            // Batch request
            r#"[]"#,
            // Pre-init rejection
            r#"{"jsonrpc": "2.0", "id": 1, "method": "tools/list"}"#,
            // Valid initialize
            r#"{"jsonrpc": "2.0", "id": 2, "method": "initialize", "params": {"protocolVersion": "2025-11-25", "capabilities": {}, "clientInfo": {}}}"#,
        ];

        for request in test_cases {
            if let Some(response_str) = handler.handle_json(request) {
                let response: Value = serde_json::from_str(&response_str).expect("Valid JSON");
                assert_eq!(
                    response["jsonrpc"], "2.0",
                    "Response missing jsonrpc field for request: {}",
                    request
                );
            }
        }
    }

    #[test]
    fn test_invalid_params_on_initialize() {
        let handler = test_handler();

        // Missing protocolVersion field
        let request = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1.0"}
            }
        }"#;

        let response_str = handler.handle_json(request).expect("Expected response");
        let response: Value = serde_json::from_str(&response_str).expect("Valid JSON");

        assert_eq!(response["error"]["code"], -32602);
    }

    #[test]
    fn test_invalid_jsonrpc_version() {
        let handler = test_handler();

        let request = r#"{
            "jsonrpc": "1.0",
            "id": 1,
            "method": "initialize"
        }"#;

        let response_str = handler.handle_json(request).expect("Expected response");
        let response: Value = serde_json::from_str(&response_str).expect("Valid JSON");

        assert_eq!(response["error"]["code"], -32600);
    }

    // Helper function to initialize a handler
    fn init_handler(handler: &McpHandler) {
        let init_request = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1.0"}
            }
        }"#;
        handler.handle_json(init_request).expect("Initialize should succeed");
    }

    // ===== TDD Tests for Plan 02: Tool Implementations =====

    #[test]
    fn test_tools_list_returns_four_tools() {
        let handler = test_handler();
        init_handler(&handler);

        let request = r#"{
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list"
        }"#;

        let response_str = handler.handle_json(request).expect("Expected response");
        let response: Value = serde_json::from_str(&response_str).expect("Valid JSON");

        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], 2);
        assert!(response["result"]["tools"].is_array());

        let tools = response["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 4, "Should return exactly 4 tools");

        // Check tool names
        let tool_names: Vec<&str> = tools
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        assert!(tool_names.contains(&"get_sources"));
        assert!(tool_names.contains(&"list_categories"));
        assert!(tool_names.contains(&"get_provenance"));
        assert!(tool_names.contains(&"get_endorsements"));
    }

    #[test]
    fn test_tools_list_has_input_schemas() {
        let handler = test_handler();
        init_handler(&handler);

        let request = r#"{
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list"
        }"#;

        let response_str = handler.handle_json(request).expect("Expected response");
        let response: Value = serde_json::from_str(&response_str).expect("Valid JSON");

        let tools = response["result"]["tools"].as_array().unwrap();

        for tool in tools {
            assert!(tool["name"].is_string(), "Tool should have name");
            assert!(tool["description"].is_string(), "Tool should have description");
            assert!(tool["inputSchema"].is_object(), "Tool should have inputSchema");
            assert_eq!(
                tool["inputSchema"]["type"].as_str().unwrap(),
                "object",
                "inputSchema should be object type"
            );
        }
    }

    #[test]
    fn test_get_sources_success() {
        let handler = test_handler();
        init_handler(&handler);

        let request = r#"{
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "get_sources",
                "arguments": {"query": "learn rust"}
            }
        }"#;

        let response_str = handler.handle_json(request).expect("Expected response");
        let response: Value = serde_json::from_str(&response_str).expect("Valid JSON");

        assert_eq!(response["id"], 3);
        assert!(response["result"]["content"].is_array());

        let content = &response["result"]["content"][0];
        assert_eq!(content["type"], "text");

        let text = content["text"].as_str().unwrap();
        assert!(text.contains("Rust Learning"), "Should contain category name");
        assert!(
            text.matches("http").count() >= 3,
            "Should contain 3 source URLs"
        );

        assert_eq!(response["result"]["isError"], false);
    }

    #[test]
    fn test_get_sources_includes_registry_metadata() {
        let handler = test_handler();
        init_handler(&handler);

        let request = r#"{
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "get_sources",
                "arguments": {"query": "learn rust"}
            }
        }"#;

        let response_str = handler.handle_json(request).expect("Expected response");
        let response: Value = serde_json::from_str(&response_str).expect("Valid JSON");

        let text = response["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("Registry Version:"), "Should include registry version");
        assert!(text.contains("Curator:"), "Should include curator name");
    }

    #[test]
    fn test_get_sources_no_match() {
        let handler = test_handler();
        init_handler(&handler);

        let request = r#"{
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": {
                "name": "get_sources",
                "arguments": {"query": "quantum physics supercollider"}
            }
        }"#;

        let response_str = handler.handle_json(request).expect("Expected response");
        let response: Value = serde_json::from_str(&response_str).expect("Valid JSON");

        assert_eq!(response["result"]["isError"], true);

        let text = response["result"]["content"][0]["text"].as_str().unwrap();
        assert!(
            text.contains("No matching category") || text.contains("Available categories"),
            "Should explain no match and show available categories"
        );
    }

    #[test]
    fn test_get_sources_empty_query() {
        let handler = test_handler();
        init_handler(&handler);

        let request = r#"{
            "jsonrpc": "2.0",
            "id": 6,
            "method": "tools/call",
            "params": {
                "name": "get_sources",
                "arguments": {"query": ""}
            }
        }"#;

        let response_str = handler.handle_json(request).expect("Expected response");
        let response: Value = serde_json::from_str(&response_str).expect("Valid JSON");

        assert_eq!(response["result"]["isError"], true);

        let text = response["result"]["content"][0]["text"].as_str().unwrap();
        assert!(
            text.contains("empty") || text.contains("cannot be empty"),
            "Should explain empty query error"
        );
    }

    #[test]
    fn test_get_sources_custom_threshold() {
        let handler = test_handler();
        init_handler(&handler);

        // With threshold 0.9, "learn rust" should fail to match (score likely < 0.9)
        let request = r#"{
            "jsonrpc": "2.0",
            "id": 7,
            "method": "tools/call",
            "params": {
                "name": "get_sources",
                "arguments": {"query": "learn rust", "threshold": 0.9}
            }
        }"#;

        let response_str = handler.handle_json(request).expect("Expected response");
        let response: Value = serde_json::from_str(&response_str).expect("Valid JSON");

        // Should return error due to high threshold
        assert_eq!(response["result"]["isError"], true);
    }

    #[test]
    fn test_list_categories_returns_all() {
        let handler = test_handler();
        init_handler(&handler);

        let request = r#"{
            "jsonrpc": "2.0",
            "id": 8,
            "method": "tools/call",
            "params": {
                "name": "list_categories",
                "arguments": {}
            }
        }"#;

        let response_str = handler.handle_json(request).expect("Expected response");
        let response: Value = serde_json::from_str(&response_str).expect("Valid JSON");

        assert_eq!(response["result"]["isError"], false);

        let text = response["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("rust-learning"), "Should include rust-learning slug");
        assert!(text.contains("bitcoin-node-setup"), "Should include bitcoin-node-setup slug");

        // Count how many category slugs are mentioned (should be 10)
        let slug_count = text.matches("-").count();
        assert!(slug_count >= 10, "Should mention all 10 categories");
    }

    #[test]
    fn test_get_provenance_returns_curator() {
        let handler = test_handler();
        init_handler(&handler);

        let request = r#"{
            "jsonrpc": "2.0",
            "id": 9,
            "method": "tools/call",
            "params": {
                "name": "get_provenance",
                "arguments": {}
            }
        }"#;

        let response_str = handler.handle_json(request).expect("Expected response");
        let response: Value = serde_json::from_str(&response_str).expect("Valid JSON");

        assert_eq!(response["result"]["isError"], false);

        let text = response["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("John Turner"), "Should include curator name");
        assert!(text.contains("Curator:"), "Should have Curator label");
    }

    #[test]
    fn test_get_endorsements_empty_v1() {
        let handler = test_handler();
        init_handler(&handler);

        let request = r#"{
            "jsonrpc": "2.0",
            "id": 10,
            "method": "tools/call",
            "params": {
                "name": "get_endorsements",
                "arguments": {}
            }
        }"#;

        let response_str = handler.handle_json(request).expect("Expected response");
        let response: Value = serde_json::from_str(&response_str).expect("Valid JSON");

        assert_eq!(response["result"]["isError"], false);

        let text = response["result"]["content"][0]["text"].as_str().unwrap();
        assert!(
            text.contains("no endorsements") || text.contains("Endorsements: 0"),
            "Should indicate no endorsements"
        );
    }

    #[test]
    fn test_unknown_tool_returns_error() {
        let handler = test_handler();
        init_handler(&handler);

        let request = r#"{
            "jsonrpc": "2.0",
            "id": 11,
            "method": "tools/call",
            "params": {
                "name": "unknown_tool",
                "arguments": {}
            }
        }"#;

        let response_str = handler.handle_json(request).expect("Expected response");
        let response: Value = serde_json::from_str(&response_str).expect("Valid JSON");

        assert_eq!(response["error"]["code"], -32601, "Should be Method not found");
    }

    #[test]
    fn test_invalid_tool_params() {
        let handler = test_handler();
        init_handler(&handler);

        // get_sources with extra unknown field
        let request = r#"{
            "jsonrpc": "2.0",
            "id": 12,
            "method": "tools/call",
            "params": {
                "name": "get_sources",
                "arguments": {"query": "test", "extra_field": "invalid"}
            }
        }"#;

        let response_str = handler.handle_json(request).expect("Expected response");
        let response: Value = serde_json::from_str(&response_str).expect("Valid JSON");

        assert_eq!(response["error"]["code"], -32602, "Should be Invalid params");
    }

    #[test]
    fn test_get_sources_missing_query() {
        let handler = test_handler();
        init_handler(&handler);

        // get_sources without query parameter
        let request = r#"{
            "jsonrpc": "2.0",
            "id": 13,
            "method": "tools/call",
            "params": {
                "name": "get_sources",
                "arguments": {}
            }
        }"#;

        let response_str = handler.handle_json(request).expect("Expected response");
        let response: Value = serde_json::from_str(&response_str).expect("Valid JSON");

        assert_eq!(response["error"]["code"], -32602, "Should be Invalid params");
    }
}
