use unpackrr::ui;

slint::include_modules!();

fn main() -> anyhow::Result<()> {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Starting Unpackrr-rs v{}", env!("CARGO_PKG_VERSION"));

    // Create and run the main window
    let main_window = MainWindow::new()?;

    tracing::info!("UI initialized successfully");

    main_window.run()?;

    Ok(())
}
