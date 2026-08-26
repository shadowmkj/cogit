// ==============================================================================
// User Interface Module
// ==============================================================================
//
// Exports the CLI interactive workflow, external editor buffer launcher,
// and the full-featured Ratatui dual-pane TUI experience.

pub mod branch_picker;
pub mod cli_prompt;
pub mod clipboard;
pub mod editor;
pub mod gh;
pub mod tui;

pub use branch_picker::select_branch_name;
pub use cli_prompt::{UserAction, UserOptions, run_cli_prompt};
pub use clipboard::copy_to_clipboard;
pub use editor::edit_message_in_editor;
pub use gh::create_github_pr;
pub use tui::{AppStatus, TuiApp, run_tui};
