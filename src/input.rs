use std::io::Read;

pub fn resolve_content(arg: Option<String>) -> crate::error::Result<String> {
    match arg {
        Some(s) => {
            let trimmed = s.trim().to_string();
            if trimmed.is_empty() {
                Err(crate::error::Error::Validation(
                    "content must not be empty".to_string(),
                ))
            } else {
                Ok(trimmed)
            }
        }
        None => {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            let content = buf.trim().to_string();
            if content.is_empty() {
                Err(crate::error::Error::Validation(
                    "content must not be empty".to_string(),
                ))
            } else {
                Ok(content)
            }
        }
    }
}
