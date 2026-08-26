// ==============================================================================
// Git Branch Context & History Bridge
// ==============================================================================
//
// Extracts commit logs, multi-commit diffs against base branches, uncommitted
// working tree diffs, and handles branch creation and checkout.

use crate::git::diff::{StagedDiff, filter_noisy_files, truncate_diff};
use anyhow::{Context, Result, bail};
use std::process::Command;

/// Parses raw git log output into non-empty lines.
pub fn parse_commit_lines(raw_log: &str) -> Vec<String> {
    raw_log
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

/// Detects the repository's primary base branch (`main` or `master`).
pub fn get_default_base_branch() -> Result<String> {
    // 1. Check if 'main' exists
    let main_check = Command::new("git")
        .args(["rev-parse", "--verify", "main"])
        .output();
    if let Ok(out) = main_check
        && out.status.success()
    {
        return Ok("main".to_string());
    }

    // 2. Check if 'master' exists
    let master_check = Command::new("git")
        .args(["rev-parse", "--verify", "master"])
        .output();
    if let Ok(out) = master_check
        && out.status.success()
    {
        return Ok("master".to_string());
    }

    // 3. Fallback to origin/main or origin/master
    Ok("main".to_string())
}

/// Retrieves list of one-line commit summaries on the current branch since `base_branch`.
pub fn get_branch_commits(base_branch: &str) -> Result<Vec<String>> {
    let range = format!("{}..HEAD", base_branch);
    let output = Command::new("git")
        .args(["log", &range, "--oneline"])
        .output()
        .with_context(|| format!("retrieve branch commits between '{}' and HEAD", base_branch))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        bail!(
            "Failed to get commits for range '{}': {}",
            range,
            err.trim()
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_commit_lines(&stdout))
}

/// Retrieves the full diff between `base_branch` and the current branch (`git diff base...HEAD`).
pub fn get_branch_diff(base_branch: &str, max_chars: usize) -> Result<StagedDiff> {
    let range = format!("{}...HEAD", base_branch);
    let output = Command::new("git")
        .args(["diff", &range])
        .output()
        .with_context(|| format!("retrieve diff for range '{}'", range))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to get diff for range '{}': {}", range, err.trim());
    }

    let raw_diff = String::from_utf8_lossy(&output.stdout).to_string();
    let (sanitized_diff, omitted_files) = filter_noisy_files(&raw_diff);
    let (content, is_truncated) = truncate_diff(&sanitized_diff, max_chars);

    Ok(StagedDiff {
        content,
        staged_files: vec![],
        omitted_files,
        is_truncated,
    })
}

/// Retrieves uncommitted working tree changes (both staged and unstaged) for branch suggestion.
pub fn get_working_tree_diff(max_chars: usize) -> Result<StagedDiff> {
    let output = Command::new("git")
        .args(["diff", "HEAD"])
        .output()
        .context("retrieve working tree diff")?;

    let raw_diff = if output.status.success() {
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        // Fallback for repository with no commits
        let uncommitted = Command::new("git")
            .args(["diff"])
            .output()
            .context("git diff fallback")?;
        String::from_utf8_lossy(&uncommitted.stdout).to_string()
    };

    let (sanitized, omitted_files) = filter_noisy_files(&raw_diff);
    let (content, is_truncated) = truncate_diff(&sanitized, max_chars);

    Ok(StagedDiff {
        content,
        staged_files: vec![],
        omitted_files,
        is_truncated,
    })
}

/// Checks out a new branch with the given name (`git checkout -b <name>`).
pub fn checkout_new_branch(branch_name: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["checkout", "-b", branch_name])
        .output()
        .with_context(|| format!("checkout new branch '{}'", branch_name))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        bail!(
            "Failed to checkout branch '{}': {}",
            branch_name,
            err.trim()
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_commit_log_output() {
        let raw_log = "ce12574 feat(ui): add interactive prompt\nbdc8b1e chore: update license\n";
        let parsed = parse_commit_lines(raw_log);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0], "ce12574 feat(ui): add interactive prompt");
        assert_eq!(parsed[1], "bdc8b1e chore: update license");
    }

    #[test]
    fn test_empty_commit_log_output() {
        let parsed = parse_commit_lines("\n  \n");
        assert!(parsed.is_empty());
    }
}
