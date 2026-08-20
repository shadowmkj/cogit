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
    long_about = "Cogit inspects your staged Git changes, invokes an LLM (Gemini, OpenAI, Ollama), \
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
    #[arg(long = "tui", help = "Launch full interactive TUI mode")]
    pub tui: bool,

    /// Print the generated commit message to stdout without prompting or committing.
    #[arg(
        long = "dry-run",
        help = "Output generated commit message to stdout without committing"
    )]
    pub dry_run: bool,

    /// Override the LLM model (e.g., 'gemini-2.5-flash', 'gpt-4o-mini', 'qwen2.5-coder:7b').
    #[arg(short = 'm', long = "model", help = "Override LLM model name")]
    pub model: Option<String>,

    /// Override the active provider name ('gemini', 'openai', 'ollama', etc.).
    #[arg(short = 'p', long = "provider", help = "Override active LLM provider")]
    pub provider: Option<String>,

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
