use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    /// Path to registry.json file. Required -- no default.
    pub registry_path: PathBuf,

    /// Logging format: "pretty" (default, colored for dev) or "json" (structured for production).
    #[serde(default = "default_log_format")]
    pub log_format: String,

    /// Server port. Defaults to 3000. Set via PORT env var (required by Render).
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_log_format() -> String {
    "pretty".to_string()
}

fn default_port() -> u16 {
    3000
}

impl Config {
    pub fn load() -> Result<Self, anyhow::Error> {
        dotenvy::dotenv().ok(); // Load .env if present, ignore if missing
        envy::from_env::<Config>().map_err(|e| {
            anyhow::anyhow!(
                "Failed to load configuration: {}\n\nRequired: REGISTRY_PATH environment variable must be set",
                e
            )
        })
    }
}
