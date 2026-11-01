use super::error::AuditError;
use super::types::{canonical_message, hash_entry_json, AuditEntry};
use std::collections::HashSet;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use std::path::Path;
use tokio::fs;

/// Load and verify the audit log from disk.
/// Verifies Ed25519 signatures for every entry and the hash chain at load time.
pub async fn load(path: impl AsRef<Path>) -> Result<Vec<AuditEntry>, AuditError> {
    let path = path.as_ref();
    let path_str = path.display().to_string();

    let contents = fs::read_to_string(path)
        .await
        .map_err(|e| AuditError::FileRead {
            path: path_str.clone(),
            error: e.to_string(),
        })?;

    let entries: Vec<AuditEntry> =
        serde_json::from_str(&contents).map_err(|e| AuditError::JsonParse {
            path: path_str.clone(),
            error: e.to_string(),
        })?;

    verify_entries(&entries)?;

    tracing::info!(entries = entries.len(), "Audit log loaded successfully");
    Ok(entries)
}

/// Verify every entry's signature and the hash chain linking them.
///
/// The first entry must have no `previous_hash`; every later entry's `previous_hash` must be
/// the hash of the entry before it. Timestamps must not go backwards and ids must be unique.
/// Because signatures cover `previous_hash`, a chain that verifies can't have had entries
/// removed, reordered, or inserted except by the key holder.
pub fn verify_entries(entries: &[AuditEntry]) -> Result<(), AuditError> {
    let mut seen = HashSet::new();
    let mut previous: Option<&AuditEntry> = None;
    for entry in entries {
        verify_signature(entry)?;
        if !seen.insert(entry.id) {
            return Err(AuditError::DuplicateId { id: entry.id });
        }
        let expected = previous.map(hash_entry_json);
        if entry.previous_hash != expected {
            return Err(AuditError::ChainBroken {
                id: entry.id,
                expected,
                actual: entry.previous_hash.clone(),
            });
        }
        if previous.is_some_and(|p| entry.timestamp < p.timestamp) {
            return Err(AuditError::TimestampOutOfOrder { id: entry.id });
        }
        previous = Some(entry);
    }
    Ok(())
}

/// Check that every entry was signed by `expected_actor` (hex-encoded Ed25519 pubkey).
///
/// [`load`] only proves each entry is signed by its own `actor`; call this with the
/// server's own key so a log signed by any other key is rejected.
pub fn verify_actors(entries: &[AuditEntry], expected_actor: &str) -> Result<(), AuditError> {
    match entries.iter().find(|e| e.actor != expected_actor) {
        Some(entry) => Err(AuditError::ActorMismatch {
            id: entry.id,
            expected: expected_actor.to_string(),
            actual: entry.actor.clone(),
        }),
        None => Ok(()),
    }
}

/// Verify an entry's Ed25519 signature against its canonical message.
fn verify_signature(entry: &AuditEntry) -> Result<(), AuditError> {
    let pubkey_bytes =
        hex::decode(&entry.actor).map_err(|_| AuditError::InvalidActorKey { id: entry.id })?;
    let pubkey_array: [u8; 32] = pubkey_bytes
        .try_into()
        .map_err(|_| AuditError::InvalidActorKey { id: entry.id })?;
    let verifying_key = VerifyingKey::from_bytes(&pubkey_array)
        .map_err(|_| AuditError::InvalidActorKey { id: entry.id })?;

    let sig_bytes =
        hex::decode(&entry.signature).map_err(|_| AuditError::InvalidSignature { id: entry.id })?;
    let sig_array: [u8; 64] = sig_bytes
        .try_into()
        .map_err(|_| AuditError::InvalidSignature { id: entry.id })?;
    let signature = Signature::from_bytes(&sig_array);

    let message = canonical_message(entry);
    verifying_key
        .verify(message.as_bytes(), &signature)
        .map_err(|_| AuditError::SignatureVerificationFailed { id: entry.id })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::types::{AuditAction, AuditEntry};
    use chrono::{TimeZone, Utc};
    use ed25519_dalek::{Signer, SigningKey};
    use uuid::Uuid;

    fn make_signed_entry() -> AuditEntry {
        let secret_bytes = [42u8; 32];
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let pubkey_hex = hex::encode(signing_key.verifying_key().to_bytes());

        let mut entry = AuditEntry {
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            timestamp: Utc.with_ymd_and_hms(2026, 2, 3, 0, 0, 0).unwrap(),
            action: AuditAction::SourceAdded,
            category: Some("rust-learning".to_string()),
            data: serde_json::json!({"name": "The Rust Book", "url": "https://doc.rust-lang.org/book/"}),
            actor: pubkey_hex,
            signature: String::new(),
            previous_hash: None,
        };

        let message = canonical_message(&entry);
        let sig = signing_key.sign(message.as_bytes());
        entry.signature = hex::encode(sig.to_bytes());
        entry
    }

    #[test]
    fn test_verify_signature_valid() {
        let entry = make_signed_entry();
        let result = verify_signature(&entry);
        assert!(result.is_ok(), "Valid signature should verify: {:?}", result);
    }

    #[test]
    fn test_verify_signature_tampered_action() {
        let mut entry = make_signed_entry();
        entry.action = AuditAction::SourceRemoved; // tamper
        let result = verify_signature(&entry);
        assert!(result.is_err(), "Tampered action should fail verification");
    }

    #[test]
    fn test_verify_signature_tampered_data() {
        let mut entry = make_signed_entry();
        entry.data = serde_json::json!({"name": "TAMPERED"}); // tamper
        let result = verify_signature(&entry);
        assert!(result.is_err(), "Tampered data should fail verification");
    }

    #[test]
    fn test_loader_rejects_invalid_signature() {
        let mut entry = make_signed_entry();
        entry.signature = hex::encode([0u8; 64]); // invalid sig
        let result = verify_signature(&entry);
        assert!(
            matches!(result, Err(AuditError::SignatureVerificationFailed { .. })),
            "Expected SignatureVerificationFailed, got: {:?}",
            result
        );
    }

    #[test]
    fn test_verify_actors_accepts_matching_key() {
        let entry = make_signed_entry();
        let actor = entry.actor.clone();
        assert!(verify_actors(&[entry], &actor).is_ok());
    }

    #[test]
    fn test_verify_actors_rejects_other_key() {
        let entry = make_signed_entry();
        let other = hex::encode(SigningKey::from_bytes(&[8u8; 32]).verifying_key().to_bytes());
        assert!(matches!(
            verify_actors(&[entry], &other),
            Err(AuditError::ActorMismatch { .. })
        ));
    }

    #[tokio::test]
    async fn test_loader_accepts_valid_json() {
        let entry = make_signed_entry();
        let json = serde_json::to_string_pretty(&vec![&entry]).unwrap();

        let tmp = std::env::temp_dir().join("test_audit_valid.json");
        tokio::fs::write(&tmp, &json).await.unwrap();

        let result = load(&tmp).await;
        assert!(result.is_ok(), "Loader should accept valid audit log: {:?}", result);
        assert_eq!(result.unwrap().len(), 1);

        let _ = tokio::fs::remove_file(&tmp).await;
    }

    #[tokio::test]
    async fn test_loader_rejects_invalid_sig_json() {
        let mut entry = make_signed_entry();
        entry.signature = hex::encode([0u8; 64]);
        let json = serde_json::to_string_pretty(&vec![&entry]).unwrap();

        let tmp = std::env::temp_dir().join("test_audit_invalid.json");
        tokio::fs::write(&tmp, &json).await.unwrap();

        let result = load(&tmp).await;
        assert!(result.is_err(), "Loader should reject invalid signature");

        let _ = tokio::fs::remove_file(&tmp).await;
    }
}
