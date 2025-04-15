use tracing_subscriber::{fmt, fmt::format::FmtSpan, EnvFilter};

pub fn init_logging() {
    // Only initialize if not already set
    if tracing_log::LogTracer::init().is_err() {
        return; // Logger already initialized
    }

    // Initialize tracing subscriber with formatting and filtering
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // When debug assertions are enabled or the verbose_logging feature is enabled,
    // include more detailed information
    #[cfg(any(debug_assertions, feature = "verbose_logging"))]
    {
        if fmt::Subscriber::builder()
            .with_env_filter(filter)
            .with_span_events(FmtSpan::CLOSE)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .try_init()
            .is_err()
        {
            // Subscriber already set, nothing to do
        }
    }

    // In release mode without the verbose_logging feature, use an extremely minimal format
    #[cfg(not(any(debug_assertions, feature = "verbose_logging")))]
    {
        // Custom minimal format that only shows the level and message
        let format = fmt::format::Format::default()
            .without_time() // Don't show timestamp
            .compact() // Use compact format
            .with_target(false) // Don't show target
            .with_level(true); // Show level

        if fmt::Subscriber::builder()
            .with_env_filter(filter)
            .with_span_events(FmtSpan::NONE)
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_file(false)
            .with_line_number(false)
            .with_ansi(true)
            .event_format(format)
            .try_init()
            .is_err()
        {
            // Subscriber already set, nothing to do
        }
    }
}
