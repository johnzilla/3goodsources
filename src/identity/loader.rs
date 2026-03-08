use super::error::IdentityError;
use super::types::{Identity, IdentityType};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

/// Load identities from a JSON file on disk.
/// Validates that bot identities have a valid operator_pubkey referencing an existing human identity.
pub async fn load(path: impl AsRef<Path>) -> Result<HashMap<String, Identity>, IdentityError> {
    let path = path.as_ref();
    let path_str = path.display().to_string();

    let contents = fs::read_to_string(path)
        .await
        .map_err(|e| IdentityError::FileRead {
            path: path_str.clone(),
            error: e.to_string(),
        })?;

    let identities: HashMap<String, Identity> =
        serde_json::from_str(&contents).map_err(|e| IdentityError::JsonParse {
            path: path_str.clone(),
            error: e.to_string(),
        })?;

    // Validate bot identities
    for (pubkey, identity) in &identities {
        if identity.identity_type == IdentityType::Bot {
            match &identity.operator_pubkey {
                None => {
                    return Err(IdentityError::MissingOperator {
                        pubkey: pubkey.clone(),
                    });
                }
                Some(operator_key) => {
                    match identities.get(operator_key) {
                        None => {
                            return Err(IdentityError::InvalidOperator {
                                pubkey: pubkey.clone(),
                                operator_pubkey: operator_key.clone(),
                            });
                        }
                        Some(operator) if operator.identity_type != IdentityType::Human => {
                            return Err(IdentityError::InvalidOperator {
                                pubkey: pubkey.clone(),
                                operator_pubkey: operator_key.clone(),
                            });
                        }
                        _ => {} // Valid: operator exists and is human
                    }
                }
            }
        }
    }

    tracing::info!(identities = identities.len(), "Identities loaded successfully");
    Ok(identities)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::types::{IdentityType, Platform, PlatformClaim};

    fn human_identity() -> Identity {
        Identity {
            name: "Alice".to_string(),
            identity_type: IdentityType::Human,
            platforms: vec![PlatformClaim {
                platform: Platform::Github,
                handle: "alice".to_string(),
                proof_url: "https://gist.github.com/alice/proof".to_string(),
            }],
            operator_pubkey: None,
        }
    }

    fn bot_identity(operator: &str) -> Identity {
        Identity {
            name: "TestBot".to_string(),
            identity_type: IdentityType::Bot,
            platforms: vec![],
            operator_pubkey: Some(operator.to_string()),
        }
    }

    #[tokio::test]
    async fn test_load_valid_human_identity() {
        let mut map = HashMap::new();
        map.insert("human_key_1".to_string(), human_identity());
        let json = serde_json::to_string_pretty(&map).unwrap();

        let tmp = std::env::temp_dir().join("test_identity_human.json");
        tokio::fs::write(&tmp, &json).await.unwrap();

        let result = load(&tmp).await;
        assert!(result.is_ok(), "Should load valid human identity: {:?}", result);
        let ids = result.unwrap();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids["human_key_1"].name, "Alice");
        assert_eq!(ids["human_key_1"].identity_type, IdentityType::Human);

        let _ = tokio::fs::remove_file(&tmp).await;
    }

    #[tokio::test]
    async fn test_load_valid_bot_with_human_operator() {
        let mut map = HashMap::new();
        map.insert("human_key_1".to_string(), human_identity());
        map.insert("bot_key_1".to_string(), bot_identity("human_key_1"));
        let json = serde_json::to_string_pretty(&map).unwrap();

        let tmp = std::env::temp_dir().join("test_identity_bot_valid.json");
        tokio::fs::write(&tmp, &json).await.unwrap();

        let result = load(&tmp).await;
        assert!(result.is_ok(), "Should load valid bot with human operator: {:?}", result);
        assert_eq!(result.unwrap().len(), 2);

        let _ = tokio::fs::remove_file(&tmp).await;
    }

    #[tokio::test]
    async fn test_load_bot_missing_operator_pubkey() {
        let mut map = HashMap::new();
        let mut bot = bot_identity("unused");
        bot.operator_pubkey = None;
        map.insert("bot_no_op".to_string(), bot);
        let json = serde_json::to_string_pretty(&map).unwrap();

        let tmp = std::env::temp_dir().join("test_identity_bot_missing_op.json");
        tokio::fs::write(&tmp, &json).await.unwrap();

        let result = load(&tmp).await;
        assert!(result.is_err(), "Should reject bot missing operator_pubkey");
        assert!(
            matches!(result, Err(IdentityError::MissingOperator { .. })),
            "Expected MissingOperator error"
        );

        let _ = tokio::fs::remove_file(&tmp).await;
    }

    #[tokio::test]
    async fn test_load_bot_nonexistent_operator() {
        let mut map = HashMap::new();
        map.insert("bot_key_1".to_string(), bot_identity("nonexistent_key"));
        let json = serde_json::to_string_pretty(&map).unwrap();

        let tmp = std::env::temp_dir().join("test_identity_bot_nonexist.json");
        tokio::fs::write(&tmp, &json).await.unwrap();

        let result = load(&tmp).await;
        assert!(result.is_err(), "Should reject bot with nonexistent operator");
        assert!(
            matches!(result, Err(IdentityError::InvalidOperator { .. })),
            "Expected InvalidOperator error"
        );

        let _ = tokio::fs::remove_file(&tmp).await;
    }

    #[tokio::test]
    async fn test_load_bot_with_bot_operator() {
        let mut map = HashMap::new();
        map.insert("bot_a".to_string(), bot_identity("bot_b"));
        map.insert("bot_b".to_string(), bot_identity("bot_a"));
        let json = serde_json::to_string_pretty(&map).unwrap();

        let tmp = std::env::temp_dir().join("test_identity_bot_bot_op.json");
        tokio::fs::write(&tmp, &json).await.unwrap();

        let result = load(&tmp).await;
        assert!(result.is_err(), "Should reject bot with bot operator");
        assert!(
            matches!(result, Err(IdentityError::InvalidOperator { .. })),
            "Expected InvalidOperator error"
        );

        let _ = tokio::fs::remove_file(&tmp).await;
    }

    #[tokio::test]
    async fn test_load_empty_json_object() {
        let tmp = std::env::temp_dir().join("test_identity_empty.json");
        tokio::fs::write(&tmp, "{}").await.unwrap();

        let result = load(&tmp).await;
        assert!(result.is_ok(), "Should load empty JSON object: {:?}", result);
        assert_eq!(result.unwrap().len(), 0);

        let _ = tokio::fs::remove_file(&tmp).await;
    }

    #[tokio::test]
    async fn test_load_invalid_json() {
        let tmp = std::env::temp_dir().join("test_identity_invalid.json");
        tokio::fs::write(&tmp, "not valid json!!!").await.unwrap();

        let result = load(&tmp).await;
        assert!(result.is_err(), "Should reject invalid JSON");
        assert!(
            matches!(result, Err(IdentityError::JsonParse { .. })),
            "Expected JsonParse error"
        );

        let _ = tokio::fs::remove_file(&tmp).await;
    }

    #[tokio::test]
    async fn test_load_file_not_found() {
        let result = load("/tmp/nonexistent_identity_file_12345.json").await;
        assert!(result.is_err(), "Should error on missing file");
        assert!(
            matches!(result, Err(IdentityError::FileRead { .. })),
            "Expected FileRead error"
        );
    }

    #[test]
    fn test_identity_deserializes_from_json() {
        let json = r#"{
            "name": "John Turner",
            "type": "human",
            "platforms": [
                {
                    "platform": "x",
                    "handle": "john",
                    "proof_url": "https://x.com/john/status/123"
                }
            ]
        }"#;
        let identity: Identity = serde_json::from_str(json).unwrap();
        assert_eq!(identity.name, "John Turner");
        assert_eq!(identity.identity_type, IdentityType::Human);
        assert_eq!(identity.platforms.len(), 1);
        assert_eq!(identity.platforms[0].platform, Platform::X);
        assert!(identity.operator_pubkey.is_none());
    }

    #[test]
    fn test_identity_serde_default_allows_extra_fields() {
        let json = r#"{
            "name": "Test",
            "type": "human",
            "platforms": [],
            "future_field": "should not cause error"
        }"#;
        let identity: Identity = serde_json::from_str(json).unwrap();
        assert_eq!(identity.name, "Test");
    }

    #[test]
    fn test_bot_serializes_without_none_operator() {
        let identity = Identity {
            name: "Human".to_string(),
            identity_type: IdentityType::Human,
            platforms: vec![],
            operator_pubkey: None,
        };
        let json = serde_json::to_string(&identity).unwrap();
        assert!(!json.contains("operator_pubkey"), "None operator_pubkey should be skipped in serialization");
    }

    #[test]
    fn test_bot_serializes_with_operator() {
        let identity = Identity {
            name: "Bot".to_string(),
            identity_type: IdentityType::Bot,
            platforms: vec![],
            operator_pubkey: Some("human_key".to_string()),
        };
        let json = serde_json::to_string(&identity).unwrap();
        assert!(json.contains("operator_pubkey"), "Some operator_pubkey should be included");
        assert!(json.contains("human_key"));
    }
}
