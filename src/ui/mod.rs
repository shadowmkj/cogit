// ==============================================================================
// User Interface Module
// ==============================================================================
//
// Exports the CLI interactive workflow, external editor buffer launcher,
// and the full-featured Ratatui dual-pane TUI experience.

pub mod cli_prompt;
pub mod editor;
pub mod tui;

pub use cli_prompt::{UserAction, UserOptions, run_cli_prompt};
pub use editor::edit_message_in_editor;
pub use tui::{AppStatus, TuiApp, run_tui};
