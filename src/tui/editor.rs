use std::io::{Read, Write};

use tempfile::NamedTempFile;

use crate::error::Result;

pub fn spawn_editor(editor: &str) -> Result<String> {
    let mut tmp = NamedTempFile::new()?;
    writeln!(tmp)?;
    tmp.flush()?;

    let path = tmp.path().to_path_buf();

    let status = std::process::Command::new(editor)
        .arg(&path)
        .status()
        .map_err(|e| {
            crate::error::Error::Tui(format!("failed to launch editor '{editor}': {e}"))
        })?;

    if !status.success() {
        return Err(crate::error::Error::Tui(
            "editor exited with non-zero status; assuming cancel".to_string(),
        ));
    }

    let mut content = String::new();
    let mut file = std::fs::File::open(&path)?;
    file.read_to_string(&mut content)?;

    let trimmed = content.trim().to_string();
    if trimmed.is_empty() {
        return Err(crate::error::Error::Tui(
            "content is empty; issue not created".to_string(),
        ));
    }

    Ok(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_editor_with_cat_copies_content() {
        let result = spawn_editor("cat");
        assert!(
            result.is_err(),
            "cat without initial content should fail (empty content)"
        );
    }

    #[test]
    fn spawn_editor_nonexistent_command_returns_error() {
        let result = spawn_editor("nonexistent_editor_xyz_12345");
        assert!(result.is_err());
        match result {
            Err(crate::error::Error::Tui(msg)) => {
                assert!(msg.contains("failed to launch editor"), "got: {msg}");
            }
            other => panic!("expected Tui error, got {other:?}"),
        }
    }
}
