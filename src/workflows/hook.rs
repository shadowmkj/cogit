// ==============================================================================
// Git Hook Workflow Orchestrator
// ==============================================================================
//
// Manages installation, uninstallation, and runtime invocation of the Git
// `prepare-commit-msg` hook to populate commit message drafts automatically.

use crate::cli::{HookAction, HookArgs};
use crate::config::Config;
use crate::git;
use crate::llm::{LlmClient, build_prompt};
use anyhow::{Context, Result};

/// Handles `cogit hook` installation, uninstallation, and internal callback.
pub async fn run_hook_flow(args: HookArgs) -> Result<()> {
    match args.action {
        HookAction::Install => {
            let path = git::install_prepare_commit_msg_hook()
                .context("install prepare-commit-msg hook")?;
            println!("Successfully installed Git hook at {}", path.display());
        }
        HookAction::Uninstall => {
            git::uninstall_prepare_commit_msg_hook()
                .context("uninstall prepare-commit-msg hook")?;
            println!("Successfully uninstalled prepare-commit-msg Git hook.");
        }
        HookAction::PrepareCommitMsg { file, source, .. } => {
            // If message source is specified (e.g. merge, message, squash, commit),
            // skip automatic pre-fill to avoid overwriting user intent or merge conflicts.
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

            let (sys_p, usr_p) = build_prompt(
                &staged_diff.content,
                &staged_diff.staged_files,
                config.preferences.detailed,
                None,
            );

            let llm_client = LlmClient::new();
            if let Ok(msg) = llm_client
                .generate_commit(&provider_name, &provider, &sys_p, &usr_p)
                .await
                && !msg.trim().is_empty()
            {
                let _ = std::fs::write(&file, format!("{}\n", msg.trim()));
            }
        }
    }
    Ok(())
}
