//! Signing utility to generate the audit_log.json file with retroactive entries.
//!
//! Usage: PKARR_SECRET_KEY=<64-hex-chars> cargo run --example sign_audit
//!
//! If PKARR_SECRET_KEY is not set, generates a deterministic test key and prints a warning.

use chrono::{TimeZone, Utc};
use ed25519_dalek::{Signer, SigningKey};
use serde_json::json;
use std::collections::BTreeMap;
use three_good_sources::audit::{canonical_message, hash_entry_json, AuditAction, AuditEntry};
use uuid::Uuid;

fn main() {
    // Load signing key
    let secret_bytes: [u8; 32] = match std::env::var("PKARR_SECRET_KEY") {
        Ok(hex_str) => {
            let bytes = hex::decode(&hex_str).expect("PKARR_SECRET_KEY must be valid hex");
            bytes
                .try_into()
                .expect("PKARR_SECRET_KEY must be 32 bytes (64 hex chars)")
        }
        Err(_) => {
            eprintln!("WARNING: PKARR_SECRET_KEY not set. Using deterministic test key.");
            eprintln!("Set PKARR_SECRET_KEY for production audit log generation.");
            [42u8; 32]
        }
    };

    let signing_key = SigningKey::from_bytes(&secret_bytes);
    let actor_hex = hex::encode(signing_key.verifying_key().to_bytes());
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
    let mut previous_hash: Option<String> = None;

    // Generate 10 category_added entries
    for (slug, cat) in &sorted_slugs {
        let mut entry = AuditEntry {
            id: Uuid::new_v4(),
            timestamp,
            action: AuditAction::CategoryAdded,
            category: Some(slug.to_string()),
            data: json!({
                "name": cat["name"].as_str().unwrap(),
                "description": cat["description"].as_str().unwrap()
            }),
            actor: actor_hex.clone(),
            signature: String::new(),
            previous_hash: previous_hash.clone(),
        };

        // Sign
        let message = canonical_message(&entry);
        let sig = signing_key.sign(message.as_bytes());
        entry.signature = hex::encode(sig.to_bytes());

        previous_hash = Some(hash_entry_json(&entry));
        entries.push(entry);
    }

    // Generate 30 source_added entries (grouped by category, ordered by rank)
    for (slug, cat) in &sorted_slugs {
        let sources = cat["sources"].as_array().expect("sources must be an array");
        let mut sources_sorted: Vec<&serde_json::Value> = sources.iter().collect();
        sources_sorted.sort_by_key(|s| s["rank"].as_u64().unwrap_or(0));

        for source in sources_sorted {
            let mut entry = AuditEntry {
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
                actor: actor_hex.clone(),
                signature: String::new(),
                previous_hash: previous_hash.clone(),
            };

            let message = canonical_message(&entry);
            let sig = signing_key.sign(message.as_bytes());
            entry.signature = hex::encode(sig.to_bytes());

            previous_hash = Some(hash_entry_json(&entry));
            entries.push(entry);
        }
    }

    // Write to audit_log.json
    let json = serde_json::to_string_pretty(&entries).expect("Failed to serialize entries");
    std::fs::write("audit_log.json", &json).expect("Failed to write audit_log.json");

    println!("Generated audit_log.json with {} entries", entries.len());
    println!("Actor public key: {}", actor_hex);
}
