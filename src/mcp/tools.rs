use schemars::{schema_for, JsonSchema};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::audit::{filter_entries, AuditEntry, AuditFilterParams};
use crate::contributions::Proposal;
use crate::identity::{Identity, IdentityType};
use crate::matcher::{MatchConfig, MatchError};
use crate::registry::Registry;
use std::collections::HashMap;
use uuid::Uuid;

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

/// Tool parameter type for get_audit_log
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetAuditLogParams {
    /// Filter entries after this ISO 8601 timestamp (e.g. "2026-02-03T00:00:00Z")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
    /// Filter entries by category slug (e.g. "rust-learning")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Filter entries by action type (e.g. "source_added", "category_added")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
}

/// Tool parameter type for get_identity
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetIdentityParams {
    /// PKARR public key (64-character hex string) to look up
    pub pubkey: String,
}

/// Tool parameter type for list_proposals
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListProposalsParams {
    /// Filter by proposal status (pending, approved, rejected, withdrawn)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Filter by category slug (e.g. "rust-learning")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

/// Tool parameter type for get_proposal
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetProposalParams {
    /// Proposal UUID
    pub id: String,
}

/// Error type for tool call operations
#[derive(Debug)]
pub enum ToolCallError {
    UnknownTool,
    InvalidParams,
}

/// Get the tools/list response with all 8 tool definitions
pub fn get_tools_list() -> Value {
    let get_sources_schema = schema_for!(GetSourcesParams);
    let list_categories_schema = schema_for!(ListCategoriesParams);
    let get_provenance_schema = schema_for!(GetProvenanceParams);
    let get_endorsements_schema = schema_for!(GetEndorsementsParams);
    let get_audit_log_schema = schema_for!(GetAuditLogParams);
    let get_identity_schema = schema_for!(GetIdentityParams);
    let list_proposals_schema = schema_for!(ListProposalsParams);
    let get_proposal_schema = schema_for!(GetProposalParams);

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
            },
            {
                "name": "get_audit_log",
                "description": "Get the public audit log of all registry changes. Returns signed, hash-chained entries showing when sources and categories were added, updated, or removed. Supports optional filtering by timestamp (since), category slug (category), and action type (action).",
                "inputSchema": serde_json::to_value(get_audit_log_schema).unwrap()
            },
            {
                "name": "get_identity",
                "description": "Look up a registered identity by PKARR public key. Returns the identity's display name, type (human/bot), linked platform handles with proof URLs for independent verification, and operator info for bot identities.",
                "inputSchema": serde_json::to_value(get_identity_schema).unwrap()
            },
            {
                "name": "list_proposals",
                "description": "List community proposals for source changes. Returns proposal summaries with optional filtering by status (pending, approved, rejected, withdrawn) and category slug.",
                "inputSchema": serde_json::to_value(list_proposals_schema).unwrap()
            },
            {
                "name": "get_proposal",
                "description": "Get full details of a community proposal by UUID, including all votes with voter pubkeys and timestamps.",
                "inputSchema": serde_json::to_value(get_proposal_schema).unwrap()
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
    audit_log: &[AuditEntry],
    identities: &HashMap<String, Identity>,
    proposals: &HashMap<Uuid, Proposal>,
) -> Result<Value, ToolCallError> {
    match name {
        "get_sources" => tool_get_sources(arguments, registry, match_config),
        "list_categories" => tool_list_categories(arguments, registry),
        "get_provenance" => tool_get_provenance(arguments, registry, pubkey_z32),
        "get_endorsements" => tool_get_endorsements(arguments, registry),
        "get_audit_log" => tool_get_audit_log(arguments, audit_log),
        "get_identity" => tool_get_identity(arguments, identities),
        "list_proposals" => tool_list_proposals(arguments, proposals),
        "get_proposal" => tool_get_proposal(arguments, proposals),
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

/// Handle get_audit_log tool call
///
/// Returns audit log entries with optional filtering by since, category, and action.
/// Formats entries as human-readable text with entry count header.
fn tool_get_audit_log(
    arguments: Option<Value>,
    audit_log: &[AuditEntry],
) -> Result<Value, ToolCallError> {
    // Parse arguments
    let params: GetAuditLogParams = if let Some(args) = arguments {
        serde_json::from_value(args).map_err(|_| ToolCallError::InvalidParams)?
    } else {
        // No arguments means no filters
        GetAuditLogParams {
            since: None,
            category: None,
            action: None,
        }
    };

    // Convert to AuditFilterParams for shared filter logic
    let filter_params = AuditFilterParams {
        since: params.since,
        category: params.category,
        action: params.action,
    };

    let filtered = filter_entries(audit_log, &filter_params);

    let mut text = format!("Audit Log ({} entries):\n", filtered.len());

    for entry in &filtered {
        let action_str = serde_json::to_value(&entry.action)
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "unknown".to_string());

        let actor_display = if entry.actor.len() > 16 {
            format!("{}...", &entry.actor[..16])
        } else {
            entry.actor.clone()
        };

        let category_display = entry
            .category
            .as_deref()
            .unwrap_or("(none)");

        text.push_str(&format!(
            "\n- {} | {} | {} | category: {} | actor: {}",
            entry.id,
            entry.timestamp.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            action_str,
            category_display,
            actor_display,
        ));
    }

    Ok(json!({
        "content": [{"type": "text", "text": text}],
        "isError": false
    }))
}

/// Handle get_identity tool call
///
/// Looks up an identity by PKARR public key. Returns formatted identity info
/// including name, type, platform claims with proof URLs, and operator info for bots.
fn tool_get_identity(
    arguments: Option<Value>,
    identities: &HashMap<String, Identity>,
) -> Result<Value, ToolCallError> {
    let params: GetIdentityParams = if let Some(args) = arguments {
        serde_json::from_value(args).map_err(|_| ToolCallError::InvalidParams)?
    } else {
        return Err(ToolCallError::InvalidParams);
    };

    match identities.get(&params.pubkey) {
        Some(identity) => {
            let type_str = match identity.identity_type {
                IdentityType::Human => "human",
                IdentityType::Bot => "bot",
            };

            let mut text = format!(
                "Identity: {}\nType: {}\nPubkey: {}\n\nPlatforms:\n",
                identity.name, type_str, params.pubkey
            );

            for claim in &identity.platforms {
                let platform_str = format!("{:?}", claim.platform).to_lowercase();
                text.push_str(&format!(
                    "- {}: {} (proof: {})\n",
                    platform_str, claim.handle, claim.proof_url
                ));
            }

            if identity.identity_type == IdentityType::Bot {
                if let Some(ref op_key) = identity.operator_pubkey {
                    text.push_str(&format!("\nOperator: {}\n", op_key));
                }
            }

            Ok(json!({
                "content": [{"type": "text", "text": text}],
                "isError": false
            }))
        }
        None => {
            let text = format!("No identity found for pubkey: {}", params.pubkey);
            Ok(json!({
                "content": [{"type": "text", "text": text}],
                "isError": true
            }))
        }
    }
}

/// Handle list_proposals tool call
///
/// Lists proposals with optional filtering by status and category.
/// Returns human-readable text with proposal count and summary lines.
fn tool_list_proposals(
    arguments: Option<Value>,
    proposals: &HashMap<Uuid, Proposal>,
) -> Result<Value, ToolCallError> {
    let params: ListProposalsParams = if let Some(args) = arguments {
        serde_json::from_value(args).map_err(|_| ToolCallError::InvalidParams)?
    } else {
        ListProposalsParams {
            status: None,
            category: None,
        }
    };

    let mut entries: Vec<(Uuid, &Proposal)> = proposals
        .iter()
        .filter(|(_, proposal)| {
            if let Some(ref status_filter) = params.status {
                let status_str = serde_json::to_value(&proposal.status)
                    .ok()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .unwrap_or_default();
                if status_str != *status_filter {
                    return false;
                }
            }
            if let Some(ref cat_filter) = params.category {
                if proposal.category != *cat_filter {
                    return false;
                }
            }
            true
        })
        .map(|(id, p)| (*id, p))
        .collect();

    // Sort by created_at descending
    entries.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));

    let mut text = format!("Proposals ({}):\n", entries.len());

    for (id, proposal) in &entries {
        let action_str = serde_json::to_value(&proposal.action)
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "unknown".to_string());

        let status_str = serde_json::to_value(&proposal.status)
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "unknown".to_string());

        let proposer_display = if proposal.proposer.len() > 16 {
            format!("{}...", &proposal.proposer[..16])
        } else {
            proposal.proposer.clone()
        };

        text.push_str(&format!(
            "- {} | {} | {} | {} | by: {} | {}\n",
            id,
            action_str,
            status_str,
            proposal.category,
            proposer_display,
            proposal.created_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        ));
    }

    Ok(json!({
        "content": [{"type": "text", "text": text}],
        "isError": false
    }))
}

/// Handle get_proposal tool call
///
/// Returns full proposal detail for a given UUID including votes.
fn tool_get_proposal(
    arguments: Option<Value>,
    proposals: &HashMap<Uuid, Proposal>,
) -> Result<Value, ToolCallError> {
    let params: GetProposalParams = if let Some(args) = arguments {
        serde_json::from_value(args).map_err(|_| ToolCallError::InvalidParams)?
    } else {
        return Err(ToolCallError::InvalidParams);
    };

    let uuid = Uuid::parse_str(&params.id).map_err(|_| ToolCallError::InvalidParams)?;

    match proposals.get(&uuid) {
        Some(proposal) => {
            let action_str = serde_json::to_value(&proposal.action)
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "unknown".to_string());

            let status_str = serde_json::to_value(&proposal.status)
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "unknown".to_string());

            let data_pretty = serde_json::to_string_pretty(&proposal.data)
                .unwrap_or_else(|_| "{}".to_string());

            let mut text = format!(
                "Proposal: {}\nAction: {}\nStatus: {}\nCategory: {}\nProposer: {}\nCreated: {}\n\nData:\n{}\n\nVotes ({}):\n",
                uuid,
                action_str,
                status_str,
                proposal.category,
                proposal.proposer,
                proposal.created_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                data_pretty,
                proposal.votes.len(),
            );

            for vote in &proposal.votes {
                let voter_display = if vote.voter.len() > 16 {
                    format!("{}...", &vote.voter[..16])
                } else {
                    vote.voter.clone()
                };

                let vote_str = serde_json::to_value(&vote.vote)
                    .ok()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| "unknown".to_string());

                text.push_str(&format!(
                    "- {} | {} | {}\n",
                    voter_display,
                    vote_str,
                    vote.timestamp.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                ));
            }

            Ok(json!({
                "content": [{"type": "text", "text": text}],
                "isError": false
            }))
        }
        None => {
            let text = format!("No proposal found for id: {}", uuid);
            Ok(json!({
                "content": [{"type": "text", "text": text}],
                "isError": true
            }))
        }
    }
}
