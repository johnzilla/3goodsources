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
}
