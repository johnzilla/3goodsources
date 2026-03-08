use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AuditEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub action: AuditAction,
    pub category: Option<String>,
    pub data: serde_json::Value,
    pub actor: String,
    pub signature: String,
    pub previous_hash: Option<String>,
}

impl Default for AuditEntry {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            action: AuditAction::SourceAdded,
            category: None,
            data: serde_json::Value::Null,
            actor: String::new(),
            signature: String::new(),
            previous_hash: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    SourceAdded,
    SourceUpdated,
    SourceRemoved,
    CategoryAdded,
    CategoryUpdated,
    CategoryRemoved,
    IdentityRegistered,
    IdentityUpdated,
    ProposalSubmitted,
    ProposalStatusChanged,
    VoteCast,
}

#[derive(Debug, Deserialize)]
pub struct AuditFilterParams {
    pub since: Option<String>,
    pub category: Option<String>,
    pub action: Option<String>,
}

/// Build the canonical message for signing/verification.
/// Format: `{timestamp}|{action}|{category_or_empty}|{sha256_of_data_json}|{actor}`
pub fn canonical_message(entry: &AuditEntry) -> String {
    let data_json = serde_json::to_string(&entry.data).unwrap();
    let data_hash = hex::encode(Sha256::digest(data_json.as_bytes()));
    let category_str = entry.category.as_deref().unwrap_or("");
    let action_str = serde_json::to_value(&entry.action)
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();
    let timestamp_str = entry
        .timestamp
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    format!(
        "{}|{}|{}|{}|{}",
        timestamp_str, action_str, category_str, data_hash, entry.actor
    )
}

/// Filter audit entries by optional since, category, and action parameters.
/// Returns a new Vec containing only matching entries.
pub fn filter_entries<'a>(
    entries: &'a [AuditEntry],
    params: &AuditFilterParams,
) -> Vec<&'a AuditEntry> {
    entries
        .iter()
        .filter(|entry| {
            // Filter by since (timestamp >= since)
            if let Some(ref since_str) = params.since {
                if let Ok(since_dt) = since_str.parse::<DateTime<Utc>>() {
                    if entry.timestamp < since_dt {
                        return false;
                    }
                }
                // If since fails to parse, ignore the filter (lenient)
            }

            // Filter by category
            if let Some(ref cat) = params.category {
                match &entry.category {
                    Some(entry_cat) if entry_cat == cat => {}
                    _ => return false,
                }
            }

            // Filter by action
            if let Some(ref action_str) = params.action {
                let entry_action_str = serde_json::to_value(&entry.action)
                    .ok()
                    .and_then(|v| v.as_str().map(|s| s.to_string()));
                match entry_action_str {
                    Some(s) if s == *action_str => {}
                    _ => return false,
                }
            }

            true
        })
        .collect()
}

/// SHA-256 hash of the compact JSON serialization of an entry, hex-encoded.
/// Used for the previous_hash chain.
pub fn hash_entry_json(entry: &AuditEntry) -> String {
    let json = serde_json::to_string(entry).unwrap();
    hex::encode(Sha256::digest(json.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_audit_entry_deserializes_all_fields() {
        let json = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "timestamp": "2026-02-03T00:00:00Z",
            "action": "source_added",
            "category": "rust-learning",
            "data": {"name": "test"},
            "actor": "abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234",
            "signature": "deadbeef",
            "previous_hash": null
        }"#;
        let entry: AuditEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.action, AuditAction::SourceAdded);
        assert_eq!(entry.category, Some("rust-learning".to_string()));
        assert!(entry.previous_hash.is_none());
    }

    #[test]
    fn test_audit_action_serializes_snake_case() {
        let val = serde_json::to_value(AuditAction::SourceAdded).unwrap();
        assert_eq!(val, "source_added");
        let val = serde_json::to_value(AuditAction::CategoryAdded).unwrap();
        assert_eq!(val, "category_added");
        let val = serde_json::to_value(AuditAction::ProposalStatusChanged).unwrap();
        assert_eq!(val, "proposal_status_changed");
    }

    #[test]
    fn test_audit_action_includes_future_types() {
        // All future action types must deserialize
        for action_str in [
            "identity_registered",
            "identity_updated",
            "proposal_submitted",
            "proposal_status_changed",
            "vote_cast",
        ] {
            let json = format!("\"{}\"", action_str);
            let action: AuditAction = serde_json::from_str(&json).unwrap();
            let reserialized = serde_json::to_value(&action).unwrap();
            assert_eq!(reserialized, action_str);
        }
    }

    #[test]
    fn test_canonical_message_deterministic() {
        let entry = AuditEntry {
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            timestamp: Utc.with_ymd_and_hms(2026, 2, 3, 0, 0, 0).unwrap(),
            action: AuditAction::SourceAdded,
            category: Some("rust-learning".to_string()),
            data: serde_json::json!({"name": "test"}),
            actor: "abcd1234".to_string(),
            signature: String::new(),
            previous_hash: None,
        };
        let msg1 = canonical_message(&entry);
        let msg2 = canonical_message(&entry);
        assert_eq!(msg1, msg2);
        assert!(msg1.starts_with("2026-02-03T00:00:00Z|source_added|rust-learning|"));
        assert!(msg1.ends_with("|abcd1234"));
    }

    #[test]
    fn test_serde_default_allows_extra_fields() {
        let json = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "timestamp": "2026-02-03T00:00:00Z",
            "action": "source_added",
            "category": null,
            "data": {},
            "actor": "aa",
            "signature": "bb",
            "previous_hash": null,
            "extra_future_field": "should not cause error"
        }"#;
        let entry: AuditEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.action, AuditAction::SourceAdded);
    }

    #[test]
    fn test_category_option_null_deserializes() {
        let json = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "timestamp": "2026-02-03T00:00:00Z",
            "action": "vote_cast",
            "category": null,
            "data": {},
            "actor": "aa",
            "signature": "bb",
            "previous_hash": null
        }"#;
        let entry: AuditEntry = serde_json::from_str(json).unwrap();
        assert!(entry.category.is_none());
    }

    #[test]
    fn test_timestamp_z_suffix_in_canonical() {
        let entry = AuditEntry {
            id: Uuid::new_v4(),
            timestamp: Utc.with_ymd_and_hms(2026, 2, 3, 0, 0, 0).unwrap(),
            action: AuditAction::CategoryAdded,
            category: None,
            data: serde_json::json!({}),
            actor: "aa".to_string(),
            signature: String::new(),
            previous_hash: None,
        };
        let msg = canonical_message(&entry);
        assert!(msg.starts_with("2026-02-03T00:00:00Z|"), "Expected Z suffix, got: {}", msg);
    }
}
