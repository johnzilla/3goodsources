use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::matcher::MatchConfig;
use crate::registry::Registry;

/// Tool parameter type for get_sources
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetSourcesParams {
    /// Natural language query describing what sources to find.
    /// Examples: "learn rust programming", "set up bitcoin node", "self-host email"
    pub query: String,
    /// Optional match threshold (0.0-1.0) for sensitivity tuning.
    /// Lower values return more results, higher values require closer matches.
    /// Default: 0.4
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f64>,
}

/// Tool parameter type for list_categories
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListCategoriesParams {}

/// Tool parameter type for get_provenance
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetProvenanceParams {}

/// Tool parameter type for get_endorsements
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetEndorsementsParams {}

/// Error type for tool call operations
#[derive(Debug)]
pub enum ToolCallError {
    UnknownTool,
    InvalidParams,
}

/// Get the tools/list response with all 4 tool definitions
pub fn get_tools_list() -> Value {
    unimplemented!("RED phase - not yet implemented")
}

/// Handle a tools/call request by dispatching to the appropriate tool
pub fn handle_tool_call(
    name: &str,
    arguments: Option<Value>,
    registry: &Registry,
    match_config: &MatchConfig,
) -> Result<Value, ToolCallError> {
    unimplemented!("RED phase - not yet implemented")
}
