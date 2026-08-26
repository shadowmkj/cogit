# Git Hook & Workflow Automation Suite Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a comprehensive Git workflow automation suite in Cogit consisting of `prepare-commit-msg` Git hook integration, Pull Request description generator (`cogit pr`), and conventional branch name generator (`cogit branch`).

**Architecture:** Extend the CLI interface using Clap subcommands with root-command backward compatibility. Provide modular Git bridge helpers for hook file manipulation, branch commit history / diff extraction, and branch checkout. Implement prompt builders for PR and branch name generation, and wire cross-platform clipboard copy, GitHub CLI delegation, and coordinator dispatch.

**Tech Stack:** Rust 2024, Clap 4.6 (derive), Tokio, Reqwest, Serve, Tempfile, Inquire, Anyhow.

**Spec:** `docs/superpowers/specs/2026-08-26-git-workflow-automation-design.md`

## Global Constraints

- **Source Code Integrity:** Strictly follow Rust guidelines, `.context()` for errors with simple present tense, `expect()` with concise explanation.
- **Dependency Discipline:** No new external crate additions without necessity (use platform-native command fallbacks for clipboard/git tools).
- **Backwards Compatibility:** Running `cogit` without subcommands MUST continue to execute default commit message generation without breaking existing flags.
- **Literate Programming:** Provide clear explanatory comments for architecture, intent, and decision points.

---

### Task 1: CLI Hierarchy & Subcommand Parser

**Files:**
- Modify: `src/cli.rs`
- Test: `src/cli.rs:tests`

**Interfaces:**
- Produces:
  - `Cli` struct with `command: Option<Commands>` and `commit_args: Args`
  - `Commands` enum with `Hook(HookArgs)`, `Pr(PrArgs)`, `Branch(BranchArgs)`
  - `HookArgs` struct with `action: HookAction`
  - `HookAction` enum with `Install`, `Uninstall`, `PrepareCommitMsg { file: PathBuf, source: Option<String>, sha: Option<String> }`
  - `PrArgs` struct with `base: Option<String>`, `copy: bool`, `create: bool`, `custom_prompt: Option<String>`, `model: Option<String>`, `provider: Option<String>`, `config_path: Option<PathBuf>`
  - `BranchArgs` struct with `prompt: Option<String>`, `checkout: bool`, `branch_type: Option<String>`, `model: Option<String>`, `provider: Option<String>`, `config_path: Option<PathBuf>`

- [ ] **Step 1: Write the failing tests for subcommands in `src/cli.rs`**

```rust
#[test]
fn test_parse_subcommand_hook_install() {
    let cli = Cli::try_parse_from(["cogit", "hook", "install"]).expect("hook install should parse");
    match cli.command {
        Some(Commands::Hook(hook_args)) => match hook_args.action {
            HookAction::Install => {}
            _ => panic!("expected HookAction::Install"),
        },
        _ => panic!("expected Commands::Hook"),
    }
}

#[test]
fn test_parse_subcommand_pr() {
    let cli = Cli::try_parse_from(["cogit", "pr", "--base", "develop", "--copy", "--create"])
        .expect("pr subcommand should parse");
    match cli.command {
        Some(Commands::Pr(pr_args)) => {
            assert_eq!(pr_args.base.as_deref(), Some("develop"));
            assert!(pr_args.copy);
            assert!(pr_args.create);
        }
        _ => panic!("expected Commands::Pr"),
    }
}

#[test]
fn test_parse_subcommand_branch() {
    let cli = Cli::try_parse_from(["cogit", "branch", "add groq provider", "-c", "-t", "feat"])
        .expect("branch subcommand should parse");
    match cli.command {
        Some(Commands::Branch(branch_args)) => {
            assert_eq!(branch_args.prompt.as_deref(), Some("add groq provider"));
            assert!(branch_args.checkout);
            assert_eq!(branch_args.branch_type.as_deref(), Some("feat"));
        }
        _ => panic!("expected Commands::Branch"),
    }
}

#[test]
fn test_root_command_fallback_without_subcommand() {
    let cli = Cli::try_parse_from(["cogit", "-d", "-m", "gpt-4o"]).expect("root args should parse");
    assert!(cli.command.is_none());
    assert!(cli.commit_args.detailed);
    assert_eq!(cli.commit_args.model.as_deref(), Some("gpt-4o"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test cli::tests::test_parse_subcommand_hook_install`
Expected: FAIL (types `Cli`, `Commands`, etc. not yet defined)

- [ ] **Step 3: Implement `Cli`, `Commands`, and subcommand argument structs in `src/cli.rs`**

```rust
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Root CLI parser for Cogit.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "cogit",
    author,
    version,
    about = "Generate conventional commit messages and automate Git workflows using AI",
    subcommand_required = false,
    arg_required_else_help = false
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[command(flatten)]
    pub commit_args: Args,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Manage Git hooks (prepare-commit-msg)
    Hook(HookArgs),
    /// Generate a Pull Request title and Markdown description
    Pr(PrArgs),
    /// Suggest conventional git branch names
    Branch(BranchArgs),
}

#[derive(Parser, Debug, Clone)]
pub struct HookArgs {
    #[command(subcommand)]
    pub action: HookAction,
}

#[derive(Subcommand, Debug, Clone)]
pub enum HookAction {
    /// Install prepare-commit-msg hook in the current repository
    Install,
    /// Uninstall prepare-commit-msg hook from the current repository
    Uninstall,
    /// Internal hook callback invoked by Git
    #[command(hide = true)]
    PrepareCommitMsg {
        /// Path to the commit message buffer file (e.g. .git/COMMIT_EDITMSG)
        file: PathBuf,
        /// Source of the commit message (message, template, merge, squash, commit)
        source: Option<String>,
        /// Commit SHA if amending or rewriting
        sha: Option<String>,
    },
}

#[derive(Parser, Debug, Clone)]
pub struct PrArgs {
    /// Target base branch to compare against (defaults to 'main' or 'master')
    #[arg(short = 'b', long = "base", help = "Target base branch name")]
    pub base: Option<String>,

    /// Copy generated PR markdown to system clipboard
    #[arg(short = 'c', long = "copy", help = "Copy PR Markdown to clipboard")]
    pub copy: bool,

    /// Create PR on GitHub directly via `gh pr create`
    #[arg(long = "create", help = "Create PR on GitHub using gh CLI")]
    pub create: bool,

    /// Override LLM model name
    #[arg(short = 'm', long = "model", help = "Override LLM model name")]
    pub model: Option<String>,

    /// Override active LLM provider
    #[arg(short = 'p', long = "provider", help = "Override active LLM provider")]
    pub provider: Option<String>,

    /// Additional context or issue description to incorporate into PR
    #[arg(long = "prompt", help = "Additional guidance for PR generation")]
    pub custom_prompt: Option<String>,

    /// Custom path to configuration file
    #[arg(long = "config", help = "Path to custom configuration file")]
    pub config_path: Option<PathBuf>,
}

#[derive(Parser, Debug, Clone)]
pub struct BranchArgs {
    /// Task description or issue title to base branch name on
    pub prompt: Option<String>,

    /// Automatically checkout the selected branch
    #[arg(short = 'c', long = "checkout", help = "Create and checkout the branch")]
    pub checkout: bool,

    /// Target prefix type (e.g. 'feat', 'fix', 'chore', 'refactor')
    #[arg(short = 't', long = "type", help = "Branch type prefix")]
    pub branch_type: Option<String>,

    /// Override LLM model name
    #[arg(short = 'm', long = "model", help = "Override LLM model name")]
    pub model: Option<String>,

    /// Override active LLM provider
    #[arg(short = 'p', long = "provider", help = "Override active LLM provider")]
    pub provider: Option<String>,

    /// Custom path to configuration file
    #[arg(long = "config", help = "Path to custom configuration file")]
    pub config_path: Option<PathBuf>,
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test cli::tests`
Expected: PASS

- [ ] **Step 5: Commit changes**

```bash
git add src/cli.rs
git commit -m "feat(cli): add Cli struct with Hook, Pr, and Branch subcommands"
```

---

### Task 2: Git Hook Bridge & File Installer

**Files:**
- Create: `src/git/hooks.rs`
- Modify: `src/git/mod.rs`
- Test: `src/git/hooks.rs:tests`

**Interfaces:**
- Produces:
  - `get_git_hooks_dir() -> Result<PathBuf>`
  - `install_prepare_commit_msg_hook() -> Result<PathBuf>`
  - `uninstall_prepare_commit_msg_hook() -> Result<()>`
  - `is_prepare_commit_msg_hook_installed() -> Result<bool>`

- [ ] **Step 1: Write the failing tests for hook management in `src/git/hooks.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test git::hooks`
Expected: FAIL (module not defined)

- [ ] **Step 3: Implement `src/git/hooks.rs`**

```rust
// ==============================================================================
// Git Hook Management
// ==============================================================================
//
// Manages the installation and uninstallation of the `prepare-commit-msg` hook
// script within the local repository's `.git/hooks/` directory.

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
```

- [ ] **Step 4: Export hooks in `src/git/mod.rs` and run tests**

Modify `src/git/mod.rs` to add `pub mod hooks;` and re-export `install_prepare_commit_msg_hook`, `uninstall_prepare_commit_msg_hook`, `is_prepare_commit_msg_hook_installed`.
Run: `cargo test git::hooks`
Expected: PASS

- [ ] **Step 5: Commit changes**

```bash
git add src/git/hooks.rs src/git/mod.rs
git commit -m "feat(git): add hook installer, uninstaller, and detection logic"
```

---

### Task 3: Git Branch & History Inspection Bridge

**Files:**
- Create: `src/git/branch_context.rs`
- Modify: `src/git/mod.rs`
- Test: `src/git/branch_context.rs:tests`

**Interfaces:**
- Produces:
  - `get_default_base_branch() -> Result<String>`
  - `get_branch_commits(base_branch: &str) -> Result<Vec<String>>`
  - `get_branch_diff(base_branch: &str, max_chars: usize) -> Result<StagedDiff>`
  - `get_working_tree_diff(max_chars: usize) -> Result<StagedDiff>`
  - `checkout_new_branch(branch_name: &str) -> Result<()>`

- [ ] **Step 1: Write the failing tests in `src/git/branch_context.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_commit_log_output() {
        let raw_log = "ce12574 feat(ui): add interactive prompt\nbdc8b1e chore: update license\n";
        let parsed = parse_commit_lines(raw_log);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0], "ce12574 feat(ui): add interactive prompt");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test git::branch_context`
Expected: FAIL

- [ ] **Step 3: Implement `src/git/branch_context.rs`**

```rust
// ==============================================================================
// Git Branch Context & History Bridge
// ==============================================================================
//
// Extracts commit logs, multi-commit diffs against base branches, uncommitted
// working tree diffs, and handles branch creation/checkout.

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
    if let Ok(out) = main_check {
        if out.status.success() {
            return Ok("main".to_string());
        }
    }

    // 2. Check if 'master' exists
    let master_check = Command::new("git")
        .args(["rev-parse", "--verify", "master"])
        .output();
    if let Ok(out) = master_check {
        if out.status.success() {
            return Ok("master".to_string());
        }
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
        bail!("Failed to get commits for range '{}': {}", range, err.trim());
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
    let sanitized_diff = filter_noisy_files(&raw_diff);
    let (content, is_truncated) = truncate_diff(&sanitized_diff, max_chars);

    Ok(StagedDiff {
        content,
        staged_files: vec![],
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
        // Fallback for brand new repository with no commits
        let uncommitted = Command::new("git").args(["diff"]).output().context("git diff fallback")?;
        String::from_utf8_lossy(&uncommitted.stdout).to_string()
    };

    let sanitized = filter_noisy_files(&raw_diff);
    let (content, is_truncated) = truncate_diff(&sanitized, max_chars);

    Ok(StagedDiff {
        content,
        staged_files: vec![],
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
        bail!("Failed to checkout branch '{}': {}", branch_name, err.trim());
    }

    Ok(())
}
```

- [ ] **Step 4: Export in `src/git/mod.rs` and run tests**

Run: `cargo test git::branch_context`
Expected: PASS

- [ ] **Step 5: Commit changes**

```bash
git add src/git/branch_context.rs src/git/mod.rs
git commit -m "feat(git): add branch context, commit history, and checkout helpers"
```

---

### Task 4: LLM Prompt Engineering for PR and Branch Generation

**Files:**
- Modify: `src/llm/prompt.rs`
- Test: `src/llm/prompt.rs:tests`

**Interfaces:**
- Produces:
  - `build_pr_prompt(commits: &[String], diff_content: &str, custom_prompt: Option<&str>) -> (String, String)`
  - `build_branch_prompt(diff_content: Option<&str>, user_hint: Option<&str>, branch_type: Option<&str>) -> (String, String)`
  - `parse_branch_suggestions(raw_response: &str) -> Vec<String>`

- [ ] **Step 1: Write failing tests in `src/llm/prompt.rs`**

```rust
#[test]
fn test_build_pr_prompt_structure() {
    let commits = vec!["abc1234 feat(auth): add jwt support".to_string()];
    let diff = "+ let token = generate_jwt();";
    let (sys, user) = build_pr_prompt(&commits, diff, Some("ticket #123"));

    assert!(sys.contains("Pull Request description"));
    assert!(user.contains("abc1234 feat(auth)"));
    assert!(user.contains("ticket #123"));
}

#[test]
fn test_parse_branch_suggestions() {
    let raw = "1. feat/jwt-auth\n2. `feat/auth-token-generation`\n* feat/jwt-support";
    let branches = parse_branch_suggestions(raw);
    assert_eq!(branches.len(), 3);
    assert_eq!(branches[0], "feat/jwt-auth");
    assert_eq!(branches[1], "feat/auth-token-generation");
    assert_eq!(branches[2], "feat/jwt-support");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test llm::prompt::test_build_pr_prompt_structure`
Expected: FAIL

- [ ] **Step 3: Implement prompt builders and branch parser in `src/llm/prompt.rs`**

```rust
/// Constructs system and user prompts for generating a GitHub Pull Request description.
pub fn build_pr_prompt(
    commits: &[String],
    diff_content: &str,
    custom_prompt: Option<&str>,
) -> (String, String) {
    let system_prompt = r#"You are an expert software engineer and technical lead.
Your task is to generate a comprehensive, well-structured Pull Request (PR) title and Markdown description based on the provided commit history and code diff.

Follow this exact format:

# Title: <Type>(<scope>): <concise imperative summary under 72 chars>

## Summary
<2-3 concise sentences summarizing the objective and impact of these changes.>

## Key Changes
- <Bulleted list of meaningful architectural and functional changes>

## Verification & Testing
- <Bulleted list of test cases, verification steps, and automated test commands run>
"#;

    let mut user_prompt = String::new();
    user_prompt.push_str("Commit History on this branch:\n");
    if commits.is_empty() {
        user_prompt.push_str("(No commits yet - uncommitted diff provided)\n");
    } else {
        for c in commits {
            user_prompt.push_str(&format!("- {}\n", c));
        }
    }

    user_prompt.push_str("\nCode Diff:\n```diff\n");
    user_prompt.push_str(diff_content);
    user_prompt.push_str("\n```\n");

    if let Some(hint) = custom_prompt.filter(|h| !h.trim().is_empty()) {
        user_prompt.push_str(&format!("\nAdditional User Instructions:\n{}\n", hint.trim()));
    }

    (system_prompt.to_string(), user_prompt)
}

/// Constructs system and user prompts for generating clean Git branch names.
pub fn build_branch_prompt(
    diff_content: Option<&str>,
    user_hint: Option<&str>,
    branch_type: Option<&str>,
) -> (String, String) {
    let system_prompt = r#"You are a developer workflow assistant.
Generate 3 to 5 clean, standard-compliant Git branch names.
Rules:
1. Use kebab-case with standard prefixes (e.g. feat/, fix/, chore/, refactor/, docs/).
2. Keep names concise (under 40 characters), descriptive, and lower-case.
3. Return ONLY a plain list of branch names, one per line. Do not include markdown formatting or numbering.
"#;

    let mut user_prompt = String::new();
    if let Some(t) = branch_type.filter(|s| !s.trim().is_empty()) {
        user_prompt.push_str(&format!("Preferred Prefix / Type: {}\n", t.trim()));
    }
    if let Some(hint) = user_hint.filter(|s| !s.trim().is_empty()) {
        user_prompt.push_str(&format!("Task Description: {}\n", hint.trim()));
    }
    if let Some(diff) = diff_content.filter(|s| !s.trim().is_empty()) {
        user_prompt.push_str(&format!("Working Diff:\n```diff\n{}\n```\n", diff));
    }

    (system_prompt.to_string(), user_prompt)
}

/// Parses raw LLM response into sanitized kebab-case branch names.
pub fn parse_branch_suggestions(raw_response: &str) -> Vec<String> {
    raw_response
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            let without_bullet = trimmed
                .trim_start_matches(|c: char| c.is_numeric() || c == '.' || c == '-' || c == '*' || c == ' ')
                .trim()
                .trim_matches('`')
                .trim_matches('"')
                .trim_matches('\'')
                .trim();
            without_bullet.to_string()
        })
        .filter(|s| !s.is_empty() && (s.contains('/') || s.contains('-')))
        .collect()
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test llm::prompt`
Expected: PASS

- [ ] **Step 5: Commit changes**

```bash
git add src/llm/prompt.rs
git commit -m "feat(llm): add PR and branch name prompt builders and parser"
```

---

### Task 5: UI Helpers — Clipboard Copy, GitHub CLI, and Branch Picker

**Files:**
- Create: `src/ui/clipboard.rs`
- Create: `src/ui/gh.rs`
- Create: `src/ui/branch_picker.rs`
- Modify: `src/ui/mod.rs`
- Test: `src/ui/branch_picker.rs:tests`

**Interfaces:**
- Produces:
  - `copy_to_clipboard(text: &str) -> Result<()>`
  - `create_github_pr(title: &str, body: &str) -> Result<String>`
  - `select_branch_name(candidates: &[String]) -> Result<String>`

- [ ] **Step 1: Implement `src/ui/clipboard.rs`**

```rust
// ==============================================================================
// Cross-Platform Clipboard Helper
// ==============================================================================

use anyhow::{Context, Result, bail};
use std::io::Write;
use std::process::{Command, Stdio};

/// Copies text content to the OS system clipboard using native clipboard tools.
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    let mut child = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .spawn()
        .context("spawn pbcopy command")?;

    #[cfg(target_os = "windows")]
    let mut child = Command::new("clip")
        .stdin(Stdio::piped())
        .spawn()
        .context("spawn clip command")?;

    #[cfg(all(unix, not(target_os = "macos")))]
    let mut child = {
        if Command::new("wl-copy").arg("--version").output().is_ok() {
            Command::new("wl-copy").stdin(Stdio::piped()).spawn().context("spawn wl-copy")?
        } else if Command::new("xclip").arg("-version").output().is_ok() {
            Command::new("xclip").args(["-selection", "clipboard"]).stdin(Stdio::piped()).spawn().context("spawn xclip")?
        } else {
            bail!("No supported clipboard tool found (wl-copy or xclip).");
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes()).context("write to clipboard tool stdin")?;
    }

    let status = child.wait().context("wait for clipboard tool to finish")?;
    if !status.success() {
        bail!("Clipboard tool exited with non-zero status");
    }

    Ok(())
}
```

- [ ] **Step 2: Implement `src/ui/gh.rs`**

```rust
// ==============================================================================
// GitHub CLI (`gh`) PR Integration
// ==============================================================================

use anyhow::{Context, Result, bail};
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

/// Creates a Pull Request using GitHub CLI `gh pr create`.
pub fn create_github_pr(title: &str, body: &str) -> Result<String> {
    let mut temp_file = NamedTempFile::new().context("create temporary file for PR body")?;
    temp_file.write_all(body.as_bytes()).context("write PR body to temp file")?;
    temp_file.flush().context("flush PR body temp file")?;

    let output = Command::new("gh")
        .args([
            "pr",
            "create",
            "--title",
            title,
            "--body-file",
            temp_file.path().to_str().expect("valid temp path"),
        ])
        .output()
        .context("execute 'gh pr create' command. Is GitHub CLI installed and authenticated?")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        bail!("'gh pr create' failed: {}", err.trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(stdout)
}
```

- [ ] **Step 3: Implement `src/ui/branch_picker.rs`**

```rust
// ==============================================================================
// Interactive Branch Selection Menu
// ==============================================================================

use anyhow::{Context, Result};
use inquire::Select;

/// Prompts the user to select from a list of generated branch names.
pub fn select_branch_name(candidates: &[String]) -> Result<String> {
    if candidates.is_empty() {
        anyhow::bail!("No valid branch name candidates were generated.");
    }

    let selection = Select::new("Select a branch name to checkout:", candidates.to_vec())
        .prompt()
        .context("prompt user for branch selection")?;

    Ok(selection)
}
```

- [ ] **Step 4: Export modules in `src/ui/mod.rs` and verify compilation**

Modify `src/ui/mod.rs` to export `clipboard`, `gh`, and `branch_picker`.
Run: `cargo check`
Expected: PASS

- [ ] **Step 5: Commit changes**

```bash
git add src/ui/clipboard.rs src/ui/gh.rs src/ui/branch_picker.rs src/ui/mod.rs
git commit -m "feat(ui): add clipboard copy, GitHub CLI launcher, and branch picker"
```

---

### Task 6: Subcommand Handlers & Main Coordinator Routing

**Files:**
- Modify: `src/main.rs`
- Test: Integration runs of subcommands

**Interfaces:**
- Implements:
  - `run_hook_flow(args: HookArgs, config: &Config) -> Result<()>`
  - `run_pr_flow(args: PrArgs, config: &Config) -> Result<()>`
  - `run_branch_flow(args: BranchArgs, config: &Config) -> Result<()>`
  - `run_commit_flow(args: Args, config: &Config) -> Result<()>`

- [ ] **Step 1: Implement handler functions in `src/main.rs`**

```rust
// Implement run_hook_flow
async fn run_hook_flow(args: cli::HookArgs) -> Result<()> {
    match args.action {
        cli::HookAction::Install => {
            let path = git::hooks::install_prepare_commit_msg_hook()
                .context("install prepare-commit-msg hook")?;
            println!("Successfully installed Git hook at {}", path.display());
        }
        cli::HookAction::Uninstall => {
            git::hooks::uninstall_prepare_commit_msg_hook()
                .context("uninstall prepare-commit-msg hook")?;
            println!("Successfully uninstalled prepare-commit-msg Git hook.");
        }
        cli::HookAction::PrepareCommitMsg { file, source, .. } => {
            // If source is specified (e.g. merge, message, squash, commit), skip silently
            if source.is_some() {
                return Ok(());
            }

            if !git::has_staged_changes().unwrap_or(false) {
                return Ok(());
            }

            let config = Config::load(None).unwrap_or_default();
            let (provider_name, provider) = match config.get_active_provider(None) {
                Ok(p) => p,
                Err(_) => return Ok(()),
            };

            let staged_diff = match git::get_staged_diff(config.preferences.max_diff_chars) {
                Ok(d) => d,
                Err(_) => return Ok(()),
            };

            let (sys_p, usr_p) = build_prompt(&staged_diff.content, &staged_diff.staged_files, config.preferences.detailed, None);
            let client = LlmClient::new();
            if let Ok(msg) = client.generate_commit(&provider_name, &provider, &sys_p, &usr_p).await {
                if !msg.trim().is_empty() {
                    let _ = std::fs::write(&file, format!("{}\n", msg.trim()));
                }
            }
        }
    }
    Ok(())
}

// Implement run_pr_flow
async fn run_pr_flow(args: cli::PrArgs, config: &Config) -> Result<()> {
    let (provider_name, mut provider) = config
        .get_active_provider(args.provider.as_deref())
        .context("resolve active LLM provider")?;
    if let Some(m) = args.model {
        provider.model = m;
    }

    let base_branch = match args.base {
        Some(b) => b,
        None => git::branch_context::get_default_base_branch().context("detect default base branch")?,
    };

    println!("Inspecting commits and diff against base branch [{}]...", base_branch);
    let commits = git::branch_context::get_branch_commits(&base_branch).context("retrieve branch commits")?;
    let diff = git::branch_context::get_branch_diff(&base_branch, config.preferences.max_diff_chars)
        .context("retrieve branch diff")?;

    let (sys_p, usr_p) = llm::prompt::build_pr_prompt(&commits, &diff.content, args.custom_prompt.as_deref());
    let client = LlmClient::new();
    println!("Generating PR description using [{}]...", provider_name);
    let pr_text = client.generate_commit(&provider_name, &provider, &sys_p, &usr_p).await.context("generate PR description")?;

    println!("\n{}\n", pr_text);

    if args.copy {
        ui::clipboard::copy_to_clipboard(&pr_text).context("copy PR description to clipboard")?;
        println!("Copied PR description to clipboard.");
    }

    if args.create {
        // Split title (first line) and body
        let lines: Vec<&str> = pr_text.lines().collect();
        let title = lines.first().unwrap_or(&"Pull Request").trim_start_matches("# Title:").trim();
        let body = lines.get(1..).map(|l| l.join("\n")).unwrap_or_default();
        let pr_url = ui::gh::create_github_pr(title, &body).context("create GitHub PR")?;
        println!("Created Pull Request: {}", pr_url);
    }

    Ok(())
}

// Implement run_branch_flow
async fn run_branch_flow(args: cli::BranchArgs, config: &Config) -> Result<()> {
    let (provider_name, mut provider) = config
        .get_active_provider(args.provider.as_deref())
        .context("resolve active LLM provider")?;
    if let Some(m) = args.model {
        provider.model = m;
    }

    let diff = git::branch_context::get_working_tree_diff(config.preferences.max_diff_chars).ok();
    let (sys_p, usr_p) = llm::prompt::build_branch_prompt(
        diff.as_ref().map(|d| d.content.as_str()),
        args.prompt.as_deref(),
        args.branch_type.as_deref(),
    );

    let client = LlmClient::new();
    let raw = client.generate_commit(&provider_name, &provider, &sys_p, &usr_p).await.context("generate branch names")?;
    let candidates = llm::prompt::parse_branch_suggestions(&raw);

    if candidates.is_empty() {
        println!("No branch name suggestions could be parsed. Output:\n{}", raw);
        return Ok(());
    }

    if args.checkout {
        let chosen = if candidates.len() == 1 {
            candidates[0].clone()
        } else {
            ui::branch_picker::select_branch_name(&candidates).context("select branch name")?
        };

        git::branch_context::checkout_new_branch(&chosen).context("checkout branch")?;
        println!("Switched to new branch '{}'", chosen);
    } else {
        println!("Suggested branch names:");
        for c in &candidates {
            println!("  git checkout -b {}", c);
        }
    }

    Ok(())
}
```

- [ ] **Step 2: Connect `main()` in `src/main.rs`**

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    if let Some(command) = cli.command {
        let config_path = match &command {
            cli::Commands::Pr(p) => p.config_path.as_deref(),
            cli::Commands::Branch(b) => b.config_path.as_deref(),
            _ => None,
        };
        let config = Config::load(config_path).unwrap_or_default();

        match command {
            cli::Commands::Hook(args) => run_hook_flow(args).await?,
            cli::Commands::Pr(args) => run_pr_flow(args, &config).await?,
            cli::Commands::Branch(args) => run_branch_flow(args, &config).await?,
        }
        return Ok(());
    }

    // Default root commit flow
    run_commit_flow(cli.commit_args).await
}
```

- [ ] **Step 3: Run full compiler and test verification**

Run: `cargo check` and `cargo test`
Expected: PASS with 0 warnings

- [ ] **Step 4: Commit changes**

```bash
git add src/main.rs
git commit -m "feat(main): wire Hook, Pr, and Branch subcommands into coordinator"
```

---

### Task 7: End-to-End Verification, Documentation & Session Notes

**Files:**
- Modify: `README.md`
- Modify: `SESSION.md`

**Interfaces:**
- Verifies full test suite and updates user-facing documentation with new subcommand capabilities.

- [ ] **Step 1: Run comprehensive quality checks**

Run:
```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```
Expected: All pass cleanly.

- [ ] **Step 2: Update `README.md` with CLI subcommands reference**

Add documentation sections for `cogit hook install/uninstall`, `cogit pr`, and `cogit branch`.

- [ ] **Step 3: Commit documentation**

```bash
git add README.md
git commit -m "docs: add subcommands documentation for hook, pr, and branch"
```
