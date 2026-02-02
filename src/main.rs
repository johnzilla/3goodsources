mod config;
mod error;
mod matcher;
mod mcp;
mod pubky;
mod registry;
mod server;

use config::Config;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn init_logging(log_format: &str) {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    match log_format {
        "json" => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt::layer().json())
                .init();
        }
        _ => {
            // Default to pretty format (colored, human-readable)
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt::layer().pretty())
                .init();
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration from environment
    let config = Config::load()?;

    // Initialize logging based on configured format
    init_logging(&config.log_format);

    tracing::info!("Starting 3GS server");

    // Load and validate match configuration
    let match_config = matcher::MatchConfig::load()?;
    match_config.validate()?;
    tracing::info!(
        threshold = match_config.match_threshold,
        fuzzy_weight = match_config.match_fuzzy_weight,
        keyword_weight = match_config.match_keyword_weight,
        "Match configuration loaded"
    );

    // Load and validate registry
    let _registry = registry::load(&config.registry_path).await?;

    Ok(())
}
