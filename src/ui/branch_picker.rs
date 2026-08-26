// ==============================================================================
// Interactive Branch Selection Menu
// ==============================================================================
//
// Prompts the user to pick one from a list of generated branch name candidates.

use anyhow::{Context, Result, bail};
use inquire::Select;

/// Prompts the user to select from a list of generated branch names.
pub fn select_branch_name(candidates: &[String]) -> Result<String> {
    if candidates.is_empty() {
        bail!("No valid branch name candidates were generated.");
    }

    let selection = Select::new("Select a branch name to checkout:", candidates.to_vec())
        .prompt()
        .context("prompt user for branch selection")?;

    Ok(selection)
}
