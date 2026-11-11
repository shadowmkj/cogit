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
pub(crate) fn filter_noisy_files(raw_diff: &str) -> (String, Vec<String>) {
    let mut clean_sections = Vec::new();
    let mut omitted_files = Vec::new();

    for section in split_diff_sections(raw_diff) {
        if section.trim().is_empty() {
            continue;
        }

        let first_line = section.lines().next().unwrap_or_default();
        let filename = extract_filename_from_diff_header(first_line);

        let is_noisy_lockfile = filename.as_ref().is_some_and(|f| {
            std::path::Path::new(f)
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|name| NOISY_LOCKFILES.contains(&name))
        });

        let is_binary = section.contains("Binary files ") || section.contains("GIT binary patch");

        if is_noisy_lockfile {
            if let Some(name) = filename {
                omitted_files.push(format!("{} (lockfile diff omitted)", name));
            }
        } else if is_binary {
            if let Some(name) = filename {
                omitted_files.push(format!("{} (binary file omitted)", name));
            }
        } else {
            clean_sections.push(section.to_string());
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

fn split_diff_sections(raw_diff: &str) -> Vec<&str> {
    let mut sections = Vec::new();
    let mut section_start = 0;
    let mut line_start = 0;

    for line in raw_diff.split_inclusive('\n') {
        if line.starts_with("diff --git ") && line_start > section_start {
            sections.push(&raw_diff[section_start..line_start]);
            section_start = line_start;
        }
        line_start += line.len();
    }

    if section_start < raw_diff.len() {
        sections.push(&raw_diff[section_start..]);
    }

    sections
}

/// Extracts the destination filepath from standard unified diff headers `diff --git a/... b/...`.
fn extract_filename_from_diff_header(header: &str) -> Option<String> {
    let paths = header.strip_prefix("diff --git ")?;
    let (_, paths) = parse_git_path(paths)?;
    let (destination, _) = parse_git_path(paths)?;
    destination.strip_prefix("b/").map(str::to_string)
}

// Git uses C-style quoted tokens for paths with whitespace and escaped bytes.
fn parse_git_path(input: &str) -> Option<(String, &str)> {
    let input = input.trim_start();
    let bytes = input.as_bytes();

    if bytes.first() != Some(&b'"') {
        let end = input.find(char::is_whitespace).unwrap_or(input.len());
        if end == 0 {
            return None;
        }
        return Some((input[..end].to_string(), &input[end..]));
    }

    let mut path = Vec::new();
    let mut cursor = 1;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'"' => {
                let path = String::from_utf8_lossy(&path).into_owned();
                return Some((path, &input[cursor + 1..]));
            }
            b'\\' => {
                cursor += 1;
                let escaped = *bytes.get(cursor)?;
                match escaped {
                    b'0'..=b'7' => {
                        let mut value = 0u16;
                        let mut digits = 0;
                        while digits < 3
                            && bytes
                                .get(cursor)
                                .is_some_and(|byte| (b'0'..=b'7').contains(byte))
                        {
                            value = value * 8 + (bytes[cursor] - b'0') as u16;
                            cursor += 1;
                            digits += 1;
                        }
                        path.push(u8::try_from(value).ok()?);
                        continue;
                    }
                    b'a' => path.push(b'\x07'),
                    b'b' => path.push(b'\x08'),
                    b't' => path.push(b'\t'),
                    b'n' => path.push(b'\n'),
                    b'v' => path.push(b'\x0b'),
                    b'f' => path.push(b'\x0c'),
                    b'r' => path.push(b'\r'),
                    b'\\' | b'"' => path.push(escaped),
                    _ => return None,
                }
                cursor += 1;
            }
            byte => {
                path.push(byte);
                cursor += 1;
            }
        }
    }

    None
}

/// Truncates the diff string at the nearest newline boundary if it exceeds max_chars.
///
/// Truncating at a newline boundary avoids breaking multi-byte UTF-8 sequences and preserves
/// unified diff line structure.
pub(crate) fn truncate_diff(diff: &str, max_chars: usize) -> (String, bool) {
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
    fn test_extract_filename_from_quoted_diff_header_with_escapes() {
        let header = r#"diff --git "a/build outputs/Cargo.lock" "b/build outputs/Cargo.lock""#;
        assert_eq!(
            extract_filename_from_diff_header(header),
            Some("build outputs/Cargo.lock".to_string())
        );

        let escaped_header =
            r#"diff --git "a/docs/space\040and quote\".txt" "b/docs/space\040and quote\".txt""#;
        assert_eq!(
            extract_filename_from_diff_header(escaped_header),
            Some("docs/space and quote\".txt".to_string())
        );

        let unicode_header = r#"diff --git "a/caf\303\251/Cargo.lock" "b/caf\303\251/Cargo.lock""#;
        assert_eq!(
            extract_filename_from_diff_header(unicode_header),
            Some("café/Cargo.lock".to_string())
        );
    }

    #[test]
    fn test_filter_noisy_files_preserves_diff_header_text_in_file_content() {
        let sample_diff = concat!(
            "diff --git a/README.md b/README.md\n",
            "index 123..456 100644\n",
            "--- a/README.md\n",
            "+++ b/README.md\n",
            "@@ -1 +1,3 @@\n",
            "+Example: diff --git a/Cargo.lock b/Cargo.lock\n",
            "+Keep this documentation line.\n",
            "diff --git a/src/lib.rs b/src/lib.rs\n",
            "index 789..abc 100644\n",
            "--- a/src/lib.rs\n",
            "+++ b/src/lib.rs\n",
            "@@ -1 +1 @@\n",
            "+pub fn test() {}\n"
        );

        let (filtered, omitted) = filter_noisy_files(sample_diff);

        assert!(filtered.contains("+Example: diff --git a/Cargo.lock b/Cargo.lock\n"));
        assert!(filtered.contains("+Keep this documentation line."));
        assert!(filtered.contains("+pub fn test() {}"));
        assert!(omitted.is_empty());
    }

    #[test]
    fn test_filter_noisy_files_handles_quoted_paths_with_spaces() {
        let sample_diff = concat!(
            "diff --git \"a/build outputs/Cargo.lock\" \"b/build outputs/Cargo.lock\"\n",
            "index 123..456 100644\n",
            "--- \"a/build outputs/Cargo.lock\"\n",
            "+++ \"b/build outputs/Cargo.lock\"\n",
            "@@ -1 +1 @@\n",
            "-old = 1\n",
            "+new = 2\n",
            "diff --git \"a/assets/site image.png\" \"b/assets/site image.png\"\n",
            "index 123..456 100644\n",
            "Binary files \"a/assets/site image.png\" and \"b/assets/site image.png\" differ\n",
            "diff --git \"a/docs/Release Notes.md\" \"b/docs/Release Notes.md\"\n",
            "index 789..abc 100644\n",
            "--- \"a/docs/Release Notes.md\"\n",
            "+++ \"b/docs/Release Notes.md\"\n",
            "@@ -1 +1 @@\n",
            "+Release notes are retained.\n"
        );

        let (filtered, omitted) = filter_noisy_files(sample_diff);

        assert!(!filtered.contains("old = 1"));
        assert!(!filtered.contains("Binary files"));
        assert!(filtered.contains("+Release notes are retained."));
        assert_eq!(
            omitted,
            vec![
                "build outputs/Cargo.lock (lockfile diff omitted)",
                "assets/site image.png (binary file omitted)"
            ]
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
