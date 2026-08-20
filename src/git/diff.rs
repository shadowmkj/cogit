// ==============================================================================
// Staged Diff Extraction and Sanitization
// ==============================================================================
//
// Extracts staged Git diffs and sanitizes the contents by stripping non-informative
// lockfiles and binary markers, capping token bounds to avoid LLM context overflow.

use anyhow::{Context, Result};
use std::process::Command;

/// Staged diff inspection result containing both raw and filtered representation.
#[derive(Debug, Clone)]
pub struct StagedDiff {
    /// Sanitized diff text suitable for sending to the LLM.
    pub content: String,
    /// List of staged file paths.
    pub staged_files: Vec<String>,
    /// List of lockfiles or binary files omitted from the LLM prompt.
    pub omitted_files: Vec<String>,
    /// Whether the diff was truncated due to size limits.
    pub is_truncated: bool,
}

/// Known dependency lockfiles and generated indexes that introduce large token volumes
/// without providing semantic context about the developer's intent.
const NOISY_LOCKFILES: &[&str] = &[
    "Cargo.lock",
    "package-lock.json",
    "pnpm-lock.yaml",
    "yarn.lock",
    "bun.lockb",
    "go.sum",
    "poetry.lock",
    "Pipfile.lock",
    "composer.lock",
    "Gemfile.lock",
];

/// Checks if the current working directory is inside a Git repository.
///
/// Uses `git rev-parse --is-inside-work-tree` because it accurately returns true
/// for both standard repository root directories, subdirectories, git worktrees,
/// and git submodules.
pub fn is_git_repository() -> Result<bool> {
    let output = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .context("execute git rev-parse to check repository status")?;

    Ok(output.status.success())
}

/// Checks if there are any staged changes in the repository.
///
/// `git diff --staged --quiet` returns an exit code of 1 when there are differences,
/// and 0 when the staged area is completely clean.
pub fn has_staged_changes() -> Result<bool> {
    let output = Command::new("git")
        .args(["diff", "--staged", "--quiet"])
        .output()
        .context("execute git diff --staged --quiet to inspect staged changes")?;

    Ok(output.status.code() == Some(1))
}

/// Retrieves list of staged file paths using `git diff --staged --name-only`.
pub fn get_staged_files() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["diff", "--staged", "--name-only"])
        .output()
        .context("execute git diff --staged --name-only")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to list staged files: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let files = stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect();

    Ok(files)
}

/// Extracts and sanitizes the staged diff, applying lockfile filtering and truncation limits.
pub fn get_staged_diff(max_chars: usize) -> Result<StagedDiff> {
    let staged_files = get_staged_files().context("retrieve staged files list")?;

    let output = Command::new("git")
        .args(["diff", "--staged"])
        .output()
        .context("execute git diff --staged")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to extract git diff: {}", stderr);
    }

    let raw_diff = String::from_utf8_lossy(&output.stdout);
    let (sanitized_diff, omitted_files) = filter_noisy_files(&raw_diff);

    let (content, is_truncated) = truncate_diff(&sanitized_diff, max_chars);

    Ok(StagedDiff {
        content,
        staged_files,
        omitted_files,
        is_truncated,
    })
}

/// Filters out lockfiles and binary differences from the raw unified diff string.
///
/// Stripping lockfile diffs prevents token exhaustion and avoids distracting the LLM
/// with package hash updates, while still appending a note listing which files were omitted.
fn filter_noisy_files(raw_diff: &str) -> (String, Vec<String>) {
    let mut clean_sections = Vec::new();
    let mut omitted_files = Vec::new();

    let sections: Vec<&str> = raw_diff.split("diff --git ").collect();

    for (index, section) in sections.into_iter().enumerate() {
        if section.trim().is_empty() {
            continue;
        }

        let section_text = if index == 0 && !raw_diff.starts_with("diff --git ") {
            section.to_string()
        } else {
            format!("diff --git {}", section)
        };

        let first_line = section_text.lines().next().unwrap_or_default();
        let filename = extract_filename_from_diff_header(first_line);

        let is_noisy_lockfile = filename.as_ref().is_some_and(|f| {
            std::path::Path::new(f)
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|name| NOISY_LOCKFILES.contains(&name))
        });

        let is_binary =
            section_text.contains("Binary files ") || section_text.contains("GIT binary patch");

        if is_noisy_lockfile {
            if let Some(name) = filename {
                omitted_files.push(format!("{} (lockfile diff omitted)", name));
            }
        } else if is_binary {
            if let Some(name) = filename {
                omitted_files.push(format!("{} (binary file omitted)", name));
            }
        } else {
            clean_sections.push(section_text);
        }
    }

    let mut result = clean_sections.join("");
    if !omitted_files.is_empty() {
        result.push_str("\n\n# Omitted files summary:\n");
        for omitted in &omitted_files {
            result.push_str(&format!("# - {}\n", omitted));
        }
    }

    (result, omitted_files)
}

/// Extracts the destination filepath from standard unified diff headers `diff --git a/... b/...`.
fn extract_filename_from_diff_header(header: &str) -> Option<String> {
    let mut parts = header.split_whitespace();
    if parts.next()? == "diff" && parts.next()? == "--git" {
        let _a_path = parts.next()?;
        let b_path = parts.next()?;
        return Some(b_path.strip_prefix("b/").unwrap_or(b_path).to_string());
    }
    None
}

/// Truncates the diff string at the nearest newline boundary if it exceeds max_chars.
///
/// Truncating at a newline boundary avoids breaking multi-byte UTF-8 sequences and preserves
/// unified diff line structure.
fn truncate_diff(diff: &str, max_chars: usize) -> (String, bool) {
    if diff.len() <= max_chars {
        return (diff.to_string(), false);
    }

    let truncated_slice = &diff[..max_chars];
    let end_index = truncated_slice.rfind('\n').unwrap_or(max_chars);

    let mut truncated = diff[..end_index].to_string();
    truncated.push_str("\n\n[Diff truncated: staged changes exceeded maximum character limit]");

    (truncated, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_filename_from_diff_header() {
        let header = "diff --git a/src/main.rs b/src/main.rs";
        assert_eq!(
            extract_filename_from_diff_header(header),
            Some("src/main.rs".to_string())
        );
    }

    #[test]
    fn test_filter_noisy_files_removes_lockfile() {
        let sample_diff = "\
diff --git a/Cargo.lock b/Cargo.lock
index 123..456 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -1,3 +1,3 @@
-old = 1
+new = 2
diff --git a/src/lib.rs b/src/lib.rs
index 789..abc 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1 +1 @@
+pub fn test() {}
";
        let (filtered, omitted) = filter_noisy_files(sample_diff);
        assert!(!filtered.contains("-old = 1"));
        assert!(filtered.contains("pub fn test() {}"));
        assert_eq!(omitted.len(), 1);
        assert!(omitted[0].contains("Cargo.lock"));
    }

    #[test]
    fn test_truncate_diff_exceeding_limit() {
        let diff = "line 1\nline 2\nline 3\nline 4\n";
        let (truncated, is_truncated) = truncate_diff(diff, 15);
        assert!(is_truncated);
        assert!(truncated.contains("line 1\nline 2"));
        assert!(truncated.contains("[Diff truncated:"));
    }
}
