// ==============================================================================
// Terminal User Interface (TUI) Module
// ==============================================================================
//
// Dual-pane terminal interface powered by Ratatui and tui-textarea.

pub mod app;
pub mod events;
pub mod ui;

pub use app::{AppStatus, Focus, TuiApp};
pub use events::run_tui;
