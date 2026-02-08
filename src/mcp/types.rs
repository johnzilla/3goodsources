use serde::{Deserialize, Serialize};
use serde_json::Value;

// JSON-RPC 2.0 error codes
pub const PARSE_ERROR: i32 = -32700;
pub const INVALID_REQUEST: i32 = -32600;
pub const METHOD_NOT_FOUND: i32 = -32601;
pub const INVALID_PARAMS: i32 = -32602;
pub const NOT_INITIALIZED: i32 = -32002;

/// JSON-RPC 2.0 request message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// JSON-RPC 2.0 response message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    /// Create a success response
    pub fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response
    pub fn error(id: Value, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: None,
            }),
        }
    }

    /// Parse error (-32700)
    pub fn parse_error() -> Self {
        Self::error(Value::Null, PARSE_ERROR, "Parse error".to_string())
    }

    /// Invalid request (-32600)
    pub fn invalid_request() -> Self {
        Self::error(Value::Null, INVALID_REQUEST, "Invalid request".to_string())
    }

    /// Method not found (-32601)
    pub fn method_not_found(id: Value) -> Self {
        Self::error(id, METHOD_NOT_FOUND, "Method not found".to_string())
    }

    /// Invalid params (-32602)
    pub fn invalid_params(id: Value) -> Self {
        Self::error(id, INVALID_PARAMS, "Invalid params".to_string())
    }

    /// Server not initialized (-32002)
    pub fn not_initialized(id: Value) -> Self {
        Self::error(
            id,
            NOT_INITIALIZED,
            "Server not initialized. Call initialize first.".to_string(),
        )
    }
}

/// JSON-RPC 2.0 error object
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// MCP initialize request parameters
/// Fields are part of MCP protocol spec and used by serde deserialization
#[derive(Debug, Clone, Deserialize)]
pub struct InitializeParams {
    #[serde(rename = "protocolVersion")]
    #[allow(dead_code)]
    pub protocol_version: String,
    #[allow(dead_code)]
    pub capabilities: Value,
    #[serde(rename = "clientInfo")]
    #[allow(dead_code)]
    pub client_info: Value,
}

/// MCP tools/call request parameters
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CallToolParams {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Value>,
}
