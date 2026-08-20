// ==============================================================================
// External Editor Integration
// ==============================================================================
//
// Spawns the user's configured or system $EDITOR to allow manual editing of
// generated commit messages in a temporary file buffer.

use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

/// Resolves the command to invoke for editing files based on configuration and environment.
pub fn get_editor_command(configured_editor: Option<&str>) -> String {
    if let Some(cmd) = configured_editor.filter(|c| !c.trim().is_empty()) {
        return cmd.trim().to_string();
    }

    for var in ["GIT_EDITOR", "VISUAL", "EDITOR"] {
        if let Ok(editor) = env::var(var) {
            let trimmed = editor.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }

    if cfg!(windows) {
        "notepad".to_string()
    } else {
        "nano".to_string()
    }
}

/// Spawns the editor with the initial message and returns the edited content.
pub fn edit_message_in_editor(
    initial_message: &str,
    configured_editor: Option<&str>,
) -> Result<String> {
    let editor_cmd = get_editor_command(configured_editor);

    let mut temp_file = NamedTempFile::new().context("create temporary file for editor buffer")?;

    temp_file
        .write_all(initial_message.as_bytes())
        .context("write initial commit message to temporary file")?;

    temp_file
        .flush()
        .context("flush initial commit message to temporary file")?;

    let temp_path = temp_file.path().to_path_buf();

    // Parse editor command and arguments (e.g. "code --wait", "subl -w", "nvim")
    let parts = shlex::split(&editor_cmd).unwrap_or_else(|| vec![editor_cmd.clone()]);
    let (program, args) = match parts.split_first() {
        Some((prog, rest)) => (prog, rest),
        None => (&editor_cmd, [].as_slice()),
    };

    let mut command = Command::new(program);
    command.args(args);
    command.arg(&temp_path);

    let status = command.status().with_context(|| {
        format!(
            "launch editor '{}' with path {}",
            program,
            temp_path.display()
        )
    })?;

    if !status.success() {
        anyhow::bail!("Editor process exited with non-zero status");
    }

    let edited_content = fs::read_to_string(&temp_path).with_context(|| {
        format!(
            "read back edited commit message from {}",
            temp_path.display()
        )
    })?;

    Ok(edited_content.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_editor_command_precedence() {
        assert_eq!(get_editor_command(Some("helix")), "helix".to_string());
    }
}
