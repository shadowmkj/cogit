// ==============================================================================
// Conventional Commit Workflow Orchestrator
// ==============================================================================
//
// Coordinates repository validation, diff extraction, prompt construction,
// LLM communication, user interaction (CLI loop, external editor, TUI),
// and final Git commit execution.

use crate::cli::Args;
use crate::config::Config;
use crate::git;
use crate::llm::{LlmClient, build_prompt};
use crate::ui::edit_message_in_editor;
use crate::ui::tui::AppStatus;
use crate::ui::{UserAction, run_cli_prompt, run_tui};
use anyhow::{Context, Result};

/// Executes the core interactive Conventional Commit generation flow.
pub async fn run_commit_flow(args: Args) -> Result<()> {
    // 1. Handle --init-config flag
    if args.init_config {
        let path = Config::initialize_default_file(args.config_path.as_deref())
            .context("initialize default config.toml file")?;
        println!("Initialized default configuration at {}", path.display());
        return Ok(());
    }

    // 2. Load configuration and resolve active provider
    let config = Config::load(args.config_path.as_deref()).context("load configuration file")?;

    let (provider_name, mut provider) = config
        .get_active_provider(args.provider.as_deref())
        .context("resolve active LLM provider")?;

    if let Some(model_override) = args.model {
        provider.model = model_override;
    }

    let detailed = args.detailed || config.preferences.detailed;
    let editor_pref = config.preferences.editor.as_deref();

    // 3. Verify Git repository state and staged changes
    if !git::is_git_repository().context("verify current directory is a git repository")? {
        eprintln!("Error: Not inside a Git repository. Please navigate to a Git project.");
        std::process::exit(1);
    }

    if !git::has_staged_changes().context("check for staged changes in repository")? {
        eprintln!("No staged changes found. Use 'git add <files>' to stage your changes first.");
        std::process::exit(0);
    }

    // 4. Extract and sanitize staged Git diff
    let staged_diff = git::get_staged_diff(config.preferences.max_diff_chars)
        .context("extract staged git diff")?;

    if staged_diff.is_truncated {
        eprintln!(
            "Notice: Staged diff exceeded maximum character limit ({} chars) and was truncated.",
            config.preferences.max_diff_chars
        );
    }

    // 5. Generate initial commit message via LLM
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

    // 6. Handle dry-run mode
    if args.dry_run {
        println!("{}", initial_message);
        return Ok(());
    }

    // 7. Handle full TUI mode
    if args.tui {
        let tui_status = run_tui(
            staged_diff,
            provider_name,
            provider,
            initial_message,
            detailed,
            editor_pref,
        )
        .await
        .context("run TUI session")?;

        match tui_status {
            AppStatus::Committed(msg) => {
                let output =
                    git::execute_commit(&msg).context("commit changes to Git repository")?;
                println!("{}", output);
            }
            AppStatus::Cancelled => {
                println!("Commit cancelled.");
            }
            _ => {}
        }

        return Ok(());
    }

    // 8. Handle direct external editor mode
    if args.edit {
        let edited = edit_message_in_editor(&initial_message, editor_pref)
            .context("launch external editor")?;
        if !edited.trim().is_empty() {
            let output =
                git::execute_commit(&edited).context("commit changes to Git repository")?;
            println!("{}", output);
            return Ok(());
        } else {
            eprintln!("Commit message was empty. Commit cancelled.");
            return Ok(());
        }
    }

    // 9. Interactive CLI prompt loop
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
