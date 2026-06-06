use std::path::Path;

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn init(log_dir: &Path) {
    let log_dir = log_dir.to_path_buf();
    std::fs::create_dir_all(&log_dir).ok();
    let log_path = log_dir.join("doppelganger.log");

    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .ok();

    if let Some(file) = file {
        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(std::sync::Mutex::new(file))
            .with_ansi(false);
        tracing_subscriber::registry().with(file_layer).init();
    }
}
