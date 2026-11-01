//! Append-only writes to the audit log.
//!
//! New entries are signed and linked to the hash of the last entry; existing entries are
//! never rewritten. Like registry signing, this runs offline (`3gs audit-append`); the
//! server only loads and verifies.

use super::error::AuditError;
use super::loader::{verify_actors, verify_entries};
use super::types::{hash_entry_json, sign_entry, AuditAction, AuditEntry};
use chrono::{DateTime, SubsecRound, Utc};
use ed25519_dalek::SigningKey;
use uuid::Uuid;

/// Build, sign, and chain a new entry onto `entries`.
///
/// The existing log is fully verified first (signatures, chain, and that every entry is
/// signed by `signing_key`), so an append never extends a log that wouldn't load.
/// `timestamp` is truncated to whole seconds, the precision the signature covers.
pub fn append_entry(
    entries: &[AuditEntry],
    signing_key: &SigningKey,
    action: AuditAction,
    category: Option<String>,
    data: serde_json::Value,
    timestamp: DateTime<Utc>,
) -> Result<AuditEntry, AuditError> {
    verify_entries(entries)?;
    verify_actors(entries, &hex::encode(signing_key.verifying_key().to_bytes()))?;

    if !data.is_object() {
        return Err(AuditError::InvalidEntry("data must be a JSON object".to_string()));
    }

    let timestamp = timestamp.trunc_subsecs(0);
    let last = entries.last();
    if last.is_some_and(|l| timestamp < l.timestamp) {
        return Err(AuditError::InvalidEntry(format!(
            "timestamp {} is before the last entry ({})",
            timestamp,
            last.unwrap().timestamp
        )));
    }

    let mut entry = AuditEntry {
        id: Uuid::new_v4(),
        timestamp,
        action,
        category,
        data,
        actor: String::new(),
        signature: String::new(),
        previous_hash: last.map(hash_entry_json),
    };
    sign_entry(&mut entry, signing_key);
    Ok(entry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use serde_json::json;

    fn key() -> SigningKey {
        SigningKey::from_bytes(&[11u8; 32])
    }

    fn at(day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, day, 12, 0, 0).unwrap()
    }

    fn log_of(n: u32) -> Vec<AuditEntry> {
        let mut log = Vec::new();
        for i in 0..n {
            let entry = append_entry(
                &log,
                &key(),
                AuditAction::SourceAdded,
                Some("rust-learning".to_string()),
                json!({"n": i}),
                at(1 + i),
            )
            .unwrap();
            log.push(entry);
        }
        log
    }

    #[test]
    fn test_append_to_empty_log_has_no_previous_hash() {
        let log = log_of(1);
        assert!(log[0].previous_hash.is_none());
        assert!(verify_entries(&log).is_ok());
    }

    #[test]
    fn test_append_links_and_keeps_existing_entries() {
        let log = log_of(3);
        let before = log.clone();
        let mut extended = log.clone();
        let entry = append_entry(&log, &key(), AuditAction::CategoryUpdated, None, json!({}), at(10)).unwrap();
        assert_eq!(entry.previous_hash, Some(hash_entry_json(&log[2])));
        extended.push(entry);
        assert!(verify_entries(&extended).is_ok());
        for (a, b) in before.iter().zip(&extended) {
            assert_eq!(hash_entry_json(a), hash_entry_json(b));
        }
    }

    #[test]
    fn test_append_truncates_subseconds() {
        let ts = Utc.timestamp_opt(1_800_000_000, 123_456_789).unwrap();
        let entry = append_entry(&[], &key(), AuditAction::SourceAdded, None, json!({}), ts).unwrap();
        assert_eq!(entry.timestamp.timestamp_subsec_nanos(), 0);
    }

    #[test]
    fn test_append_rejects_backdated_entry() {
        let log = log_of(2);
        let err = append_entry(&log, &key(), AuditAction::SourceAdded, None, json!({}), at(1)).unwrap_err();
        assert!(matches!(err, AuditError::InvalidEntry(_)));
    }

    #[test]
    fn test_append_rejects_other_key() {
        let log = log_of(1);
        let other = SigningKey::from_bytes(&[12u8; 32]);
        let err = append_entry(&log, &other, AuditAction::SourceAdded, None, json!({}), at(5)).unwrap_err();
        assert!(matches!(err, AuditError::ActorMismatch { .. }));
    }

    #[test]
    fn test_append_rejects_non_object_data() {
        let err = append_entry(&[], &key(), AuditAction::SourceAdded, None, json!("x"), at(1)).unwrap_err();
        assert!(matches!(err, AuditError::InvalidEntry(_)));
    }

    #[test]
    fn test_chain_detects_deleted_entry() {
        let mut log = log_of(3);
        log.remove(1);
        assert!(matches!(verify_entries(&log), Err(AuditError::ChainBroken { .. })));
    }

    #[test]
    fn test_relinking_without_key_breaks_signature() {
        // Delete the middle entry and patch previous_hash to hide it
        let mut log = log_of(3);
        log.remove(1);
        log[1].previous_hash = Some(hash_entry_json(&log[0]));
        assert!(matches!(
            verify_entries(&log),
            Err(AuditError::SignatureVerificationFailed { .. })
        ));
    }

    #[test]
    fn test_chain_detects_reorder() {
        let mut log = log_of(3);
        log.swap(1, 2);
        assert!(verify_entries(&log).is_err());
    }

    #[test]
    fn test_append_refuses_broken_log() {
        let mut log = log_of(3);
        log.remove(1);
        assert!(append_entry(&log, &key(), AuditAction::SourceAdded, None, json!({}), at(10)).is_err());
    }
}
