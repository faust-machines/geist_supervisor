use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

pub fn init_logging() {
    // Only initialize if not already set
    if tracing_log::LogTracer::init().is_err() {
        return; // Logger already initialized
    }

    // Initialize tracing subscriber with formatting and filtering
    if tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
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
