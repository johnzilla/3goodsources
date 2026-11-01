//! History-rewriting signer for audit_log.json. For new entries, use `3gs audit-append`,
//! which never touches existing entries.
//!
//! This tool re-signs and re-chains *every* entry, so anyone who saved an older copy of the
//! log will see different signatures. Only use it to migrate the log (a new key, or a new
//! canonical message format); it refuses to run without `--rewrite-history`.
//!
//! Usage:
//!   PKARR_SECRET_KEY=<64-hex-chars> cargo run --example sign_audit -- --rewrite-history
//!       Re-sign the existing audit_log.json in place. Entry ids, timestamps, actions,
//!       categories and data are kept; actor, signature and previous_hash are rewritten.
//!   PKARR_SECRET_KEY=<64-hex-chars> cargo run --example sign_audit -- --rewrite-history --from-registry
//!       Regenerate the retroactive entries from registry.json (new ids).
//!
//! PKARR_SECRET_KEY is required: the server rejects audit entries not signed by its own key.

use chrono::{TimeZone, Utc};
use ed25519_dalek::SigningKey;
use serde_json::json;
use std::collections::BTreeMap;
use three_good_sources::audit::{hash_entry_json, sign_entry, verify_entries, AuditAction, AuditEntry};
use uuid::Uuid;

fn main() {
    if !std::env::args().any(|a| a == "--rewrite-history") {
        eprintln!(
            "Error: this rewrites every audit signature. To add an entry, use:\n  \
             cargo run -- audit-append --action <action> [--category <slug>] [--data <json>]\n\
             To re-sign the whole log anyway (key or format migration), pass --rewrite-history."
        );
        std::process::exit(1);
    }

    // Load signing key
    let hex_str = std::env::var("PKARR_SECRET_KEY").unwrap_or_else(|_| {
        eprintln!("Error: PKARR_SECRET_KEY must be set to the curator's production key.");
        std::process::exit(1);
    });
    let secret_bytes: [u8; 32] = hex::decode(hex_str.trim())
        .expect("PKARR_SECRET_KEY must be valid hex")
        .try_into()
        .expect("PKARR_SECRET_KEY must be 32 bytes (64 hex chars)");

    let signing_key = SigningKey::from_bytes(&secret_bytes);
    let actor_hex = hex::encode(signing_key.verifying_key().to_bytes());

    let mut entries = if std::env::args().any(|a| a == "--from-registry") {
        entries_from_registry()
    } else {
        let existing = std::fs::read_to_string("audit_log.json")
            .expect("audit_log.json must exist at project root (or pass --from-registry)");
        serde_json::from_str::<Vec<AuditEntry>>(&existing).expect("audit_log.json must be valid")
    };

    // Sign and chain every entry
    let mut previous_hash: Option<String> = None;
    for entry in &mut entries {
        entry.previous_hash = previous_hash.clone();
        sign_entry(entry, &signing_key);
        previous_hash = Some(hash_entry_json(entry));
    }
    verify_entries(&entries).expect("re-signed log must verify");

    // Write to audit_log.json
    let json = serde_json::to_string_pretty(&entries).expect("Failed to serialize entries");
    std::fs::write("audit_log.json", &json).expect("Failed to write audit_log.json");

    println!("Signed audit_log.json with {} entries", entries.len());
    println!("Actor public key: {}", actor_hex);
}

/// Build the unsigned retroactive entries: one category_added per category, then one
/// source_added per source (grouped by category, ordered by rank).
fn entries_from_registry() -> Vec<AuditEntry> {
    let timestamp = Utc.with_ymd_and_hms(2026, 2, 3, 0, 0, 0).unwrap();

    // Load registry
    let registry_json =
        std::fs::read_to_string("registry.json").expect("registry.json must exist at project root");
    let registry: serde_json::Value =
        serde_json::from_str(&registry_json).expect("registry.json must be valid JSON");

    let categories = registry["categories"]
        .as_object()
        .expect("categories must be an object");

    // Sort categories alphabetically by slug
    let sorted_slugs: BTreeMap<&String, &serde_json::Value> = categories.iter().collect();

    let mut entries: Vec<AuditEntry> = Vec::with_capacity(40);

    for (slug, cat) in &sorted_slugs {
        entries.push(AuditEntry {
            id: Uuid::new_v4(),
            timestamp,
            action: AuditAction::CategoryAdded,
            category: Some(slug.to_string()),
            data: json!({
                "name": cat["name"].as_str().unwrap(),
                "description": cat["description"].as_str().unwrap()
            }),
            ..Default::default()
        });
    }

    for (slug, cat) in &sorted_slugs {
        let sources = cat["sources"].as_array().expect("sources must be an array");
        let mut sources_sorted: Vec<&serde_json::Value> = sources.iter().collect();
        sources_sorted.sort_by_key(|s| s["rank"].as_u64().unwrap_or(0));

        for source in sources_sorted {
            entries.push(AuditEntry {
                id: Uuid::new_v4(),
                timestamp,
                action: AuditAction::SourceAdded,
                category: Some(slug.to_string()),
                data: json!({
                    "name": source["name"].as_str().unwrap(),
                    "url": source["url"].as_str().unwrap(),
                    "type": source["type"].as_str().unwrap(),
                    "rank": source["rank"].as_u64().unwrap(),
                    "why": source["why"].as_str().unwrap()
                }),
                ..Default::default()
            });
        }
    }

    entries
}
