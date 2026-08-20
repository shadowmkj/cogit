// ==============================================================================
// Interactive CLI Prompt Workflow
// ==============================================================================
//
// Lightweight terminal menu using `inquire` that allows users to quickly review,
// commit, edit in $EDITOR, regenerate with extra context, or cancel.

use crate::ui::editor::edit_message_in_editor;
use anyhow::{Context, Result};
use inquire::{Select, Text};
use std::fmt;

/// User actions returned from the interactive CLI prompt.
pub enum UserAction {
    /// Commit the current message to Git.
    Commit(String),
    /// Request LLM regeneration with optional user guidance.
    Regenerate(Option<String>),
    /// Abort the operation without making changes.
    Cancel,
}

/// Menu options available to the user in the interactive prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserOptions {
    Commit,
    Edit,
    Regenerate,
    Cancel,
}

impl fmt::Display for UserOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Commit => write!(f, "Commit (apply message)"),
            Self::Edit => write!(f, "Edit in $EDITOR"),
            Self::Regenerate => write!(f, "Regenerate (with optional hint)"),
            Self::Cancel => write!(f, "Cancel"),
        }
    }
}

/// Runs the interactive review loop until the user chooses to commit, regenerate, or cancel.
pub fn run_cli_prompt(mut message: String, configured_editor: Option<&str>) -> Result<UserAction> {
    loop {
        println!("\nProposed Conventional Commit Message:");
        println!("----------------------------------------");
        println!("{}", message);
        println!("----------------------------------------\n");

        let options = vec![
            UserOptions::Commit,
            UserOptions::Edit,
            UserOptions::Regenerate,
            UserOptions::Cancel,
        ];

        let selection = Select::new("What would you like to do?", options)
            .prompt()
            .context("prompt user for action")?;

        match selection {
            UserOptions::Commit => {
                return Ok(UserAction::Commit(message));
            }
            UserOptions::Edit => match edit_message_in_editor(&message, configured_editor) {
                Ok(edited) => {
                    if !edited.trim().is_empty() {
                        message = edited;
                    } else {
                        println!("Warning: Edited message was empty, keeping previous message.");
                    }
                }
                Err(err) => {
                    eprintln!("Error launching editor: {}", err);
                }
            },
            UserOptions::Regenerate => {
                let hint = Text::new("Enter additional guidance / instructions (optional):")
                    .prompt()
                    .context("prompt user for regeneration hint")?;

                let optional_hint = if hint.trim().is_empty() {
                    None
                } else {
                    Some(hint)
                };

                return Ok(UserAction::Regenerate(optional_hint));
            }
            UserOptions::Cancel => {
                return Ok(UserAction::Cancel);
            }
        }
    }
}
