// ==============================================================================
// Cogit: AI-Powered Conventional Commit CLI & TUI
// ==============================================================================
//
// Main Application Coordinator: Drives argument parsing, config loading,
// Git status verification, LLM commit message generation, and interactive workflows.

pub mod cli;
pub mod config;
pub mod git;
pub mod llm;
pub mod ui;

use anyhow::{Context, Result};
use clap::Parser;
use cli::Args;
use config::Config;
use llm::{LlmClient, build_prompt};
use ui::{UserAction, run_cli_prompt};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // 1. Handle --init-config flag
    if args.init_config {
        let path = Config::initialize_default_file(args.config_path.as_deref())
            .context("initialize default config.toml file")?;
        println!("Initialized default configuration at {}", path.display());
        return Ok(());
    }

    // 2. Load configuration from ~/.config/cogit/config.toml or custom path
    let config = Config::load(args.config_path.as_deref()).context("load configuration file")?;

    // 3. Resolve active provider and preferences
    let (provider_name, mut provider) = config
        .get_active_provider(args.provider.as_deref())
        .context("resolve active LLM provider")?;

    if let Some(model_override) = args.model {
        provider.model = model_override;
    }

    let detailed = args.detailed || config.preferences.detailed;
    let editor_pref = config.preferences.editor.as_deref();

    // 4. Verify Git repository state
    if !git::is_git_repository().context("verify current directory is a git repository")? {
        eprintln!("Error: Not inside a Git repository. Please navigate to a Git project.");
        std::process::exit(1);
    }

    if !git::has_staged_changes().context("check for staged changes in repository")? {
        eprintln!("No staged changes found. Use 'git add <files>' to stage your changes first.");
        std::process::exit(0);
    }

    // 5. Extract and sanitize staged Git diff
    let staged_diff = git::get_staged_diff(config.preferences.max_diff_chars)
        .context("extract staged git diff")?;

    if staged_diff.is_truncated {
        eprintln!(
            "Notice: Staged diff exceeded maximum character limit ({} chars) and was truncated.",
            config.preferences.max_diff_chars
        );
    }

    // 6. Initialize LLM client and generate initial commit message
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

    let initial_message = llm_client
        .generate_commit(&provider_name, &provider, &system_prompt, &user_prompt)
        .await
        .context("generate initial commit message from LLM provider")?;

    // 7. Handle Dry-Run Mode
    if args.dry_run {
        println!("{}", initial_message);
        return Ok(());
    }

    // 8. Default: Interactive CLI Review Loop
    let mut current_message = initial_message;
    loop {
        let action = run_cli_prompt(current_message.clone(), editor_pref)
            .context("run interactive CLI prompt")?;

        match action {
            UserAction::Commit(msg) => {
                let output =
                    git::execute_commit(&msg).context("commit changes to Git repository")?;
                println!("{}", output);
                break;
            }
            UserAction::Regenerate(feedback) => {
                println!("Regenerating commit message with feedback...");
                let (sys_p, usr_p) = build_prompt(
                    &staged_diff.content,
                    &staged_diff.staged_files,
                    detailed,
                    feedback.as_deref(),
                );

                current_message = llm_client
                    .generate_commit(&provider_name, &provider, &sys_p, &usr_p)
                    .await
                    .context("regenerate commit message from LLM provider")?;
            }
            UserAction::Cancel => {
                println!("Commit cancelled.");
                break;
            }
        }
    }

    Ok(())
}
