use std::sync::OnceLock;
use tracing_subscriber::EnvFilter;

static LOGGING_INIT: OnceLock<()> = OnceLock::new();

pub fn init() {
    LOGGING_INIT.get_or_init(|| {
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));

        let log_dir = dirs::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        let log_path = log_dir.join("doppelganger").join("doppelganger.log");

        let _ = std::fs::create_dir_all(log_path.parent().expect("log path has parent"));

        let log_file = std::fs::File::create(&log_path).ok();
        if let Some(file) = log_file {
            tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .with_writer(file)
                .with_ansi(false)
                .init();
        }
    });
}
