use std::fs::File;
use tracing_subscriber::EnvFilter;

pub fn init() {
    let log_file = File::create(".doppelganger.log").ok();
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));

    if let Some(file) = log_file {
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_writer(file)
            .with_ansi(false)
            .init();
    }
}
