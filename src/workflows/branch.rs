// ==============================================================================
// Conventional Branch Name Workflow Orchestrator
// ==============================================================================
//
// Extracts uncommitted diff or uses user task description to generate
// clean conventional branch names, offering interactive selection and checkout.

use crate::cli::BranchArgs;
use crate::config::Config;
use crate::git;
use crate::llm::LlmClient;
use crate::llm::prompt::{build_branch_prompt, parse_branch_suggestions};
use crate::ui;
use anyhow::{Context, Result};

/// Handles `cogit branch` conventional branch name suggestion and checkout.
pub async fn run_branch_flow(args: BranchArgs) -> Result<()> {
    if !git::is_git_repository().context("verify current directory is a git repository")? {
        eprintln!("Error: Not inside a Git repository. Please navigate to a Git project.");
        std::process::exit(1);
    }

    let config = Config::load(args.config_path.as_deref()).context("load configuration file")?;
    let (provider_name, mut provider) = config
        .get_active_provider(args.provider.as_deref())
        .context("resolve active LLM provider")?;

    if let Some(model_override) = args.model {
        provider.model = model_override;
    }

    let diff = git::get_working_tree_diff(config.preferences.max_diff_chars).ok();
    let (sys_p, usr_p) = build_branch_prompt(
        diff.as_ref().map(|d| d.content.as_str()),
        args.prompt.as_deref(),
        args.branch_type.as_deref(),
    );

    let llm_client = LlmClient::new();
    println!(
        "Generating branch name suggestions using [{}] with model '{}'...",
        provider_name, provider.model
    );

    let raw = llm_client
        .generate_commit(&provider_name, &provider, &sys_p, &usr_p)
        .await
        .context("generate branch suggestions from LLM provider")?;

    let candidates = parse_branch_suggestions(&raw);

    if candidates.is_empty() {
        println!(
            "No standard branch names could be parsed. LLM Output:\n{}",
            raw
        );
        return Ok(());
    }

    if args.checkout {
        let chosen = if candidates.len() == 1 {
            candidates[0].clone()
        } else {
            ui::select_branch_name(&candidates).context("select branch name")?
        };

        git::checkout_new_branch(&chosen).context("checkout new branch")?;
        println!("Switched to a new branch '{}'", chosen);
    } else {
        println!("\nSuggested branch names:");
        for c in &candidates {
            println!("  git checkout -b {}", c);
        }
        println!();
    }

    Ok(())
}
