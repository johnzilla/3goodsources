//! audit-append CLI subcommand — appends one signed, hash-chained entry to audit_log.json.
//!
//! Like `sign-registry`, this runs before server config is loaded and reads the key from
//! PKARR_SECRET_KEY (environment or .env). Existing entries are never rewritten: the new
//! entry is spliced in before the closing `]`, so earlier bytes of the file stay identical.
//!
//! Usage:
//!   3gs audit-append --action <action> [--category <slug>] [--data <json>] [--audit-log <path>]

use crate::audit::{append_entry, verify_actors, verify_entries, AuditAction, AuditEntry};
use ed25519_dalek::SigningKey;

const USAGE: &str = "\
Usage: 3gs audit-append --action <action> [OPTIONS]

Appends one entry to the audit log, signed with PKARR_SECRET_KEY (environment or .env)
and linked to the previous entry's hash. Existing entries are never changed.

Required:
  --action <action>     source_added, source_updated, source_removed, category_added,
                        category_updated, category_removed, identity_registered,
                        identity_updated, proposal_submitted, proposal_status_changed, vote_cast

Optional:
  --category <slug>     Category the change applies to
  --data <json>         JSON object describing the change (default: {})
  --audit-log <path>    Audit log file (default: $AUDIT_LOG_PATH, else audit_log.json)

Example:
  3gs audit-append --action source_updated --category rust-learning \\
    --data '{\"rank\": 2, \"name\": \"Rust by Example\", \"url\": \"https://doc.rust-lang.org/rust-by-example/\"}'
";

/// Entry point for the audit-append subcommand.
///
/// `args` is the full `std::env::args()` collection (index 0 = binary, index 1 = "audit-append").
pub fn run(args: Vec<String>) -> Result<(), String> {
    let mut action: Option<String> = None;
    let mut category: Option<String> = None;
    let mut data: Option<String> = None;
    let mut path: Option<String> = None;

    let mut i = 2usize;
    while i < args.len() {
        let flag = args[i].as_str();
        if flag == "--help" || flag == "-h" {
            println!("{}", USAGE);
            return Ok(());
        }
        let slot = match flag {
            "--action" => &mut action,
            "--category" => &mut category,
            "--data" => &mut data,
            "--audit-log" => &mut path,
            unknown => return Err(format!("Unknown flag: {}\n\n{}", unknown, USAGE)),
        };
        i += 1;
        if i >= args.len() {
            return Err(format!("{} requires a value\n\n{}", flag, USAGE));
        }
        *slot = Some(args[i].clone());
        i += 1;
    }

    let action = action.ok_or_else(|| format!("--action is required\n\n{}", USAGE))?;
    let action: AuditAction = serde_json::from_value(serde_json::Value::String(action.clone()))
        .map_err(|_| format!("Unknown action '{}'\n\n{}", action, USAGE))?;
    let data: serde_json::Value = serde_json::from_str(data.as_deref().unwrap_or("{}"))
        .map_err(|e| format!("--data is not valid JSON: {}", e))?;

    // Load .env only once arguments are valid
    dotenvy::dotenv().ok();
    let path = path
        .or_else(|| std::env::var("AUDIT_LOG_PATH").ok())
        .unwrap_or_else(|| "audit_log.json".to_string());

    let secret_hex = std::env::var("PKARR_SECRET_KEY")
        .map_err(|_| "PKARR_SECRET_KEY is not set (environment or .env)".to_string())?;
    let keypair = crate::pubky::identity::generate_or_load_keypair(Some(secret_hex.trim()))
        .map_err(|e| e.to_string())?;
    let signing_key = SigningKey::from_bytes(&keypair.secret_key());

    let contents = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read '{}': {}", path, e))?;
    let (updated, entry) = append_to_contents(&contents, &signing_key, action, category, data)?;
    std::fs::write(&path, updated).map_err(|e| format!("Failed to write '{}': {}", path, e))?;

    println!("Appended entry {} to {} ({:?})", entry.id, path, entry.action);
    Ok(())
}

/// Append a new signed entry to audit log JSON text, returning the new text and entry.
///
/// The existing text is kept byte-for-byte; the result is re-verified before it is returned.
pub fn append_to_contents(
    contents: &str,
    signing_key: &SigningKey,
    action: AuditAction,
    category: Option<String>,
    data: serde_json::Value,
) -> Result<(String, AuditEntry), String> {
    let entries: Vec<AuditEntry> = serde_json::from_str(contents)
        .map_err(|e| format!("Audit log is not valid JSON: {}", e))?;

    let entry = append_entry(&entries, signing_key, action, category, data, chrono::Utc::now())
        .map_err(|e| format!("Refusing to append: {}", e))?;

    let entry_json = serde_json::to_string_pretty(&entry)
        .map_err(|e| e.to_string())?
        .lines()
        .map(|line| format!("  {}", line))
        .collect::<Vec<_>>()
        .join("\n");

    let body = contents
        .trim_end()
        .strip_suffix(']')
        .ok_or("Audit log must be a JSON array")?
        .trim_end();
    let updated = if entries.is_empty() {
        format!("[\n{}\n]\n", entry_json)
    } else {
        format!("{},\n{}\n]\n", body, entry_json)
    };

    // Re-verify what will be written
    let reparsed: Vec<AuditEntry> = serde_json::from_str(&updated).map_err(|e| e.to_string())?;
    verify_entries(&reparsed).map_err(|e| format!("Appended log failed verification: {}", e))?;
    verify_actors(&reparsed, &hex::encode(signing_key.verifying_key().to_bytes()))
        .map_err(|e| format!("Appended log failed verification: {}", e))?;

    Ok((updated, entry))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn key() -> SigningKey {
        SigningKey::from_bytes(&[13u8; 32])
    }

    #[test]
    fn test_append_to_empty_file() {
        let (text, entry) = append_to_contents("[]", &key(), AuditAction::CategoryAdded, None, json!({})).unwrap();
        let entries: Vec<AuditEntry> = serde_json::from_str(&text).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, entry.id);
    }

    #[test]
    fn test_append_keeps_existing_bytes() {
        let (first, _) = append_to_contents("[]", &key(), AuditAction::CategoryAdded, None, json!({})).unwrap();
        let (second, _) = append_to_contents(&first, &key(), AuditAction::SourceAdded, None, json!({"a": 1})).unwrap();
        let prefix = first.trim_end().strip_suffix(']').unwrap().trim_end();
        assert!(second.starts_with(prefix));
        assert_eq!(serde_json::from_str::<Vec<AuditEntry>>(&second).unwrap().len(), 2);
    }

    #[test]
    fn test_append_rejects_other_key() {
        let (first, _) = append_to_contents("[]", &key(), AuditAction::CategoryAdded, None, json!({})).unwrap();
        let other = SigningKey::from_bytes(&[14u8; 32]);
        let err = append_to_contents(&first, &other, AuditAction::SourceAdded, None, json!({})).unwrap_err();
        assert!(err.contains("Refusing to append"));
    }

    #[test]
    fn test_unknown_action() {
        let args: Vec<String> = ["3gs", "audit-append", "--action", "nope"].iter().map(|s| s.to_string()).collect();
        assert!(run(args).unwrap_err().contains("Unknown action"));
    }

    #[test]
    fn test_missing_value() {
        let args: Vec<String> = ["3gs", "audit-append", "--action"].iter().map(|s| s.to_string()).collect();
        assert!(run(args).unwrap_err().contains("requires a value"));
    }
}
