use serde::Serialize;

pub fn print_json<T: Serialize>(value: &T) -> crate::error::Result<()> {
    let json = serde_json::to_string_pretty(value)?;
    println!("{json}");
    Ok(())
}
