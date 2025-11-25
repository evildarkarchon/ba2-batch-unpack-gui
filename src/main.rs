use unpackrr::{config::AppConfig, logging, ui};
use std::panic;

fn main() -> anyhow::Result<()> {
    // Load configuration (if available)
    let config = AppConfig::load().ok();

    // Initialize logging system
    // This sets up both console and file logging with rotation
    logging::init(config.as_ref())?;

    // Phase 3.3: Set up panic handler to log panics
    panic::set_hook(Box::new(|panic_info| {
        let payload = panic_info.payload();
        let message = payload.downcast_ref::<&str>().map_or_else(
            || {
                payload
                    .downcast_ref::<String>()
                    .cloned()
                    .unwrap_or_else(|| "Unknown panic payload".to_string())
            },
            |s| (*s).to_string(),
        );

        let location = panic_info.location().map_or_else(
            || "Unknown location".to_string(),
            |loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()),
        );

        tracing::error!("PANIC occurred at {}: {}", location, message);
    }));

    tracing::info!("Starting Unpackrr-rs v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!(
        "Log directory: {}",
        logging::get_log_dir().map_or_else(|_| "Unknown".to_string(), |p| p.display().to_string())
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
