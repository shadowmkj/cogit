// ==============================================================================
// Command Line Interface Definition
// ==============================================================================
//
// Defines all CLI flags and parameters accepted by cogit using clap derive.

use clap::Parser;
use std::path::PathBuf;

/// Fast, lightweight AI-powered Conventional Commit assistant for Git.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "cogit",
    author,
    version,
    about = "Generate conventional commit messages from staged Git diffs using AI",
    long_about = "Cogit inspects your staged Git changes, invokes an LLM (Gemini, OpenAI, Grok, Groq, Ollama), \
                  and offers interactive review, editing ($EDITOR), regeneration, and committing."
)]
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
        let args = Args::try_parse_from(["cogit"]).expect("default args should parse successfully");
        assert!(!args.edit);
        assert!(!args.tui);
        assert!(!args.dry_run);
        assert!(!args.detailed);
        assert!(args.provider.is_none());
        assert!(args.model.is_none());
    }

    #[test]
    fn test_parse_edit_flag() {
        let args_short = Args::try_parse_from(["cogit", "-e"]).expect("-e flag should parse");
        assert!(args_short.edit);

        let args_long =
            Args::try_parse_from(["cogit", "--edit"]).expect("--edit flag should parse");
        assert!(args_long.edit);
    }

    #[test]
    fn test_edit_conflicts_with_tui() {
        let result = Args::try_parse_from(["cogit", "--edit", "--tui"]);
        assert!(
            result.is_err(),
            "--edit and --tui should be mutually exclusive"
        );
    }

    #[test]
    fn test_edit_conflicts_with_dry_run() {
        let result = Args::try_parse_from(["cogit", "-e", "--dry-run"]);
        assert!(
            result.is_err(),
            "--edit and --dry-run should be mutually exclusive"
        );
    }

    #[test]
    fn test_parse_options_with_edit() {
        let args = Args::try_parse_from([
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

        assert!(args.edit);
        assert!(args.detailed);
        assert_eq!(args.provider.as_deref(), Some("openai"));
        assert_eq!(args.model.as_deref(), Some("gpt-4o"));
        assert_eq!(args.custom_prompt.as_deref(), Some("focus on security"));
    }
}
