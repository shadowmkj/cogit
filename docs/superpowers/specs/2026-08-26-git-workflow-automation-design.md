# Design Specification: Git Hook & Workflow Automation Suite

**Date:** 2026-08-26  
**Status:** Approved  
**Topic:** Git Hook (`prepare-commit-msg`), PR Description Generator (`cogit pr`), and Branch Name Generator (`cogit branch`)

---

## 1. Overview & Objectives

Cogit is an AI-powered developer assistant for Git. This feature suite expands Cogit from a single commit generator into a comprehensive Git workflow companion with three primary capabilities:

1. **Git Hook Integration (`cogit hook`)**:
   - `cogit hook install`: Installs `.git/hooks/prepare-commit-msg` in the active repository.
   - `cogit hook uninstall`: Safely removes or unlinks the hook.
   - `cogit hook prepare-commit-msg <FILE> [SOURCE] [SHA]`: Internal callback that auto-populates `.git/COMMIT_EDITMSG` when running `git commit` without `-m`.
2. **Pull Request Description Generator (`cogit pr`)**:
   - Analyzes commits (`git log <base>..HEAD`) and full diff (`git diff <base>...HEAD`) against base branch (e.g. `main` or `master`).
   - Generates structured PR title and Markdown description following industry best practices (Summary, Changes, Testing, Checklist).
   - Supports `--copy` (clipboard) and `--create` (delegates to GitHub CLI `gh pr create`).
3. **Branch Name Generator (`cogit branch`)**:
   - Derives context from working tree diffs (`git diff HEAD`) or an optional prompt description.
   - Generates 3-5 clean, kebab-case branch names with standard prefixes (`feat/`, `fix/`, `chore/`, `refactor/`).
   - Offers optional auto-checkout via `-c` / `--checkout`.

---

## 2. CLI Architecture & Backwards Compatibility

To maintain 100% backward compatibility with existing `cogit` invocations, the root command retains all existing commit flags while allowing subcommands.

```
cogit [FLAGS]                      -> Default Conventional Commit flow
├── hook                           -> Git hook lifecycle
│   ├── install                    -> Installs .git/hooks/prepare-commit-msg
│   ├── uninstall                  -> Removes .git/hooks/prepare-commit-msg
│   └── prepare-commit-msg <ARGS>  -> Hidden callback executed by Git
├── pr [OPTIONS]                   -> Pull request generator
└── branch [OPTIONS] [PROMPT]      -> Branch name generator
```

---

## 3. Component Design

### 3.1 CLI Layer (`src/cli.rs`)
- `Cli` top-level struct containing `command: Option<Commands>` and flattened `commit_args: Args`.
- `Commands`:
  - `Hook(HookArgs)` with `HookAction::Install`, `HookAction::Uninstall`, and `HookAction::PrepareCommitMsg`.
  - `Pr(PrArgs)` with `--base`, `--copy`, `--create`, `--prompt`, `--model`, `--provider`, `--config`.
  - `Branch(BranchArgs)` with `prompt`, `-c/--checkout`, `-t/--type`, `--model`, `--provider`, `--config`.

### 3.2 Git Bridge Layer (`src/git/`)
- `src/git/hooks.rs`:
  - `install_prepare_commit_msg_hook() -> Result<PathBuf>`: Writes executable shell script `.git/hooks/prepare-commit-msg`.
  - `uninstall_prepare_commit_msg_hook() -> Result<()>`: Removes `.git/hooks/prepare-commit-msg`.
  - `is_hook_installed() -> Result<bool>`.
- `src/git/branch_context.rs`:
  - `get_default_base_branch() -> Result<String>`: Detects `main`, `master`, or tracking remote.
  - `get_branch_diff(base: &str, max_chars: usize) -> Result<StagedDiff>`: Runs `git diff <base>...HEAD`.
  - `get_branch_commits(base: &str) -> Result<Vec<String>>`: Runs `git log <base>..HEAD --oneline`.
  - `get_working_tree_diff(max_chars: usize) -> Result<StagedDiff>`: Runs `git diff HEAD`.
  - `checkout_new_branch(name: &str) -> Result<()>`: Executes `git checkout -b <name>`.

### 3.3 LLM Prompt Engineering (`src/llm/prompt.rs`)
- `build_pr_prompt(commits: &[String], diff: &str, custom_prompt: Option<&str>) -> (String, String)`:
  - Generates markdown formatted PR description with Title, Summary, Details, and Testing Checklist.
- `build_branch_prompt(diff: Option<&str>, user_hint: Option<&str>, branch_type: Option<&str>) -> (String, String)`:
  - Instructs LLM to generate 3-5 clean conventional branch names (one per line).

### 3.4 Workflow & UI Handlers (`src/ui/`)
- `src/ui/clipboard.rs`:
  - `copy_to_clipboard(text: &str) -> Result<()>`: Cross-platform clipboard copy using OS clipboard tools (`pbcopy`, `wl-copy`/`xclip`, `clip.exe`).
- `src/ui/gh.rs`:
  - `create_github_pr(title: &str, body: &str) -> Result<String>`: Invokes `gh pr create --title <title> --body-file <tempfile>`.
- `src/ui/branch_picker.rs`:
  - `select_branch_name(candidates: &[String]) -> Result<String>`: Interactive selection menu via `inquire::Select`.

---

## 4. Error Handling & Edge Cases
- **Hook execution**: If `source` is passed (`merge`, `message`, `squash`, `commit`), exit cleanly with code 0 without overwriting.
- **Empty diffs**: `cogit pr` and `cogit hook` cleanly abort if no diffs/commits are detected.
- **Missing `gh` CLI**: When `--create` is requested but `gh` is missing, display clear installation instructions.
- **Clipboard fallbacks**: If clipboard copy fails on headless servers, print content to stdout and display a notice.
