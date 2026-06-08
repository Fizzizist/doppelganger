use std::sync::OnceLock;
use tracing_subscriber::EnvFilter;

static LOGGING_INIT: OnceLock<()> = OnceLock::new();

pub fn init() {
    LOGGING_INIT.get_or_init(|| {
        let log_file = std::fs::File::create(".doppelganger.log").ok();
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));

        if let Some(file) = log_file {
            tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .with_writer(file)
                .with_ansi(false)
                .init();
        }
    });
}
