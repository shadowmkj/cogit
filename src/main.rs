pub mod cli;
pub mod git;

use anyhow::{Context, Result};
use clap::Parser;
use cli::Args;

fn main() -> Result<()> {
    let args = Args::parse();

    if !git::is_git_repository().context("verify current directory is a git repository")? {
        eprintln!("Error: Not inside a Git repository. Please navigate to a Git project.");
        std::process::exit(1);
    }

    if !git::has_staged_changes().context("check for staged changes in repository")? {
        eprintln!("No staged changes found. Use 'git add <files>' to stage your changes first.");
        std::process::exit(0);
    }

    let staged_diff = git::get_staged_diff(32_000).context("extract staged git diff")?;

    println!("Staged Files ({}):", staged_diff.staged_files.len());
    for file in &staged_diff.staged_files {
        println!("  - {}", file);
    }

    if !staged_diff.omitted_files.is_empty() {
        println!("\nOmitted Files ({}):", staged_diff.omitted_files.len());
        for file in &staged_diff.omitted_files {
            println!("  - {}", file);
        }
    }

    if staged_diff.is_truncated {
        println!("\nNotice: Staged diff exceeded maximum character limit and was truncated.");
    }

    if args.dry_run {
        println!(
            "\n[Dry Run] Staged diff extracted successfully ({} characters).",
            staged_diff.content.len()
        );
    }

    Ok(())
}
