/// Fork CLI subcommand — scaffolds a ready-to-run 3GS node.
///
/// Intentionally decoupled from all server logic (no crate:: imports).
/// Depends only on external crates: pkarr, hex, serde_json, chrono.
///
/// Usage:
///   3gs fork --endorse <pubkey> --url <peer_url> [--name <curator_name>] [--output <dir>]

const USAGE: &str = "\
Usage: 3gs fork --endorse <pubkey> --url <peer_url> [OPTIONS]

Required:
  --endorse <pubkey>   PKARR public key (z-base-32) of the peer to endorse
  --url <peer_url>     URL of the peer's 3GS instance (e.g. https://peer.example.com)

Optional:
  --name <name>        Your curator display name (default: \"New Curator\")
  --output <dir>       Output directory path (default: ./3gs-fork-<first8chars>)

Examples:
  3gs fork --endorse abc123xyz --url https://3gs.ai
  3gs fork --endorse abc123xyz --url https://3gs.ai --name \"My Node\" --output ./my-3gs-node
";

/// Entry point for the fork subcommand.
///
/// `args` is the full `std::env::args()` collection (index 0 = binary, index 1 = "fork").
pub fn run(args: Vec<String>) -> Result<(), String> {
    // Parse flags from args[2..]
    let mut endorse_pubkey: Option<String> = None;
    let mut peer_url: Option<String> = None;
    let mut curator_name: Option<String> = None;
    let mut output_dir: Option<String> = None;

    let mut i = 2usize;
    while i < args.len() {
        match args[i].as_str() {
            "--endorse" => {
                i += 1;
                if i >= args.len() {
                    return Err(format!("--endorse requires a value\n\n{}", USAGE));
                }
                endorse_pubkey = Some(args[i].clone());
            }
            "--url" => {
                i += 1;
                if i >= args.len() {
                    return Err(format!("--url requires a value\n\n{}", USAGE));
                }
                peer_url = Some(args[i].clone());
            }
            "--name" => {
                i += 1;
                if i >= args.len() {
                    return Err(format!("--name requires a value\n\n{}", USAGE));
                }
                curator_name = Some(args[i].clone());
            }
            "--output" => {
                i += 1;
                if i >= args.len() {
                    return Err(format!("--output requires a value\n\n{}", USAGE));
                }
                output_dir = Some(args[i].clone());
            }
            unknown => {
                return Err(format!("Unknown flag: {}\n\n{}", unknown, USAGE));
            }
        }
        i += 1;
    }

    // Validate required flags
    let endorse_pubkey = endorse_pubkey
        .ok_or_else(|| format!("--endorse is required\n\n{}", USAGE))?;
    let peer_url = peer_url
        .ok_or_else(|| format!("--url is required\n\n{}", USAGE))?;
    let curator_name = curator_name.unwrap_or_else(|| "New Curator".to_string());

    // Generate a fresh keypair for this node
    let keypair = pkarr::Keypair::random();
    let secret_hex = hex::encode(keypair.secret_key());
    let pubkey_z32 = keypair.public_key().to_z32();

    // Determine output directory
    let dir = output_dir.unwrap_or_else(|| format!("3gs-fork-{}", &pubkey_z32[..8]));

    // Check that output directory does not already exist
    if std::path::Path::new(&dir).exists() {
        return Err(format!(
            "Output directory '{}' already exists. Remove it or choose a different --output path.",
            dir
        ));
    }

    // Create output directory
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create directory '{}': {}", dir, e))?;

    // Today's date for registry "updated" and endorsement "since" fields
    let today_iso = chrono::Local::now().format("%Y-%m-%d").to_string();

    // 1. Write registry.json
    let registry = serde_json::json!({
        "version": "0.1.0",
        "updated": today_iso,
        "curator": {
            "name": curator_name,
            "pubkey": pubkey_z32
        },
        "endorsements": [{
            "pubkey": endorse_pubkey,
            "url": peer_url,
            "name": null,
            "since": today_iso
        }],
        "categories": {}
    });
    let registry_json = serde_json::to_string_pretty(&registry)
        .map_err(|e| format!("Failed to serialize registry.json: {}", e))?;
    std::fs::write(format!("{}/registry.json", dir), registry_json)
        .map_err(|e| format!("Failed to write '{}/registry.json': {}", dir, e))?;

    // 2. Write identities.json — empty flat object
    std::fs::write(format!("{}/identities.json", dir), "{}")
        .map_err(|e| format!("Failed to write '{}/identities.json': {}", dir, e))?;

    // 3. Write contributions.json — empty flat HashMap (deserializes as HashMap<Uuid, Proposal>)
    std::fs::write(format!("{}/contributions.json", dir), "{}")
        .map_err(|e| format!("Failed to write '{}/contributions.json': {}", dir, e))?;

    // 4. Write audit_log.json — empty array
    std::fs::write(format!("{}/audit_log.json", dir), "[]")
        .map_err(|e| format!("Failed to write '{}/audit_log.json': {}", dir, e))?;

    // 5. Write .env with all required config vars
    let env_content = format!(
        "REGISTRY_PATH=registry.json\n\
         AUDIT_LOG_PATH=audit_log.json\n\
         IDENTITIES_PATH=identities.json\n\
         CONTRIBUTIONS_PATH=contributions.json\n\
         PKARR_SECRET_KEY={}\n\
         PORT=3000\n",
        secret_hex
    );
    std::fs::write(format!("{}/.env", dir), env_content)
        .map_err(|e| format!("Failed to write '{}/.env': {}", dir, e))?;

    // Print success output
    println!(
        "3GS Node Scaffolded Successfully!\n\
         \n\
         \x20 Directory:  {dir}\n\
         \x20 Public Key: {pubkey_z32}\n\
         \n\
         \x20 WARNING: Your secret key is in {dir}/.env -- keep it safe!\n\
         \n\
         \x20 To run your node:\n\
         \x20   cd {dir}\n\
         \x20   # Copy the binary or run from repo root:\n\
         \x20   cargo run\n\
         \n\
         \x20 To add categories, edit registry.json and add entries to the \"categories\" object.\n"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_args(extra: &[&str]) -> Vec<String> {
        let mut args = vec!["3gs".to_string(), "fork".to_string()];
        args.extend(extra.iter().map(|s| s.to_string()));
        args
    }

    #[test]
    fn test_missing_endorse_flag() {
        let args = make_args(&["--url", "http://localhost:3001"]);
        let result = run(args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("--endorse"));
    }

    #[test]
    fn test_missing_url_flag() {
        let args = make_args(&["--endorse", "somepubkey"]);
        let result = run(args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("--url"));
    }

    #[test]
    fn test_unknown_flag() {
        let args = make_args(&["--endorse", "pk", "--url", "http://x.com", "--bogus"]);
        let result = run(args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown flag"));
    }

    #[test]
    fn test_scaffolds_directory() {
        let tmp = format!("/tmp/3gs-fork-test-{}", std::process::id());
        let args = make_args(&[
            "--endorse", "testpubkey123",
            "--url", "http://localhost:3001",
            "--name", "Test Curator",
            "--output", &tmp,
        ]);
        let result = run(args);
        assert!(result.is_ok(), "fork::run failed: {:?}", result.err());

        // All 5 files must exist
        assert!(std::path::Path::new(&format!("{}/registry.json", tmp)).exists());
        assert!(std::path::Path::new(&format!("{}/identities.json", tmp)).exists());
        assert!(std::path::Path::new(&format!("{}/contributions.json", tmp)).exists());
        assert!(std::path::Path::new(&format!("{}/audit_log.json", tmp)).exists());
        assert!(std::path::Path::new(&format!("{}/.env", tmp)).exists());

        // registry.json must be valid JSON with required fields
        let reg_content = std::fs::read_to_string(format!("{}/registry.json", tmp)).unwrap();
        let reg: serde_json::Value = serde_json::from_str(&reg_content).unwrap();
        assert_eq!(reg["endorsements"][0]["pubkey"], "testpubkey123");
        assert_eq!(reg["endorsements"][0]["url"], "http://localhost:3001");
        assert_eq!(reg["curator"]["name"], "Test Curator");

        // identities.json and contributions.json must be empty objects
        assert_eq!(std::fs::read_to_string(format!("{}/identities.json", tmp)).unwrap(), "{}");
        assert_eq!(std::fs::read_to_string(format!("{}/contributions.json", tmp)).unwrap(), "{}");

        // audit_log.json must be empty array
        assert_eq!(std::fs::read_to_string(format!("{}/audit_log.json", tmp)).unwrap(), "[]");

        // .env must contain required vars
        let env_content = std::fs::read_to_string(format!("{}/.env", tmp)).unwrap();
        assert!(env_content.contains("REGISTRY_PATH=registry.json"));
        assert!(env_content.contains("AUDIT_LOG_PATH=audit_log.json"));
        assert!(env_content.contains("IDENTITIES_PATH=identities.json"));
        assert!(env_content.contains("CONTRIBUTIONS_PATH=contributions.json"));
        assert!(env_content.contains("PKARR_SECRET_KEY="));
        assert!(env_content.contains("PORT=3000"));

        // Cleanup
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn test_existing_directory_errors() {
        let tmp = format!("/tmp/3gs-fork-exists-{}", std::process::id());
        std::fs::create_dir_all(&tmp).unwrap();

        let args = make_args(&[
            "--endorse", "pk",
            "--url", "http://x.com",
            "--output", &tmp,
        ]);
        let result = run(args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));

        std::fs::remove_dir_all(&tmp).ok();
    }
}
