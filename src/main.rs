// ==============================================================================
// Cogit: AI-Powered Conventional Commit CLI & TUI
// ==============================================================================
//
// Application entry point and subcommand dispatcher.

pub mod cli;
pub mod config;
pub mod git;
pub mod llm;
pub mod ui;
pub mod workflows;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use workflows::{run_branch_flow, run_commit_flow, run_hook_flow, run_pr_flow};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        match command {
            Commands::Hook(args) => run_hook_flow(args).await?,
            Commands::Pr(args) => run_pr_flow(args).await?,
            Commands::Branch(args) => run_branch_flow(args).await?,
        }
        return Ok(());
    }

    // Default root Conventional Commit generation flow
    run_commit_flow(cli.commit_args).await
}
