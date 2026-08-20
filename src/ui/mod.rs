// ==============================================================================
// User Interface Module
// ==============================================================================
//
// Exports the CLI interactive workflow and external editor buffer launcher.

pub mod cli_prompt;
pub mod editor;

pub use cli_prompt::{UserAction, run_cli_prompt};
pub use editor::edit_message_in_editor;
