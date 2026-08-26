# Tasks: Cogit Implementation

## Phase 1: Foundation & Git Bridge
- [x] **Task 1: Configure Cargo Dependencies & Core CLI Arguments**
  - Configure `Cargo.toml` dependencies (`clap`, `tokio`, `reqwest`, `serde`, `ratatui`, `tui-textarea`, etc.)
  - Implement `src/cli.rs` with `Args` struct (`-d`, `--tui`, `--dry-run`, `-m`, `-p`, `--prompt`, `--config`, `--init-config`)
- [x] **Task 2: Git Repository Verification & Staged Status Checks**
  - Implement `is_git_repository()` in `src/git/diff.rs`
  - Implement `has_staged_changes()` in `src/git/diff.rs`
- [x] **Task 3: Staged Diff Extraction, Lockfile Filtering & Truncation**
  - Implement `get_staged_diff()` in `src/git/diff.rs`
  - Strip lockfiles (`Cargo.lock`, `package-lock.json`, `pnpm-lock.yaml`, etc.) and binary hunks
  - Implement character bounds truncation with warnings
- [x] **Task 4: Safe Git Commit Execution**
  - Implement `execute_commit()` in `src/git/commit.rs` using temporary file and `git commit -F`

### Checkpoint 1: Foundation
- [x] `cargo check` and `cargo test git::` pass

---

## Phase 2: Configuration Loader & LLM Integration
- [x] **Task 5: Configuration Management & Environment Variable Expansion**
  - Implement `src/config.rs` (`Config`, `ProviderConfig`, `PreferencesConfig`)
  - Implement `${ENV_VAR}` string interpolation
  - Implement `--init-config` file generation
- [x] **Task 6: Conventional Commit Prompt Construction**
  - Implement `src/llm/prompt.rs` with standard and detailed prompt builders
- [x] **Task 7: LLM Response Cleaner & Markdown Stripper**
  - Implement `src/llm/cleaner.rs` to strip markdown fences, preambles, and quotes
- [x] **Task 8: Async OpenAI & Gemini HTTP Client**
  - Implement `src/llm/gemini.rs` for native Google Gemini REST API
  - Implement `src/llm/openai.rs` for OpenAI-compatible REST API
  - Implement `src/llm/client.rs` for unified multi-provider dispatch

### Checkpoint 2: LLM & Config
- [x] `cargo test config::` and `cargo test llm::` pass

---

## Phase 3: Interactive CLI Workflow & Editor Fallback
- [x] **Task 9: External Editor Buffer Integration**
  - Implement `src/ui/editor.rs` resolving `$GIT_EDITOR`, `$VISUAL`, `$EDITOR`, or config preference
  - Safe temp file editing and read-back
- [x] **Task 10: Inquire Interactive Menu Loop**
  - Implement `src/ui/cli_prompt.rs` with `inquire::Select` and `inquire::Text`
  - Options: Commit, Edit in $EDITOR, Regenerate (with hint), Cancel
- [x] **Task 11: Main Coordinator & Dry-Run Mode**
  - Wire `--dry-run` flag in `src/main.rs`
  - Connect full CLI loop: Diff -> LLM -> Interactive Prompt -> Safe Commit

### Checkpoint 3: Interactive CLI Flow
- [x] Test `--dry-run` output and editor resolution pass
- [x] Interactive CLI loop functions end-to-end

---

## Phase 4: Full TUI Experience (Ratatui)
- [x] **Task 12: TUI State Machine & Data Models**
  - Implement `src/ui/tui/app.rs` with `TuiApp`, `Focus`, and `AppStatus`
  - Split diff lines and configure `tui_textarea::TextArea`
- [x] **Task 13: Dual-Pane Layout & Syntax Colorized Widgets**
  - Implement `src/ui/tui/ui.rs` rendering dual-pane layout, colorized diff, and footer
  - Render regeneration and help popups
- [x] **Task 14: Crossterm Event Loop & Action Dispatcher**
  - Implement `src/ui/tui/events.rs` with terminal raw mode lifecycle
  - Support focus switching, textarea input, async regeneration, and editor handoff
- [x] **Task 15: Wire `--tui` Flag in Main Coordinator**
  - Connect `--tui` flag in `src/main.rs` to launch Ratatui interface

### Checkpoint 4: TUI Verification
- [x] `cargo test` passes (21 tests)
- [x] Terminal cleanup on exit verified

---

## Phase 5: Verification, Quality Assurance & Documentation
- [x] **Task 16: Test Suite, Clippy & Rustfmt Verification**
  - Run `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`
- [x] **Task 17: Documentation & Session Notes**
  - Updated `README.md` with badges, ADR links, and provider configurations
  - Documented ADR-001 through ADR-004

### Checkpoint 5: Project Complete
- [x] All tests passing, 0 linter warnings, clean formatting

---

## Phase 6: Git Hook & Workflow Automation Suite
- [ ] **Task 18: CLI Hierarchy & Subcommand Parser**
  - Implement `Cli`, `Commands`, `HookArgs`, `PrArgs`, `BranchArgs` in `src/cli.rs`
  - Maintain root `Args` backward compatibility
- [ ] **Task 19: Git Hook Bridge & File Installer**
  - Implement `src/git/hooks.rs` (`install`, `uninstall`, `is_installed`)
- [ ] **Task 20: Git Branch & History Inspection Bridge**
  - Implement `src/git/branch_context.rs` (`get_default_base_branch`, `get_branch_commits`, `get_branch_diff`, `checkout_new_branch`)
- [ ] **Task 21: LLM Prompt Engineering for PR and Branch Generation**
  - Implement `build_pr_prompt`, `build_branch_prompt`, and `parse_branch_suggestions` in `src/llm/prompt.rs`
- [ ] **Task 22: UI Helpers — Clipboard Copy, GitHub CLI, and Branch Picker**
  - Implement `src/ui/clipboard.rs`, `src/ui/gh.rs`, and `src/ui/branch_picker.rs`
- [ ] **Task 23: Subcommand Handlers & Main Coordinator Routing**
  - Implement `run_hook_flow`, `run_pr_flow`, `run_branch_flow` in `src/main.rs`
- [ ] **Task 24: End-to-End Verification, Documentation & Session Notes**
  - Verify test suite, clippy, update `README.md`
