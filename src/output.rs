use serde::Serialize;

use crate::error::Result;

pub fn to_json<T: Serialize>(value: &T) -> Result<String> {
    Ok(serde_json::to_string_pretty(value)?)
}

pub fn output<T: Serialize>(value: &T) -> Result<()> {
    let json = to_json(value)?;
    // Safety: write to stdout is permitted in output module for CLI output
    use std::io::Write;
    let mut stdout = std::io::stdout();
    stdout.write_all(json.as_bytes())?;
    stdout.write_all(b"\n")?;
    stdout.flush()?;
    Ok(())
}
