use schemars::{schema_for, JsonSchema};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::matcher::{MatchConfig, MatchError};
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
    let get_sources_schema = schema_for!(GetSourcesParams);
    let list_categories_schema = schema_for!(ListCategoriesParams);
    let get_provenance_schema = schema_for!(GetProvenanceParams);
    let get_endorsements_schema = schema_for!(GetEndorsementsParams);

    json!({
        "tools": [
            {
                "name": "get_sources",
                "description": "Find three curated, human-vetted sources for a topic. Searches across categories using fuzzy matching against known query patterns. Returns the matching category with name, description, and all three ranked sources including URLs and explanations. Example queries: 'learn rust programming', 'set up a bitcoin node', 'self-host email server'.",
                "inputSchema": serde_json::to_value(get_sources_schema).unwrap()
            },
            {
                "name": "list_categories",
                "description": "List all available topic categories in the registry. Returns each category's slug identifier, display name, and description. Use this to discover what topics have curated sources before querying. No parameters required.",
                "inputSchema": serde_json::to_value(list_categories_schema).unwrap()
            },
            {
                "name": "get_provenance",
                "description": "Get curator identity and verification information for this registry. Returns the curator's name, PKARR public key (when available), registry version, and instructions for cryptographic verification of source authenticity. No parameters required.",
                "inputSchema": serde_json::to_value(get_provenance_schema).unwrap()
            },
            {
                "name": "get_endorsements",
                "description": "Get the list of endorsed curators for this registry. In v1, this returns an empty list. Future versions will support curator endorsements with trust relationships. No parameters required.",
                "inputSchema": serde_json::to_value(get_endorsements_schema).unwrap()
            }
        ]
    })
}

/// Handle a tools/call request by dispatching to the appropriate tool
pub fn handle_tool_call(
    name: &str,
    arguments: Option<Value>,
    registry: &Registry,
    match_config: &MatchConfig,
    pubkey_z32: &str,
) -> Result<Value, ToolCallError> {
    match name {
        "get_sources" => tool_get_sources(arguments, registry, match_config),
        "list_categories" => tool_list_categories(arguments, registry),
        "get_provenance" => tool_get_provenance(arguments, registry, pubkey_z32),
        "get_endorsements" => tool_get_endorsements(arguments, registry),
        _ => Err(ToolCallError::UnknownTool),
    }
}

/// Handle get_sources tool call
///
/// Matches a natural language query against the registry categories and returns
/// the matching category with all three curated sources. Supports optional threshold
/// parameter for match sensitivity tuning.
///
/// Returns MCP content with isError: true for no match, empty query, or stop-word-only queries.
fn tool_get_sources(
    arguments: Option<Value>,
    registry: &Registry,
    match_config: &MatchConfig,
) -> Result<Value, ToolCallError> {
    // Parse arguments
    let params: GetSourcesParams = if let Some(args) = arguments {
        serde_json::from_value(args).map_err(|_| ToolCallError::InvalidParams)?
    } else {
        return Err(ToolCallError::InvalidParams);
    };

    // Create modified config if threshold provided
    let config = if let Some(threshold) = params.threshold {
        MatchConfig {
            match_threshold: threshold,
            match_fuzzy_weight: match_config.match_fuzzy_weight,
            match_keyword_weight: match_config.match_keyword_weight,
        }
    } else {
        match_config.clone()
    };

    // Attempt to match query
    let result = crate::matcher::match_query(&params.query, registry, &config);

    match result {
        Ok(match_result) => {
            // Format successful response
            let category = &match_result.category;
            let mut text = format!(
                "Category: {}\nSlug: {}\nDescription: {}\n\nRegistry Version: {}\nCurator: {} ({})\n\nSources:\n",
                category.name,
                match_result.slug,
                category.description,
                registry.version,
                registry.curator.name,
                registry.curator.pubkey
            );

            for source in &category.sources {
                text.push_str(&format!(
                    "\n{}. {}\n   URL: {}\n   Type: {:?}\n   Why: {}\n",
                    source.rank, source.name, source.url, source.source_type, source.why
                ));
            }

            Ok(json!({
                "content": [{"type": "text", "text": text}],
                "isError": false
            }))
        }
        Err(MatchError::BelowThreshold {
            closest_slug,
            closest_score,
            all_slugs,
            ..
        }) => {
            let mut slugs = all_slugs;
            slugs.sort();
            let text = format!(
                "No matching category found for query '{}'. Closest match: {} (score: {:.2}). Available categories: {}",
                params.query,
                closest_slug,
                closest_score,
                slugs.join(", ")
            );
            Ok(json!({
                "content": [{"type": "text", "text": text}],
                "isError": true
            }))
        }
        Err(MatchError::EmptyQuery) => {
            let text = "Query cannot be empty. Provide a natural language query describing what sources you need.";
            Ok(json!({
                "content": [{"type": "text", "text": text}],
                "isError": true
            }))
        }
        Err(MatchError::QueryAllStopWords) => {
            let text = "Query contains only common words (stop words) with no searchable content. Try more specific terms.";
            Ok(json!({
                "content": [{"type": "text", "text": text}],
                "isError": true
            }))
        }
    }
}

/// Handle list_categories tool call
///
/// Returns a formatted list of all available categories in the registry,
/// sorted by slug. Each entry includes the slug, display name, and description.
fn tool_list_categories(
    arguments: Option<Value>,
    registry: &Registry,
) -> Result<Value, ToolCallError> {
    // Parse arguments if provided (should be empty object or None)
    if let Some(args) = arguments {
        // Try to parse to validate structure
        let _params: ListCategoriesParams =
            serde_json::from_value(args).map_err(|_| ToolCallError::InvalidParams)?;
    }

    // Collect and sort categories by slug
    let mut categories: Vec<_> = registry.categories.iter().collect();
    categories.sort_by_key(|(slug, _)| *slug);

    let mut text = format!("Categories ({}):\n", categories.len());

    for (slug, category) in categories {
        text.push_str(&format!(
            "\n- {}: {}\n  {}\n",
            slug, category.name, category.description
        ));
    }

    Ok(json!({
        "content": [{"type": "text", "text": text}],
        "isError": false
    }))
}

/// Handle get_provenance tool call
///
/// Returns curator identity and verification information for this registry,
/// including curator name, PKARR public key, registry version, and instructions
/// for cryptographic verification.
fn tool_get_provenance(
    arguments: Option<Value>,
    registry: &Registry,
    pubkey_z32: &str,
) -> Result<Value, ToolCallError> {
    // Parse arguments if provided (should be empty object or None)
    if let Some(args) = arguments {
        let _params: GetProvenanceParams =
            serde_json::from_value(args).map_err(|_| ToolCallError::InvalidParams)?;
    }

    let text = format!(
        "Curator: {}\nPublic Key: {}\nRegistry Version: {}\nLast Updated: {}\nEndorsements: {} endorsement(s)\n\nVerification:\nThis registry is curated by {}. Source authenticity can be verified\nusing the PKARR public key above. Each source is manually researched\nand vetted for quality. The registry is cryptographically signed to\nprevent tampering.",
        registry.curator.name,
        pubkey_z32,
        registry.version,
        registry.updated,
        registry.endorsements.len(),
        registry.curator.name
    );

    Ok(json!({
        "content": [{"type": "text", "text": text}],
        "isError": false
    }))
}

/// Handle get_endorsements tool call
///
/// Returns the list of curator endorsements for this registry.
/// In v1, this always returns an empty list with an explanatory message.
fn tool_get_endorsements(
    arguments: Option<Value>,
    registry: &Registry,
) -> Result<Value, ToolCallError> {
    // Parse arguments if provided (should be empty object or None)
    if let Some(args) = arguments {
        let _params: GetEndorsementsParams =
            serde_json::from_value(args).map_err(|_| ToolCallError::InvalidParams)?;
    }

    let text = if registry.endorsements.is_empty() {
        "Endorsements: 0\n\nThis registry does not yet have any endorsements. Endorsements allow\nother curators to vouch for the quality of this registry's sources.\nThis feature will be available in a future version.".to_string()
    } else {
        // Future: list endorsements
        format!("Endorsements: {}", registry.endorsements.len())
    };

    Ok(json!({
        "content": [{"type": "text", "text": text}],
        "isError": false
    }))
}
