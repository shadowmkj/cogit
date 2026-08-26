// ==============================================================================
// GitHub CLI (`gh`) Integration
// ==============================================================================
//
// Delegates Pull Request creation to the official GitHub CLI tool `gh pr create`.

use anyhow::{Context, Result, bail};
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

/// Creates a Pull Request using GitHub CLI `gh pr create`.
pub fn create_github_pr(title: &str, body: &str) -> Result<String> {
    let mut temp_file = NamedTempFile::new().context("create temporary file for PR body")?;
    temp_file
        .write_all(body.as_bytes())
        .context("write PR body to temporary file")?;
    temp_file.flush().context("flush PR body temporary file")?;

    let temp_path = temp_file.path();

    let output = Command::new("gh")
        .arg("pr")
        .arg("create")
        .arg("--title")
        .arg(title)
        .arg("--body-file")
        .arg(temp_path)
        .output()
        .context("execute 'gh pr create' command. Is GitHub CLI installed and authenticated?")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        bail!("'gh pr create' failed: {}", err.trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(stdout)
}
