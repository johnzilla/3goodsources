//! sign-registry CLI subcommand — signs registry.json offline with the curator's PKARR key.
//!
//! Like `fork`, this runs before server config is loaded. The secret key is read from
//! PKARR_SECRET_KEY (environment or .env) and never leaves this process.
//!
//! Usage:
//!   3gs sign-registry [--registry <path>] [--check]

use crate::registry::signing::{self, RegistrySignature};
use crate::registry::Registry;

const USAGE: &str = "\
Usage: 3gs sign-registry [OPTIONS]

Signs registry.json with the Ed25519 key in PKARR_SECRET_KEY (read from the
environment or .env). curator.pubkey must match that key.

Optional:
  --registry <path>    Registry file to sign (default: $REGISTRY_PATH, else registry.json)
  --check              Only verify the existing signature against curator.pubkey; no key needed

Examples:
  3gs sign-registry
  3gs sign-registry --registry ./my-node/registry.json
  3gs sign-registry --check
";

/// Entry point for the sign-registry subcommand.
///
/// `args` is the full `std::env::args()` collection (index 0 = binary, index 1 = "sign-registry").
pub fn run(args: Vec<String>) -> Result<(), String> {
    dotenvy::dotenv().ok();

    let mut path: Option<String> = None;
    let mut check_only = false;

    let mut i = 2usize;
    while i < args.len() {
        match args[i].as_str() {
            "--registry" => {
                i += 1;
                if i >= args.len() {
                    return Err(format!("--registry requires a value\n\n{}", USAGE));
                }
                path = Some(args[i].clone());
            }
            "--check" => check_only = true,
            "--help" | "-h" => {
                println!("{}", USAGE);
                return Ok(());
            }
            unknown => {
                return Err(format!("Unknown flag: {}\n\n{}", unknown, USAGE));
            }
        }
        i += 1;
    }

    let path = path
        .or_else(|| std::env::var("REGISTRY_PATH").ok())
        .unwrap_or_else(|| "registry.json".to_string());

    let contents = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read '{}': {}", path, e))?;

    if check_only {
        let doc: serde_json::Value = serde_json::from_str(&contents)
            .map_err(|e| format!("'{}' is not valid JSON: {}", path, e))?;
        let pubkey = signing::curator_pubkey(&doc).map_err(|e| e.to_string())?;
        let signature = signing::verify_raw(&contents, pubkey)
            .map_err(|e| format!("'{}' signature check failed: {}", path, e))?;
        println!("OK: {} is signed by {} (sig {}…)", path, pubkey, &signature.sig[..16]);
        return Ok(());
    }

    let secret_hex = std::env::var("PKARR_SECRET_KEY")
        .map_err(|_| "PKARR_SECRET_KEY is not set (environment or .env)".to_string())?;
    let keypair = crate::pubky::identity::generate_or_load_keypair(Some(&secret_hex))
        .map_err(|e| e.to_string())?;

    let signed = sign_contents(&contents, &keypair.secret_key())?;
    std::fs::write(&path, &signed).map_err(|e| format!("Failed to write '{}': {}", path, e))?;

    println!("Signed {} as {}", path, keypair.public_key().to_z32());
    Ok(())
}

/// Sign registry JSON text and return the new file contents.
///
/// Keeps the file's existing layout: the signature is written as the last top-level
/// field, replacing a signature previously written by this tool. If the layout can't
/// be preserved, falls back to a pretty-printed rewrite. The result is always
/// re-verified before it is returned.
pub fn sign_contents(contents: &str, secret_key: &[u8; 32]) -> Result<String, String> {
    let doc: serde_json::Value =
        serde_json::from_str(contents).map_err(|e| format!("Registry is not valid JSON: {}", e))?;
    serde_json::from_value::<Registry>(doc.clone())
        .map_err(|e| format!("Registry does not match the schema: {}", e))?;

    let pubkey = pkarr::Keypair::from_secret_key(secret_key).public_key().to_z32();
    let curator = signing::curator_pubkey(&doc).map_err(|e| e.to_string())?;
    if curator != pubkey {
        return Err(format!(
            "curator.pubkey is '{}' but PKARR_SECRET_KEY belongs to '{}'. \
             Set curator.pubkey to '{}' and re-run.",
            curator, pubkey, pubkey
        ));
    }

    let signature = signing::sign_document(&doc, secret_key).map_err(|e| e.to_string())?;

    let preserved = splice_signature(contents, &doc, &signature);
    if let Some(text) = preserved.filter(|t| signing::verify_raw(t, &pubkey).is_ok()) {
        return Ok(text);
    }

    let mut doc = doc;
    doc["signature"] = serde_json::to_value(&signature).map_err(|e| e.to_string())?;
    let text = serde_json::to_string_pretty(&doc).map_err(|e| e.to_string())? + "\n";
    signing::verify_raw(&text, &pubkey).map_err(|e| format!("Signed output failed verification: {}", e))?;
    Ok(text)
}

/// Format the signature as the line this tool writes into registry.json.
fn signature_line(signature: &RegistrySignature) -> String {
    format!(
        "  \"signature\": {{ \"alg\": \"{}\", \"sig\": \"{}\" }}",
        signature.alg, signature.sig
    )
}

/// Insert (or replace) the signature as the final top-level field without reformatting.
fn splice_signature(
    contents: &str,
    doc: &serde_json::Value,
    signature: &RegistrySignature,
) -> Option<String> {
    let mut body = contents.trim_end();

    match doc.get("signature") {
        Some(existing) => {
            let existing: RegistrySignature = serde_json::from_value(existing.clone()).ok()?;
            let suffix = format!(",\n{}\n}}", signature_line(&existing));
            body = body.strip_suffix(&suffix)?;
        }
        None => {
            body = body.strip_suffix('}')?.trim_end();
        }
    }

    Some(format!("{},\n{}\n}}\n", body, signature_line(signature)))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: [u8; 32] = [9u8; 32];

    fn registry_text() -> String {
        let pubkey = pkarr::Keypair::from_secret_key(&SECRET).public_key().to_z32();
        format!(
            "{{\n  \"version\": \"0.1.0\",\n  \"updated\": \"2026-11-01\",\n  \"curator\": {{\n    \"name\": \"T\",\n    \"pubkey\": \"{}\"\n  }},\n  \"endorsements\": [],\n  \"categories\": {{}}\n}}\n",
            pubkey
        )
    }

    #[test]
    fn test_sign_preserves_layout_and_verifies() {
        let original = registry_text();
        let signed = sign_contents(&original, &SECRET).unwrap();
        assert!(signed.starts_with(original.trim_end().strip_suffix('}').unwrap().trim_end()));
        assert!(signed.trim_end().ends_with("\" }\n}"));
        let pubkey = pkarr::Keypair::from_secret_key(&SECRET).public_key().to_z32();
        assert!(signing::verify_raw(&signed, &pubkey).is_ok());
    }

    #[test]
    fn test_resign_is_idempotent() {
        let once = sign_contents(&registry_text(), &SECRET).unwrap();
        let twice = sign_contents(&once, &SECRET).unwrap();
        assert_eq!(once, twice);
    }

    #[test]
    fn test_resign_after_edit_replaces_signature() {
        let once = sign_contents(&registry_text(), &SECRET).unwrap();
        let edited = once.replace("2026-11-01", "2026-11-02");
        let twice = sign_contents(&edited, &SECRET).unwrap();
        assert_eq!(twice.matches("\"signature\"").count(), 1);
        assert_ne!(once, twice);
    }

    #[test]
    fn test_refuses_mismatched_curator_pubkey() {
        let err = sign_contents(&registry_text(), &[1u8; 32]).unwrap_err();
        assert!(err.contains("curator.pubkey"));
    }

    #[test]
    fn test_refuses_schema_violation() {
        let text = registry_text().replace("\"categories\"", "\"bogus\": 1,\n  \"categories\"");
        let err = sign_contents(&text, &SECRET).unwrap_err();
        assert!(err.contains("schema"));
    }

    #[test]
    fn test_unknown_flag() {
        let args = vec!["3gs".into(), "sign-registry".into(), "--nope".into()];
        assert!(run(args).unwrap_err().contains("Unknown flag"));
    }
}
