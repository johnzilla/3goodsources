
use three_good_sources::{
    audit, config::Config, contributions, federation, fork, identity, matcher, mcp, pubky, registry,
    server, sign,
};
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
    // Check for CLI subcommands before loading server config
    {
        let args: Vec<String> = std::env::args().collect();
        let result = match args.get(1).map(String::as_str) {
            Some("fork") => Some(fork::run(args)),
            Some("sign-registry") => Some(sign::run(args)),
            _ => None,
        };
        match result {
            Some(Ok(())) => std::process::exit(0),
            Some(Err(e)) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
            None => {}
        }
    }

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
    let keypair = pubky::identity::generate_or_load_keypair(
        config.pkarr_secret_key.as_deref()
    )?;
    let public_key = keypair.public_key();
    tracing::info!(
        pubkey = %public_key.to_z32(),
        "Server identity initialized"
    );

    // Load and validate registry, then verify it is signed by this server's key
    let (registry, registry_raw) = registry::load_with_raw(&config.registry_path).await?;
    match registry::signing::verify_raw(&registry_raw, &public_key.to_z32()) {
        Ok(signature) => tracing::info!(sig = %signature.sig, "Registry signature verified"),
        Err(err) if config.allow_unsigned_registry => tracing::warn!(
            error = %err,
            "Registry signature check failed; continuing because ALLOW_UNSIGNED_REGISTRY=true"
        ),
        Err(err) => anyhow::bail!(
            "Registry signature check failed: {}\n\nSign it with `3gs sign-registry` (uses PKARR_SECRET_KEY), \
             or set ALLOW_UNSIGNED_REGISTRY=true for local development.",
            err
        ),
    }
    let registry = Arc::new(registry);
    let registry_raw = Arc::new(registry_raw);

    // Load audit log
    let audit_log = Arc::new(audit::load(&config.audit_log_path).await?);
    tracing::info!(entries = audit_log.len(), "Audit log loaded");

    // Load identities
    let identities = Arc::new(identity::load(&config.identities_path).await?);
    tracing::info!(count = identities.len(), "Identities loaded");

    // Load contributions (validates voter pubkeys against identities)
    let contributions = contributions::load(&config.contributions_path, &identities).await?;
    let proposals = Arc::new(contributions);
    tracing::info!(count = proposals.len(), "Contributions loaded");

    // Create peer cache from endorsements
    let peer_cache = Arc::new(federation::PeerCache::new(
        registry.endorsements.clone(),
        public_key.to_z32(),
    ));
    tracing::info!(peers = peer_cache.peer_count().await, "Peer cache initialized");

    // Create shutdown channel
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    // Run initial refresh before server starts
    peer_cache.refresh_all().await;

    // Spawn background refresh loop (every 5 minutes)
    let refresh_cache = Arc::clone(&peer_cache);
    let refresh_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
        let mut shutdown_rx = shutdown_rx;
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    refresh_cache.refresh_all().await;
                }
                _ = shutdown_rx.changed() => {
                    tracing::info!("Peer cache refresh loop shutting down");
                    break;
                }
            }
        }
    });

    // Create MCP handler with shared registry and match config
    let pubkey_z32 = public_key.to_z32();
    let mcp_handler = mcp::McpHandler::new(
        Arc::clone(&registry),
        match_config,
        pubkey_z32,
        Arc::clone(&audit_log),
        Arc::clone(&identities),
        Arc::clone(&proposals),
        Arc::clone(&peer_cache),
    );

    // Build application state
    let app_state = Arc::new(server::AppState {
        mcp_handler,
        registry,
        registry_raw,
        pubkey: public_key,
        audit_log,
        identities,
        proposals,
        peer_cache,
    });

    // Build router with routes and middleware
    let app = server::build_router(app_state);

    // Bind to configured address and start server
    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!(port = config.port, "Server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    // Signal background tasks to stop
    let _ = shutdown_tx.send(true);
    // Wait for refresh loop to finish (clean shutdown)
    let _ = refresh_handle.await;

    Ok(())
}
