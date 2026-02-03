mod config;
mod error;
mod matcher;
mod mcp;
mod pubky;
mod registry;
mod server;

use config::Config;
use std::sync::Arc;
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

    // Generate or load PKARR keypair for server identity
    let keypair = crate::pubky::identity::generate_or_load_keypair(
        config.pkarr_secret_key.as_deref()
    )?;
    let public_key = keypair.public_key();
    tracing::info!(
        pubkey = %public_key.to_z32(),
        "Server identity initialized"
    );

    // Load and validate registry
    let registry = Arc::new(registry::load(&config.registry_path).await?);

    // Create MCP handler with shared registry and match config
    let pubkey_z32 = public_key.to_z32();
    let mcp_handler = mcp::McpHandler::new(Arc::clone(&registry), match_config, pubkey_z32);

    // Build application state
    let app_state = Arc::new(server::AppState {
        mcp_handler,
        registry,
        pubkey: public_key,
    });

    // Build router with routes and middleware
    let app = server::build_router(app_state);

    // Bind to configured address and start server
    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!(port = config.port, "Server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
