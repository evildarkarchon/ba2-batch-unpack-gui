use unpackrr::{config::AppConfig, logging, ui};

fn main() -> anyhow::Result<()> {
    // Load configuration (if available)
    let config = AppConfig::load().ok();

    // Initialize logging system
    // This sets up both console and file logging with rotation
    logging::init(config.as_ref())?;

    tracing::info!("Starting Unpackrr-rs v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!(
        "Log directory: {}",
        logging::get_log_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "Unknown".to_string())
    );

    if let Some(ref cfg) = config {
        tracing::info!("Configuration loaded successfully");
        tracing::debug!("Debug mode: {}", cfg.advanced.show_debug);
        tracing::debug!("Log level: {:?}", cfg.advanced.log_level);
    } else {
        tracing::warn!("Configuration not found, using defaults");
    }

    // Run the UI (this will initialize and run the Slint event loop)
    ui::run()?;

    tracing::info!("Application shutting down");

    Ok(())
}
