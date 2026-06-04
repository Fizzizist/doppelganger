use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;
use std::time::SystemTime;

use crate::error::Result;

static LOGGER: std::sync::OnceLock<Mutex<BufWriter<File>>> = std::sync::OnceLock::new();

pub fn init(log_file_path: &Path) -> Result<()> {
    let file = File::options()
        .create(true)
        .append(true)
        .open(log_file_path)?;
    let writer = BufWriter::new(file);
    LOGGER
        .set(Mutex::new(writer))
        .expect("logger already initialized");
    Ok(())
}

pub fn log(msg: &str) {
    let Some(logger) = LOGGER.get() else {
        return;
    };
    let Ok(mut writer) = logger.lock() else {
        return;
    };
    let timestamp = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => format!("{}", d.as_secs()),
        Err(_) => String::from("unknown"),
    };
    let _ = writer.write_all(format!("[{timestamp}] {msg}\n").as_bytes());
    let _ = writer.flush();
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::logging::log(&format!($($arg)*));
    };
}
