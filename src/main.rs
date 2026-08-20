// ==============================================================================
// Cogit: AI-Powered Conventional Commit CLI & TUI
// ==============================================================================
//
// Phase 2 Entrypoint: Multi-Provider LLM Integration & Config Management

pub mod cli;
pub mod config;
pub mod git;
pub mod llm;

use anyhow::{Context, Result};
use clap::Parser;
use cli::Args;
use config::Config;
use llm::{LlmClient, build_prompt};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Handle --init-config flag
    if args.init_config {
        let path = Config::initialize_default_file(args.config_path.as_deref())
            .context("initialize default config.toml file")?;
        println!("Initialized default configuration at {}", path.display());
        return Ok(());
    }

    // Load configuration from ~/.config/cogit/config.toml or custom path
    let config = Config::load(args.config_path.as_deref()).context("load configuration file")?;

    // Resolve active provider and preferences
    let (provider_name, mut provider) = config
        .get_active_provider(args.provider.as_deref())
        .context("resolve active LLM provider")?;

    if let Some(model_override) = args.model {
        provider.model = model_override;
    }

    let detailed = args.detailed || config.preferences.detailed;

    // Verify Git repository state
    if !git::is_git_repository().context("verify current directory is a git repository")? {
        eprintln!("Error: Not inside a Git repository. Please navigate to a Git project.");
        std::process::exit(1);
    }

    if !git::has_staged_changes().context("check for staged changes in repository")? {
        eprintln!("No staged changes found. Use 'git add <files>' to stage your changes first.");
        std::process::exit(0);
    }

    // Extract and sanitize staged Git diff
    let staged_diff = git::get_staged_diff(config.preferences.max_diff_chars)
        .context("extract staged git diff")?;

    if staged_diff.is_truncated {
        eprintln!(
            "Notice: Staged diff exceeded maximum character limit ({} chars) and was truncated.",
            config.preferences.max_diff_chars
        );
    }

    // Initialize LLM client and build prompt
    let llm_client = LlmClient::new();
    let (system_prompt, user_prompt) = build_prompt(
        &staged_diff.content,
        &staged_diff.staged_files,
        detailed,
        args.custom_prompt.as_deref(),
    );

    println!(
        "Generating commit message using provider [{}] with model '{}'...",
        provider_name, provider.model
    );

    let generated_message = llm_client
        .generate_commit(&provider_name, &provider, &system_prompt, &user_prompt)
        .await
        .context("generate commit message from LLM provider")?;

    if args.dry_run {
        println!("{}", generated_message);
        return Ok(());
    }

    println!("\nGenerated Commit Message:\n");
    println!("----------------------------------------");
    println!("{}", generated_message);
    println!("----------------------------------------");

    Ok(())
}
