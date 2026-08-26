// ==============================================================================
// Pull Request Description Workflow Orchestrator
// ==============================================================================
//
// Inspects commit history and unified diff against a base branch, invokes
// LLM for structured Markdown generation, and provides clipboard & GitHub CLI actions.

use crate::cli::PrArgs;
use crate::config::Config;
use crate::git;
use crate::llm::LlmClient;
use crate::llm::prompt::build_pr_prompt;
use crate::ui;
use anyhow::{Context, Result};

/// Handles `cogit pr` generation comparing branch diff against base branch.
pub async fn run_pr_flow(args: PrArgs) -> Result<()> {
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

    let base_branch = match args.base {
        Some(b) => b,
        None => git::get_default_base_branch().context("detect default base branch")?,
    };

    println!(
        "Inspecting commits and diff against base branch [{}]...",
        base_branch
    );
    let commits =
        git::get_branch_commits(&base_branch).context("retrieve branch commit history")?;
    let branch_diff = git::get_branch_diff(&base_branch, config.preferences.max_diff_chars)
        .context("retrieve branch diff against base")?;

    if branch_diff.is_truncated {
        eprintln!(
            "Notice: Branch diff exceeded maximum character limit ({} chars) and was truncated.",
            config.preferences.max_diff_chars
        );
    }

    let (sys_p, usr_p) = build_pr_prompt(
        &commits,
        &branch_diff.content,
        args.custom_prompt.as_deref(),
    );

    let llm_client = LlmClient::new();
    println!(
        "Generating PR description using provider [{}] with model '{}'...",
        provider_name, provider.model
    );

    let pr_text = llm_client
        .generate_commit(&provider_name, &provider, &sys_p, &usr_p)
        .await
        .context("generate PR description from LLM provider")?;

    println!("\n{}\n", pr_text);

    if args.copy {
        match ui::copy_to_clipboard(&pr_text) {
            Ok(()) => println!("Copied PR description to clipboard."),
            Err(e) => eprintln!("Warning: Failed to copy to clipboard: {}", e),
        }
    }

    if args.create {
        let lines: Vec<&str> = pr_text.lines().collect();
        let title = lines
            .iter()
            .find(|l| l.starts_with("# Title:") || l.starts_with("Title:"))
            .map(|l| {
                l.trim_start_matches("# Title:")
                    .trim_start_matches("Title:")
                    .trim()
            })
            .unwrap_or_else(|| lines.first().unwrap_or(&"Pull Request"));

        let pr_url = ui::create_github_pr(title, &pr_text).context("create GitHub Pull Request")?;
        println!("Successfully created Pull Request: {}", pr_url);
    }

    Ok(())
}
