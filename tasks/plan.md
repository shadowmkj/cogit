# Implementation Plan: Cogit (AI-Powered Conventional Commit Assistant)

## Overview
A fast, lightweight CLI and TUI tool in Rust (2024 edition) that inspects staged Git diffs, prompts a Large Language Model (local Ollama or cloud OpenAI-compatible APIs) to generate Conventional Commits messages, and provides interactive review, editing, regeneration, and safe committing.

## Architecture Decisions
- **CLI Framework**: `clap` with `derive` and `env` for declarative flag and subcommand handling (`-d`, `--tui`, `--dry-run`, `-m`, `-p`, `--prompt`, `--config`, `--init-config`).
- **Configuration Management**: `~/.config/cogit/config.toml` using `serde`, `toml`, and `directories` with environment variable interpolation (`${OPENAI_API_KEY}`) and sensible fallback defaults for Ollama (`http://localhost:11434/v1`) and OpenAI.
- **Git Bridge**: Direct invocation of `git` via `std::process::Command` ensuring pre-commit hooks and GPG signing are preserved. Commits are executed using `git commit -F <tempfile>`.
- **Diff Sanitization**: Strips lockfiles (`Cargo.lock`, `package-lock.json`, `pnpm-lock.yaml`, `go.sum`, etc.) and binary files. Bounds character/token limits with explicit truncation notifications.
- **LLM Client & Prompting**: OpenAI-compatible `/v1/chat/completions` async HTTP client (`reqwest` with rustls). Generates standard (single line <=72 chars, imperative mood) or detailed (subject + bulleted body) conventional commits.
- **Output Cleaner**: Strips markdown code blocks (` ``` `), conversational introductions, and surrounding quotes.
- **Interactive CLI & TUI**:
  - Inquire menu loop (`[y] Commit`, `[e] Edit in $EDITOR`, `[r] Regenerate`, `[q] Cancel`).
  - External editor buffer integration ($`GIT_EDITOR`, `$VISUAL`, `$EDITOR`, `nano`/`vim`).
  - Full Ratatui TUI with dual-pane layout (colorized scrollable diff viewer + live editable `tui-textarea`).

---

## Detailed Task Breakdown

### Phase 1: Foundation & Git Bridge (CLI Skeleton)

#### Task 1: Configure Cargo Dependencies & Core CLI Arguments
- **Description:** Define dependencies in `Cargo.toml` and implement declarative CLI flags in `src/cli.rs`.
- **Acceptance criteria:**
  - [ ] `Cargo.toml` includes `clap`, `tokio`, `reqwest`, `serde`, `serde_json`, `toml`, `directories`, `inquire`, `crossterm`, `ratatui`, `tui-textarea`, `anyhow`, `tempfile`, and `shlex`.
  - [ ] `Args` struct parses `-d`/`--detailed`, `--tui`, `--dry-run`, `-m`/`--model`, `-p`/`--provider`, `--prompt`, `--config`, and `--init-config`.
- **Verification:**
  - `cargo check` compiles without errors.
  - `cargo run -- --help` prints usage instructions and all options.
- **Dependencies:** None
- **Files touched:** `Cargo.toml`, `src/cli.rs`
- **Estimated scope:** S (2 files)

#### Task 2: Git Repository Verification & Staged Status Checks
- **Description:** Implement repository state checks to ensure the working directory is a Git repository and has staged changes.
- **Acceptance criteria:**
  - [ ] `is_git_repository()` validates `git rev-parse --is-inside-work-tree`.
  - [ ] `has_staged_changes()` checks `git diff --staged --quiet`.
  - [ ] Informative error messages displayed if not in a repository or if staged changes are missing.
- **Verification:**
  - Unit tests for repository detection.
  - `cargo test --lib git`
- **Dependencies:** Task 1
- **Files touched:** `src/git/mod.rs`, `src/git/diff.rs`
- **Estimated scope:** S (2 files)

#### Task 3: Staged Diff Extraction, Lockfile Filtering & Truncation
- **Description:** Extract `git diff --staged`, sanitize by omitting lockfiles and binary files, and enforce max character bounds with truncation notices.
- **Acceptance criteria:**
  - [ ] `get_staged_diff()` extracts staged diff and lists staged files.
  - [ ] Lockfiles (`Cargo.lock`, `package-lock.json`, etc.) and binary hunks are excluded from prompt content.
  - [ ] Oversized diffs are truncated at newline boundaries with an appended warning note.
- **Verification:**
  - Unit tests verifying lockfile filtering and truncation limits.
  - `cargo test git::diff::tests`
- **Dependencies:** Task 2
- **Files touched:** `src/git/diff.rs`
- **Estimated scope:** M (1 file + unit tests)

#### Task 4: Safe Git Commit Execution
- **Description:** Implement safe commit execution using `git commit -F` with a temporary file to support multi-line messages and preserve hooks and GPG signing.
- **Acceptance criteria:**
  - [ ] `execute_commit()` writes message to a secure temporary file and invokes `git commit -F`.
  - [ ] Captures stdout/stderr and returns clear error messages on failure.
- **Verification:**
  - Unit/integration test for commit command formatting.
  - `cargo test git::commit`
- **Dependencies:** Task 1
- **Files touched:** `src/git/commit.rs`, `src/git/mod.rs`
- **Estimated scope:** S (2 files)

---

### Checkpoint 1: Foundation & Git Operations
- [ ] Dependencies configured and building cleanly
- [ ] Git repository checks and staged diff extraction verified with unit tests
- [ ] Safe commit execution logic ready

---

### Phase 2: Configuration Loader & LLM Integration

#### Task 5: Configuration Management & Environment Variable Expansion
- **Description:** Load and parse user configuration from `~/.config/cogit/config.toml`, support `${VAR}` expansion, and provide `--init-config`.
- **Acceptance criteria:**
  - [ ] `Config` struct parses `default_provider`, `providers`, and `preferences`.
  - [ ] `${ENV_VAR}` substrings in API keys and URLs are replaced with actual environment variables.
  - [ ] Fallback defaults for Ollama and OpenAI when config file is absent.
  - [ ] `--init-config` writes a documented template to the config path.
- **Verification:**
  - Unit tests for TOML parsing and `${ENV}` substitution.
  - `cargo test config::tests`
- **Dependencies:** Task 1
- **Files touched:** `src/config.rs`
- **Estimated scope:** M (1 file + unit tests)

#### Task 6: Conventional Commit Prompt Construction
- **Description:** Formulate system and user prompts enforcing Conventional Commits specification for standard and detailed modes.
- **Acceptance criteria:**
  - [ ] System prompt specifies types (`feat`, `fix`, `refactor`, `chore`, etc.), imperative mood, and <=72 character limit.
  - [ ] Detailed mode prompts for subject line + blank line + bullet-pointed body.
  - [ ] User prompt includes staged file list, custom guidance, and sanitized diff.
- **Verification:**
  - Unit tests verifying prompt format differences and custom guidance inclusion.
  - `cargo test llm::prompt::tests`
- **Dependencies:** Task 3
- **Files touched:** `src/llm/prompt.rs`, `src/llm/mod.rs`
- **Estimated scope:** S (2 files)

#### Task 7: LLM Response Cleaner & Markdown Stripper
- **Description:** Sanitize raw LLM outputs to remove code fences, preambles, and enclosing quotes.
- **Acceptance criteria:**
  - [ ] Strips markdown fences (```` ``` ````, ```` ```git ````).
  - [ ] Removes introductory conversational phrases ("Here is the commit message:").
  - [ ] Strips surrounding quotes.
- **Verification:**
  - Unit tests covering various malformed and fenced responses.
  - `cargo test llm::cleaner::tests`
- **Dependencies:** None
- **Files touched:** `src/llm/cleaner.rs`
- **Estimated scope:** S (1 file + unit tests)

#### Task 8: Async OpenAI & Ollama HTTP Client
- **Description:** Build an async HTTP client for OpenAI-compatible `/v1/chat/completions` REST endpoints.
- **Acceptance criteria:**
  - [ ] `LlmClient` dispatches async POST requests with Bearer auth and JSON payload.
  - [ ] Normalizes base URLs into `/chat/completions` endpoint paths.
  - [ ] Returns sanitized commit message or informative error on failure.
- **Verification:**
  - Unit tests for endpoint formatting.
  - `cargo test llm::client::tests`
- **Dependencies:** Tasks 5, 6, 7
- **Files touched:** `src/llm/client.rs`
- **Estimated scope:** S (1 file + unit tests)

---

### Checkpoint 2: Configuration & LLM Client
- [ ] Config loading and `${ENV}` expansion tested
- [ ] Prompt construction and response cleaner unit tests passing
- [ ] Async HTTP client ready for OpenAI and Ollama endpoints

---

### Phase 3: Interactive CLI Workflow & Editor Fallback

#### Task 9: External Editor Buffer Integration
- **Description:** Spawn the system editor (`$GIT_EDITOR`, `$VISUAL`, `$EDITOR`, or fallback) on a temporary file to let users manually edit the commit message.
- **Acceptance criteria:**
  - [ ] Resolves editor from config preferences or environment variables.
  - [ ] Spawns editor on temporary file and reads back updated content upon exit.
- **Verification:**
  - Unit test for editor resolution precedence.
  - `cargo test ui::editor::tests`
- **Dependencies:** Task 1
- **Files touched:** `src/ui/editor.rs`, `src/ui/mod.rs`
- **Estimated scope:** S (2 files)

#### Task 10: Inquire Interactive Menu Loop
- **Description:** Implement an interactive terminal menu using `inquire` offering review, commit, edit, regenerate, and cancel options.
- **Acceptance criteria:**
  - [ ] Displays proposed commit message clearly.
  - [ ] Options: Commit, Edit in `$EDITOR`, Regenerate (with guidance prompt), Cancel.
  - [ ] Loops back after editing with the updated message.
- **Verification:**
  - Code compiles without warnings.
- **Dependencies:** Tasks 4, 8, 9
- **Files touched:** `src/ui/cli_prompt.rs`
- **Estimated scope:** S (1 file)

#### Task 11: Wire Main Coordinator & Dry-Run Mode
- **Description:** Orchestrate argument parsing, config loading, Git verification, LLM generation, `--dry-run` output, and interactive prompt loop in `main.rs`.
- **Acceptance criteria:**
  - [ ] `--dry-run` prints generated message to stdout and exits with code 0.
  - [ ] Default CLI mode launches inquire prompt loop and executes commit on confirmation.
  - [ ] `--init-config` initializes default configuration and exits cleanly.
- **Verification:**
  - `cargo test`
  - Manual test with `cargo run -- --help` and `cargo run -- --init-config`.
- **Dependencies:** Tasks 1, 2, 3, 4, 5, 8, 10
- **Files touched:** `src/main.rs`
- **Estimated scope:** M (1 file)

---

### Checkpoint 3: CLI Workflow Complete
- [ ] `--dry-run` outputs commit message to stdout
- [ ] Interactive CLI loop supports Commit, Edit in `$EDITOR`, Regenerate, and Cancel
- [ ] Repository checks and config loading integrated smoothly

---

### Phase 4: Full TUI Experience (Ratatui)

#### Task 12: TUI State Machine & Data Models
- **Description:** Build the `TuiApp` state machine managing focus (`Diff`, `Editor`, `RegenerateInput`), scrolling, status, and `tui-textarea`.
- **Acceptance criteria:**
  - [ ] `TuiApp` holds `staged_diff`, `textarea`, `focus`, `status`, and `diff_scroll`.
  - [ ] Helper methods to get/set commit message, toggle focus, and scroll diff.
- **Verification:**
  - Code compiles cleanly.
- **Dependencies:** Tasks 3, 5
- **Files touched:** `src/ui/tui/app.rs`
- **Estimated scope:** S (1 file)

#### Task 13: Dual-Pane Layout & Diff Colorization View
- **Description:** Render header, responsive dual-pane body (scrollable colorized diff + commit editor), status footer, and modal popups.
- **Acceptance criteria:**
  - [ ] Header shows Cogit version, active model, and staged file count.
  - [ ] Diff pane highlights additions (green), deletions (red), and headers (cyan).
  - [ ] Commit editor pane renders `tui-textarea` with focus indicator.
  - [ ] Popups for regeneration feedback and error reporting.
- **Verification:**
  - Layout renders properly across various terminal widths.
- **Dependencies:** Task 12
- **Files touched:** `src/ui/tui/view.rs`
- **Estimated scope:** M (1 file)

#### Task 14: TUI Keyboard Navigation & Event Loop
- **Description:** Implement keyboard event handling (`Tab` focus switch, `j/k`/arrows diff scrolling, `Enter`/`Ctrl+s` commit, `r` regenerate, `e` external editor, `Esc`/`q` quit), raw mode setup, and async regeneration.
- **Acceptance criteria:**
  - [ ] Keyboard events dispatched according to active focus.
  - [ ] Panic hook guarantees terminal recovery if an unexpected panic occurs.
  - [ ] Async LLM regeneration runs with a loading state and updates editor buffer.
- **Verification:**
  - Keyboard navigation and focus switching respond accurately.
- **Dependencies:** Tasks 8, 9, 12, 13
- **Files touched:** `src/ui/tui/events.rs`, `src/ui/tui/mod.rs`
- **Estimated scope:** M (2 files)

#### Task 15: Integrate TUI Mode in Main Coordinator
- **Description:** Connect `--tui` CLI flag in `src/main.rs` to launch the full TUI interface and commit upon confirmation.
- **Acceptance criteria:**
  - [ ] `cogit --tui` launches Ratatui TUI.
  - [ ] Commits staged changes upon TUI confirmation and restores terminal cleanly.
- **Verification:**
  - `cargo run -- --tui` launches TUI.
- **Dependencies:** Tasks 11, 14
- **Files touched:** `src/main.rs`, `src/ui/mod.rs`
- **Estimated scope:** S (2 files)

---

### Checkpoint 4: Full TUI Experience Complete
- [ ] Full TUI launches with colorized diffs and live text editor
- [ ] Keyboard navigation, regeneration modal, and external editor dispatch working
- [ ] Safe commit execution and clean terminal exit verified

---

### Phase 5: Verification, Quality Assurance & Documentation

#### Task 16: Test Suite, Clippy & Rustfmt Verification
- **Description:** Ensure all automated tests pass, zero compiler warnings, zero clippy warnings, and clean formatting.
- **Acceptance criteria:**
  - [ ] All unit tests pass (`cargo test`).
  - [ ] `cargo clippy --all-targets -- -D warnings` reports 0 warnings.
  - [ ] `cargo fmt --check` passes with no diffs.
- **Verification:**
  - Run full test and linter suite.
- **Dependencies:** Tasks 1–15
- **Files touched:** All source files
- **Estimated scope:** S (whole project check)

#### Task 17: Documentation & Session Notes
- **Description:** Create user documentation and maintain incidental notes.
- **Acceptance criteria:**
  - [ ] `SESSION.md` records deferred tasks and future enhancements.
  - [ ] Usage instructions and configuration options documented.
- **Verification:**
  - Review markdown documents.
- **Dependencies:** Task 16
- **Files touched:** `SESSION.md`, `README.md`
- **Estimated scope:** S (2 files)

---

### Checkpoint 5: Project Complete & Release Ready
- [ ] 100% test pass rate
- [ ] Clean linter and formatter checks
- [ ] Complete documentation and verified CLI/TUI workflows

---

## Risks and Mitigations
| Risk | Impact | Mitigation |
|---|---|---|
| LLM returns markdown code fences or conversational text | High | Implement regex and string stripping in `cleaner.rs` with thorough unit tests. |
| Large diffs overflow LLM token/context limits | Medium | Bound character counts with `max_diff_chars` (default 32k) and filter lockfiles/binaries. |
| Terminal left in raw mode after crash | High | Install panic hook that automatically disables raw mode and restores alternate screen. |
| Shell escaping issues in multi-line commit messages | High | Use temporary files and invoke `git commit -F <file>`. |

## Open Questions
- Default models: `qwen2.5-coder:7b` (Ollama) and `gpt-4o-mini` (OpenAI).
- Staged diff character truncation limit: 32,000 chars.
