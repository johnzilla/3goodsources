use super::error::ContributionError;
use super::types::Proposal;
use crate::identity::types::Identity;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use uuid::Uuid;

/// Load contributions from a JSON file on disk.
/// Validates that all voter pubkeys reference existing identities.
pub async fn load(
    path: impl AsRef<Path>,
    identities: &HashMap<String, Identity>,
) -> Result<HashMap<Uuid, Proposal>, ContributionError> {
    let path = path.as_ref();
    let path_str = path.display().to_string();

    let contents = fs::read_to_string(path)
        .await
        .map_err(|e| ContributionError::FileRead {
            path: path_str.clone(),
            error: e.to_string(),
        })?;

    let proposals: HashMap<Uuid, Proposal> =
        serde_json::from_str(&contents).map_err(|e| ContributionError::JsonParse {
            path: path_str.clone(),
            error: e.to_string(),
        })?;

    // Validate voter pubkeys against identities
    for (proposal_id, proposal) in &proposals {
        for vote in &proposal.votes {
            if !identities.contains_key(&vote.voter) {
                return Err(ContributionError::UnknownVoter {
                    voter_pubkey: vote.voter.clone(),
                    proposal_id: proposal_id.to_string(),
                });
            }
        }
    }

    tracing::info!(
        proposals = proposals.len(),
        "Contributions loaded successfully"
    );
    Ok(proposals)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::types::{Identity, IdentityType};

    fn test_identity(name: &str) -> Identity {
        Identity {
            name: name.to_string(),
            identity_type: IdentityType::Human,
            platforms: vec![],
            operator_pubkey: None,
        }
    }

    fn test_identities() -> HashMap<String, Identity> {
        let mut map = HashMap::new();
        map.insert("voter_pubkey_1".to_string(), test_identity("Alice"));
        map.insert("voter_pubkey_2".to_string(), test_identity("Bob"));
        map
    }

    #[tokio::test]
    async fn test_load_valid_proposals() {
        let json = r#"{
            "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d": {
                "action": "add_source",
                "status": "pending",
                "category": "rust",
                "proposer": "voter_pubkey_1",
                "created_at": "2026-03-08T12:00:00Z",
                "data": {"name": "Test Source"},
                "votes": [
                    {
                        "voter": "voter_pubkey_1",
                        "vote": "support",
                        "timestamp": "2026-03-08T12:30:00Z"
                    }
                ]
            }
        }"#;

        let tmp = std::env::temp_dir().join("test_contributions_valid.json");
        tokio::fs::write(&tmp, json).await.unwrap();

        let identities = test_identities();
        let result = load(&tmp, &identities).await;
        assert!(
            result.is_ok(),
            "Should load valid proposals: {:?}",
            result
        );
        let proposals = result.unwrap();
        assert_eq!(proposals.len(), 1);

        let _ = tokio::fs::remove_file(&tmp).await;
    }

    #[tokio::test]
    async fn test_load_empty_json_object() {
        let tmp = std::env::temp_dir().join("test_contributions_empty.json");
        tokio::fs::write(&tmp, "{}").await.unwrap();

        let identities = test_identities();
        let result = load(&tmp, &identities).await;
        assert!(
            result.is_ok(),
            "Should load empty JSON object: {:?}",
            result
        );
        assert_eq!(result.unwrap().len(), 0);

        let _ = tokio::fs::remove_file(&tmp).await;
    }

    #[tokio::test]
    async fn test_load_unknown_voter_rejected() {
        let json = r#"{
            "a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d": {
                "action": "add_source",
                "status": "pending",
                "category": "rust",
                "proposer": "voter_pubkey_1",
                "created_at": "2026-03-08T12:00:00Z",
                "data": {},
                "votes": [
                    {
                        "voter": "unknown_pubkey",
                        "vote": "support",
                        "timestamp": "2026-03-08T12:30:00Z"
                    }
                ]
            }
        }"#;

        let tmp = std::env::temp_dir().join("test_contributions_unknown_voter.json");
        tokio::fs::write(&tmp, json).await.unwrap();

        let identities = test_identities();
        let result = load(&tmp, &identities).await;
        assert!(result.is_err(), "Should reject unknown voter");
        assert!(
            matches!(result, Err(ContributionError::UnknownVoter { .. })),
            "Expected UnknownVoter error"
        );

        let _ = tokio::fs::remove_file(&tmp).await;
    }

    #[tokio::test]
    async fn test_load_invalid_json() {
        let tmp = std::env::temp_dir().join("test_contributions_invalid.json");
        tokio::fs::write(&tmp, "not valid json!!!").await.unwrap();

        let identities = test_identities();
        let result = load(&tmp, &identities).await;
        assert!(result.is_err(), "Should reject invalid JSON");
        assert!(
            matches!(result, Err(ContributionError::JsonParse { .. })),
            "Expected JsonParse error"
        );

        let _ = tokio::fs::remove_file(&tmp).await;
    }

    #[tokio::test]
    async fn test_load_file_not_found() {
        let identities = test_identities();
        let result = load("/tmp/nonexistent_contributions_file_12345.json", &identities).await;
        assert!(result.is_err(), "Should error on missing file");
        assert!(
            matches!(result, Err(ContributionError::FileRead { .. })),
            "Expected FileRead error"
        );
    }

    #[test]
    fn test_proposal_status_deserializes() {
        use crate::contributions::types::ProposalStatus;
        let cases = vec![
            ("\"pending\"", ProposalStatus::Pending),
            ("\"approved\"", ProposalStatus::Approved),
            ("\"rejected\"", ProposalStatus::Rejected),
            ("\"withdrawn\"", ProposalStatus::Withdrawn),
        ];
        for (json, expected) in cases {
            let status: ProposalStatus = serde_json::from_str(json).unwrap();
            assert_eq!(status, expected);
        }
    }

    #[test]
    fn test_proposal_action_deserializes() {
        use crate::contributions::types::ProposalAction;
        let cases = vec![
            ("\"add_source\"", ProposalAction::AddSource),
            ("\"update_source\"", ProposalAction::UpdateSource),
            ("\"remove_source\"", ProposalAction::RemoveSource),
            ("\"add_category\"", ProposalAction::AddCategory),
            ("\"update_category\"", ProposalAction::UpdateCategory),
        ];
        for (json, expected) in cases {
            let action: ProposalAction = serde_json::from_str(json).unwrap();
            assert_eq!(action, expected);
        }
    }

    #[test]
    fn test_vote_choice_deserializes() {
        use crate::contributions::types::VoteChoice;
        let support: VoteChoice = serde_json::from_str("\"support\"").unwrap();
        assert_eq!(support, VoteChoice::Support);
        let oppose: VoteChoice = serde_json::from_str("\"oppose\"").unwrap();
        assert_eq!(oppose, VoteChoice::Oppose);
    }

    #[test]
    fn test_proposal_serde_default_allows_extra_fields() {
        let json = r#"{
            "action": "add_source",
            "status": "pending",
            "category": "test",
            "proposer": "key1",
            "created_at": "2026-03-08T12:00:00Z",
            "data": {},
            "votes": [],
            "future_field": "should not cause error"
        }"#;
        let proposal: Proposal = serde_json::from_str(json).unwrap();
        assert_eq!(proposal.category, "test");
    }
}
