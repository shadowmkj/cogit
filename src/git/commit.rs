// ==============================================================================
// Safe Git Commit Execution
// ==============================================================================
//
// Performs the actual git commit operation by writing the message to a temporary
// file and executing `git commit -F`. This approach guarantees support for multi-line
// commit descriptions, prevents shell escaping bugs, and respects user pre-commit
// hooks and GPG commit signing setups.

use anyhow::{Context, Result};
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

/// Commits staged changes with the provided commit message.
///
/// Writing the commit message to a temporary file and passing it to `git commit -F`
/// provides several crucial advantages over `-m`:
/// 1. Multi-line commit subjects and bodies are passed with exact newline formatting.
/// 2. Shell quote escaping and command interpolation hazards are eliminated.
/// 3. Pre-commit hooks, commit-msg hooks, and GPG/SSH commit signing are fully preserved.
pub fn execute_commit(message: &str) -> Result<String> {
    let mut temp_file =
        NamedTempFile::new().context("create temporary file for git commit message")?;

    temp_file
        .write_all(message.as_bytes())
        .context("write commit message to temporary file")?;

    temp_file
        .flush()
        .context("flush temporary commit message file")?;

    let temp_path = temp_file.path();

    let output = Command::new("git")
        .arg("commit")
        .arg("-F")
        .arg(temp_path)
        .output()
        .context("execute git commit command")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        let error_msg = if !stderr.trim().is_empty() {
            stderr.trim().to_string()
        } else {
            stdout.trim().to_string()
        };
        anyhow::bail!("git commit failed: {}", error_msg);
    }

    let combined_output = if !stdout.trim().is_empty() {
        stdout.trim().to_string()
    } else {
        stderr.trim().to_string()
    };

    Ok(combined_output)
}
