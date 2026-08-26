// ==============================================================================
// Command Line Interface Definition
// ==============================================================================
//
// Defines the root CLI and subcommands accepted by cogit using clap derive.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Root CLI parser for Cogit.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "cogit",
    author,
    version,
    about = "AI-powered Conventional Commit assistant and Git workflow automation",
    long_about = "Cogit inspects your Git changes, generates compliant Conventional Commits, \
                  manages prepare-commit-msg hooks, and automates PR and branch creation.",
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
    #[arg(
        short = 'c',
        long = "checkout",
        help = "Create and checkout the branch"
    )]
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

/// Arguments for standard commit message generation.
#[derive(Parser, Debug, Clone)]
pub struct Args {
    /// Generate a detailed commit message with a subject and bulleted body.
    #[arg(
        short = 'd',
        long = "detailed",
        help = "Generate a subject and detailed bulleted body"
    )]
    pub detailed: bool,

    /// Launch full Ratatui TUI mode instead of the lightweight CLI prompt.
    #[arg(
        long = "tui",
        help = "Launch full interactive TUI mode",
        conflicts_with_all = ["dry_run", "edit"]
    )]
    pub tui: bool,

    /// Print the generated commit message to stdout without prompting or committing.
    #[arg(
        long = "dry-run",
        help = "Output generated commit message to stdout without committing",
        conflicts_with_all = ["tui", "edit"]
    )]
    pub dry_run: bool,

    /// Override the LLM model (e.g., 'gemini-3.5-flash-lite', 'grok-2-latest', 'llama-3.3-70b-versatile').
    #[arg(short = 'm', long = "model", help = "Override LLM model name")]
    pub model: Option<String>,

    /// Override the active provider name ('gemini', 'openai', 'grok', 'groq', 'ollama').
    #[arg(short = 'p', long = "provider", help = "Override active LLM provider")]
    pub provider: Option<String>,

    /// Open the generated commit message directly in an external editor before committing.
    #[arg(
        short = 'e',
        long = "edit",
        help = "Edit generated commit message in $EDITOR before committing",
        conflicts_with_all = ["tui", "dry_run"]
    )]
    pub edit: bool,

    /// Additional guidance or context instructions for the commit message generation.
    #[arg(
        long = "prompt",
        help = "Additional hint or context for commit generation"
    )]
    pub custom_prompt: Option<String>,

    /// Custom path to the configuration file (defaults to ~/.config/cogit/config.toml).
    #[arg(long = "config", help = "Path to custom configuration file")]
    pub config_path: Option<PathBuf>,

    /// Initialize default configuration file in standard config directory and exit.
    #[arg(
        long = "init-config",
        help = "Create a default config.toml if one does not exist"
    )]
    pub init_config: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_default_args() {
        let cli = Cli::try_parse_from(["cogit"]).expect("default args should parse successfully");
        assert!(cli.command.is_none());
        assert!(!cli.commit_args.edit);
        assert!(!cli.commit_args.tui);
        assert!(!cli.commit_args.dry_run);
        assert!(!cli.commit_args.detailed);
        assert!(cli.commit_args.provider.is_none());
        assert!(cli.commit_args.model.is_none());
    }

    #[test]
    fn test_parse_edit_flag() {
        let cli_short = Cli::try_parse_from(["cogit", "-e"]).expect("-e flag should parse");
        assert!(cli_short.commit_args.edit);

        let cli_long = Cli::try_parse_from(["cogit", "--edit"]).expect("--edit flag should parse");
        assert!(cli_long.commit_args.edit);
    }

    #[test]
    fn test_edit_conflicts_with_tui() {
        let result = Cli::try_parse_from(["cogit", "--edit", "--tui"]);
        assert!(
            result.is_err(),
            "--edit and --tui should be mutually exclusive"
        );
    }

    #[test]
    fn test_edit_conflicts_with_dry_run() {
        let result = Cli::try_parse_from(["cogit", "-e", "--dry-run"]);
        assert!(
            result.is_err(),
            "--edit and --dry-run should be mutually exclusive"
        );
    }

    #[test]
    fn test_parse_options_with_edit() {
        let cli = Cli::try_parse_from([
            "cogit",
            "-e",
            "-d",
            "-p",
            "openai",
            "-m",
            "gpt-4o",
            "--prompt",
            "focus on security",
        ])
        .expect("combined flags should parse");

        assert!(cli.commit_args.edit);
        assert!(cli.commit_args.detailed);
        assert_eq!(cli.commit_args.provider.as_deref(), Some("openai"));
        assert_eq!(cli.commit_args.model.as_deref(), Some("gpt-4o"));
        assert_eq!(
            cli.commit_args.custom_prompt.as_deref(),
            Some("focus on security")
        );
    }

    #[test]
    fn test_parse_subcommand_hook_install() {
        let cli =
            Cli::try_parse_from(["cogit", "hook", "install"]).expect("hook install should parse");
        match cli.command {
            Some(Commands::Hook(hook_args)) => match hook_args.action {
                HookAction::Install => {}
                _ => panic!("expected HookAction::Install"),
            },
            _ => panic!("expected Commands::Hook"),
        }
    }

    #[test]
    fn test_parse_subcommand_hook_prepare_commit_msg() {
        let cli = Cli::try_parse_from([
            "cogit",
            "hook",
            "prepare-commit-msg",
            ".git/COMMIT_EDITMSG",
            "message",
        ])
        .expect("prepare-commit-msg callback should parse");

        match cli.command {
            Some(Commands::Hook(hook_args)) => match hook_args.action {
                HookAction::PrepareCommitMsg { file, source, sha } => {
                    assert_eq!(file, PathBuf::from(".git/COMMIT_EDITMSG"));
                    assert_eq!(source.as_deref(), Some("message"));
                    assert!(sha.is_none());
                }
                _ => panic!("expected HookAction::PrepareCommitMsg"),
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
}
