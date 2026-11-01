//! Registry signing and verification.
//!
//! A registry document carries an embedded detached signature:
//!
//! ```json
//! "signature": { "alg": "ed25519", "sig": "<128 hex chars>" }
//! ```
//!
//! The signed message is `3gs-registry-v1\n` followed by the canonical JSON of the
//! whole document with the top-level `signature` key removed. Canonical JSON is
//! compact (no whitespace) with object keys sorted by byte order at every level.
//! Signatures are always checked against the raw JSON document, never a re-serialized
//! struct, so unknown fields and explicit nulls are covered by the signature.
//!
//! The signing key must be the one named in `curator.pubkey` (z-base-32).

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

/// Domain separation prefix for registry signatures (distinct from audit log signatures).
pub const SIGNING_DOMAIN: &str = "3gs-registry-v1\n";

/// The only supported signature algorithm.
pub const SIGNATURE_ALG: &str = "ed25519";

/// Embedded registry signature
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RegistrySignature {
    /// Signature algorithm (always "ed25519")
    pub alg: String,
    /// Hex-encoded 64-byte Ed25519 signature
    pub sig: String,
}

/// Registry signature errors
#[derive(Debug, Error, PartialEq)]
pub enum SignatureError {
    #[error("registry document must be a JSON object")]
    NotAnObject,

    #[error("registry has no signature")]
    Missing,

    #[error("malformed signature field: {0}")]
    Malformed(String),

    #[error("unsupported signature algorithm '{0}' (expected 'ed25519')")]
    UnsupportedAlg(String),

    #[error("registry has no curator.pubkey")]
    MissingCuratorPubkey,

    #[error("curator.pubkey '{actual}' does not match expected key '{expected}'")]
    PubkeyMismatch { expected: String, actual: String },

    #[error("invalid public key '{0}'")]
    InvalidPubkey(String),

    #[error("signature does not verify against curator.pubkey")]
    Invalid,
}

/// Serialize a JSON value canonically: compact, object keys sorted at every level.
///
/// Sorting is done explicitly rather than relying on serde_json's map ordering,
/// so the output is the same whether or not `preserve_order` is enabled.
pub fn canonical_json(value: &Value) -> String {
    let mut out = String::new();
    write_canonical(value, &mut out);
    out
}

fn write_canonical(value: &Value, out: &mut String) {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            out.push('{');
            for (i, key) in keys.into_iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&serde_json::to_string(key).unwrap());
                out.push(':');
                write_canonical(&map[key], out);
            }
            out.push('}');
        }
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_canonical(item, out);
            }
            out.push(']');
        }
        scalar => out.push_str(&serde_json::to_string(scalar).unwrap()),
    }
}

/// Build the exact bytes that are signed: domain prefix + canonical JSON without `signature`.
pub fn signing_message(doc: &Value) -> Result<Vec<u8>, SignatureError> {
    let mut unsigned = doc.as_object().ok_or(SignatureError::NotAnObject)?.clone();
    unsigned.remove("signature");
    let mut message = SIGNING_DOMAIN.as_bytes().to_vec();
    message.extend_from_slice(canonical_json(&Value::Object(unsigned)).as_bytes());
    Ok(message)
}

/// Compute the signature for a registry document (any existing signature is ignored).
pub fn sign_document(doc: &Value, secret_key: &[u8; 32]) -> Result<RegistrySignature, SignatureError> {
    let message = signing_message(doc)?;
    let signature = SigningKey::from_bytes(secret_key).sign(&message);
    Ok(RegistrySignature {
        alg: SIGNATURE_ALG.to_string(),
        sig: hex::encode(signature.to_bytes()),
    })
}

/// Read `curator.pubkey` from a registry document.
pub fn curator_pubkey(doc: &Value) -> Result<&str, SignatureError> {
    doc.get("curator")
        .and_then(|c| c.get("pubkey"))
        .and_then(Value::as_str)
        .ok_or(SignatureError::MissingCuratorPubkey)
}

/// Verify a registry document's signature.
///
/// `curator.pubkey` must equal `expected_pubkey` (z-base-32) and the embedded signature
/// must verify against that key. Pass the key you trust (the server's own identity, or the
/// pubkey you endorsed), never the document's self-declared key.
pub fn verify_document(doc: &Value, expected_pubkey: &str) -> Result<RegistrySignature, SignatureError> {
    let actual = curator_pubkey(doc)?;
    if actual != expected_pubkey {
        return Err(SignatureError::PubkeyMismatch {
            expected: expected_pubkey.to_string(),
            actual: actual.to_string(),
        });
    }

    let signature_value = doc
        .as_object()
        .ok_or(SignatureError::NotAnObject)?
        .get("signature")
        .ok_or(SignatureError::Missing)?;
    let signature: RegistrySignature = serde_json::from_value(signature_value.clone())
        .map_err(|e| SignatureError::Malformed(e.to_string()))?;
    if signature.alg != SIGNATURE_ALG {
        return Err(SignatureError::UnsupportedAlg(signature.alg));
    }

    let sig_bytes: [u8; 64] = hex::decode(&signature.sig)
        .map_err(|e| SignatureError::Malformed(e.to_string()))?
        .try_into()
        .map_err(|_| SignatureError::Malformed("signature must be 64 bytes".to_string()))?;

    let public_key = pkarr::PublicKey::try_from(expected_pubkey)
        .map_err(|_| SignatureError::InvalidPubkey(expected_pubkey.to_string()))?;
    let verifying_key = VerifyingKey::from_bytes(public_key.as_bytes())
        .map_err(|_| SignatureError::InvalidPubkey(expected_pubkey.to_string()))?;

    let message = signing_message(doc)?;
    verifying_key
        .verify(&message, &Signature::from_bytes(&sig_bytes))
        .map_err(|_| SignatureError::Invalid)?;

    Ok(signature)
}

/// Parse raw registry JSON and verify it with [`verify_document`].
pub fn verify_raw(raw: &str, expected_pubkey: &str) -> Result<RegistrySignature, SignatureError> {
    let doc: Value =
        serde_json::from_str(raw).map_err(|e| SignatureError::Malformed(e.to_string()))?;
    verify_document(&doc, expected_pubkey)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const SECRET: [u8; 32] = [7u8; 32];

    fn pubkey_z32() -> String {
        pkarr::Keypair::from_secret_key(&SECRET).public_key().to_z32()
    }

    fn signed_doc() -> Value {
        let mut doc = json!({
            "version": "0.1.0",
            "updated": "2026-11-01",
            "curator": { "name": "Test", "pubkey": pubkey_z32() },
            "endorsements": [],
            "categories": {}
        });
        let sig = sign_document(&doc, &SECRET).unwrap();
        doc["signature"] = serde_json::to_value(sig).unwrap();
        doc
    }

    #[test]
    fn test_canonical_json_sorts_keys_recursively() {
        let value = json!({"b": 1, "a": {"d": [1, {"z": null, "y": "x"}], "c": true}});
        assert_eq!(
            canonical_json(&value),
            r#"{"a":{"c":true,"d":[1,{"y":"x","z":null}]},"b":1}"#
        );
    }

    #[test]
    fn test_signing_message_excludes_signature_and_has_domain() {
        let doc = signed_doc();
        let message = String::from_utf8(signing_message(&doc).unwrap()).unwrap();
        assert!(message.starts_with(SIGNING_DOMAIN));
        assert!(!message.contains("signature"));
    }

    #[test]
    fn test_sign_then_verify() {
        let doc = signed_doc();
        assert!(verify_document(&doc, &pubkey_z32()).is_ok());
    }

    #[test]
    fn test_verify_rejects_tampering() {
        let mut doc = signed_doc();
        doc["updated"] = json!("2027-01-01");
        assert_eq!(verify_document(&doc, &pubkey_z32()), Err(SignatureError::Invalid));
    }

    #[test]
    fn test_verify_rejects_added_field() {
        let mut doc = signed_doc();
        doc["extra"] = json!(null);
        assert_eq!(verify_document(&doc, &pubkey_z32()), Err(SignatureError::Invalid));
    }

    #[test]
    fn test_verify_rejects_unexpected_pubkey() {
        let doc = signed_doc();
        let other = pkarr::Keypair::from_secret_key(&[8u8; 32]).public_key().to_z32();
        assert!(matches!(
            verify_document(&doc, &other),
            Err(SignatureError::PubkeyMismatch { .. })
        ));
    }

    #[test]
    fn test_verify_rejects_signature_from_other_key() {
        let mut doc = signed_doc();
        let forged = sign_document(&doc, &[8u8; 32]).unwrap();
        doc["signature"] = serde_json::to_value(forged).unwrap();
        assert_eq!(verify_document(&doc, &pubkey_z32()), Err(SignatureError::Invalid));
    }

    #[test]
    fn test_verify_rejects_missing_signature() {
        let mut doc = signed_doc();
        doc.as_object_mut().unwrap().remove("signature");
        assert_eq!(verify_document(&doc, &pubkey_z32()), Err(SignatureError::Missing));
    }

    #[test]
    fn test_verify_rejects_unknown_alg() {
        let mut doc = signed_doc();
        doc["signature"]["alg"] = json!("rsa");
        assert_eq!(
            verify_document(&doc, &pubkey_z32()),
            Err(SignatureError::UnsupportedAlg("rsa".to_string()))
        );
    }

    #[test]
    fn test_verify_rejects_malformed_sig() {
        let mut doc = signed_doc();
        doc["signature"]["sig"] = json!("abcd");
        assert!(matches!(
            verify_document(&doc, &pubkey_z32()),
            Err(SignatureError::Malformed(_))
        ));
    }

    #[test]
    fn test_key_order_does_not_affect_signature() {
        let doc = signed_doc();
        let reordered: Value = serde_json::from_str(&format!(
            r#"{{"signature":{},"categories":{{}},"endorsements":[],"curator":{{"pubkey":"{}","name":"Test"}},"updated":"2026-11-01","version":"0.1.0"}}"#,
            doc["signature"],
            pubkey_z32()
        ))
        .unwrap();
        assert!(verify_document(&reordered, &pubkey_z32()).is_ok());
    }
}
