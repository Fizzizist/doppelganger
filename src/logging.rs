use std::path::Path;

pub fn init(log_path: &Path) -> std::io::Result<()> {
    let file = std::fs::File::create(log_path)?;
    tracing_subscriber::fmt()
        .with_writer(file)
        .with_ansi(false)
        .init();
    Ok(())
}
