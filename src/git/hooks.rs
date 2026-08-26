// ==============================================================================
// Git Hook Management
// ==============================================================================
//
// Manages the installation, uninstallation, and lifecycle of the `prepare-commit-msg`
// hook script within the local repository's `.git/hooks/` directory.

use anyhow::{Context, Result, bail};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

const HOOK_SCRIPT_CONTENT: &str = r#"#!/usr/bin/env sh
# Installed by Cogit (https://github.com/shadowmkj/cogit)
# Automatically populates commit message buffer when no message is provided.
if command -v cogit >/dev/null 2>&1; then
    cogit hook prepare-commit-msg "$1" "$2" "$3"
fi
"#;

/// Locates the active `.git/hooks` directory by querying Git.
pub fn get_git_hooks_dir() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--git-path", "hooks"])
        .output()
        .context("locate git hooks directory")?;

    if !output.status.success() {
        bail!("Failed to locate git hooks directory. Are you inside a Git repository?");
    }

    let hooks_rel_or_abs = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let hooks_path = PathBuf::from(hooks_rel_or_abs);

    if hooks_path.is_absolute() {
        Ok(hooks_path)
    } else {
        let current_dir = std::env::current_dir().context("get current directory")?;
        Ok(current_dir.join(hooks_path))
    }
}

/// Installs the prepare-commit-msg hook into the target hooks directory.
pub fn install_hook_at(hooks_dir: &Path) -> Result<PathBuf> {
    fs::create_dir_all(hooks_dir).context("create git hooks directory")?;
    let hook_path = hooks_dir.join("prepare-commit-msg");

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&hook_path)
        .context("create prepare-commit-msg hook file")?;

    file.write_all(HOOK_SCRIPT_CONTENT.as_bytes())
        .context("write hook script content")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&hook_path, fs::Permissions::from_mode(0o755))
            .context("set executable permissions on hook script")?;
    }

    Ok(hook_path)
}

/// Installs the prepare-commit-msg hook in the current repository.
pub fn install_prepare_commit_msg_hook() -> Result<PathBuf> {
    let hooks_dir = get_git_hooks_dir().context("resolve git hooks directory")?;
    install_hook_at(&hooks_dir)
}

/// Uninstalls the hook from the target hooks directory.
pub fn uninstall_hook_at(hooks_dir: &Path) -> Result<()> {
    let hook_path = hooks_dir.join("prepare-commit-msg");
    if hook_path.exists() {
        fs::remove_file(&hook_path).context("remove prepare-commit-msg hook file")?;
    }
    Ok(())
}

/// Uninstalls the prepare-commit-msg hook from the current repository.
pub fn uninstall_prepare_commit_msg_hook() -> Result<()> {
    let hooks_dir = get_git_hooks_dir().context("resolve git hooks directory")?;
    uninstall_hook_at(&hooks_dir)
}

/// Checks whether the Cogit prepare-commit-msg hook is installed.
pub fn is_prepare_commit_msg_hook_installed() -> Result<bool> {
    let hooks_dir = get_git_hooks_dir().context("resolve git hooks directory")?;
    let hook_path = hooks_dir.join("prepare-commit-msg");
    if !hook_path.exists() {
        return Ok(false);
    }
    let content = fs::read_to_string(&hook_path).unwrap_or_default();
    Ok(content.contains("cogit hook prepare-commit-msg"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_install_and_uninstall_hook_in_dir() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let hooks_dir = temp_dir.path().join(".git").join("hooks");
        fs::create_dir_all(&hooks_dir).expect("create hooks dir");

        let hook_file = hooks_dir.join("prepare-commit-msg");
        assert!(!hook_file.exists());

        let installed_path = install_hook_at(&hooks_dir).expect("install hook");
        assert!(installed_path.exists());

        let content = fs::read_to_string(&installed_path).expect("read hook file");
        assert!(content.contains("cogit hook prepare-commit-msg"));

        uninstall_hook_at(&hooks_dir).expect("uninstall hook");
        assert!(!hook_file.exists());
    }
}
