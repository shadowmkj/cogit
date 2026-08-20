// ==============================================================================
// TUI Application State Model
// ==============================================================================
//
// Encapsulates the visual state machine, scroll positions, focus management,
// and editable text buffers for the dual-pane Ratatui interface.

use crate::git::StagedDiff;
use ratatui_textarea::TextArea;

/// Tracks which visual pane or overlay currently receives keyboard input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    /// Left pane: scrollable colorized git diff.
    Diff,
    /// Right pane: editable commit message textarea.
    Editor,
    /// Modal overlay: prompt for regeneration guidance.
    RegenPopup,
    /// Modal overlay: keyboard shortcuts reference.
    HelpPopup,
}

/// Lifecycle status of the TUI session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppStatus {
    /// Actively interacting and editing.
    Editing,
    /// Asynchronously waiting for LLM generation/regeneration.
    Regenerating,
    /// User confirmed commit with final message payload.
    Committed(String),
    /// User exited without committing.
    Cancelled,
}

/// Root state struct for the interactive Terminal User Interface.
pub struct TuiApp<'a> {
    /// Staged diff inspection data.
    pub staged_diff: StagedDiff,
    /// Active provider display name (e.g. "gemini", "openai").
    pub provider_name: String,
    /// Active model name (e.g. "gemini-2.5-flash").
    pub model_name: String,
    /// Pre-split diff lines for fast scroll rendering.
    pub diff_lines: Vec<String>,
    /// Vertical scroll offset in lines for the diff pane.
    pub diff_scroll: usize,
    /// Live editable commit message buffer.
    pub textarea: TextArea<'a>,
    /// Currently focused pane or popup.
    pub focus: Focus,
    /// Current execution status.
    pub status: AppStatus,
    /// Buffer for user input in the regeneration feedback popup.
    pub regen_input: String,
    /// Optional transient error message to display in the footer.
    pub error_message: Option<String>,
}

impl<'a> TuiApp<'a> {
    /// Initializes a new TUI state instance with initial diff and generated commit message.
    pub fn new(
        staged_diff: StagedDiff,
        provider_name: String,
        model_name: String,
        initial_commit_message: &str,
    ) -> Self {
        let diff_lines: Vec<String> = staged_diff.content.lines().map(String::from).collect();

        let lines: Vec<String> = initial_commit_message.lines().map(String::from).collect();

        let mut textarea = TextArea::new(lines);
        textarea.set_cursor_line_style(ratatui::style::Style::default());

        Self {
            staged_diff,
            provider_name,
            model_name,
            diff_lines,
            diff_scroll: 0,
            textarea,
            focus: Focus::Editor,
            status: AppStatus::Editing,
            regen_input: String::new(),
            error_message: None,
        }
    }

    /// Extracts the full multi-line commit message currently in the textarea.
    pub fn get_commit_message(&self) -> String {
        self.textarea.lines().join("\n").trim().to_string()
    }

    /// Replaces the current textarea content with a newly generated message.
    pub fn set_commit_message(&mut self, message: &str) {
        let lines: Vec<String> = message.lines().map(String::from).collect();
        self.textarea = TextArea::new(lines);
        self.textarea
            .set_cursor_line_style(ratatui::style::Style::default());
    }

    /// Scrolls the diff view upward by the requested number of lines.
    pub fn scroll_diff_up(&mut self, delta: usize) {
        self.diff_scroll = self.diff_scroll.saturating_sub(delta);
    }

    /// Scrolls the diff view downward bounded by the total line count.
    pub fn scroll_diff_down(&mut self, delta: usize) {
        let max_scroll = self.diff_lines.len().saturating_sub(1);
        self.diff_scroll = (self.diff_scroll + delta).min(max_scroll);
    }

    /// Toggles focus between the diff viewer and the commit message editor.
    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Diff => Focus::Editor,
            Focus::Editor => Focus::Diff,
            Focus::RegenPopup => Focus::RegenPopup,
            Focus::HelpPopup => Focus::HelpPopup,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_diff() -> StagedDiff {
        StagedDiff {
            content: "line1\nline2\nline3\nline4\nline5".to_string(),
            staged_files: vec!["src/main.rs".to_string()],
            omitted_files: vec![],
            is_truncated: false,
        }
    }

    #[test]
    fn test_tui_app_initialization() {
        let diff = create_test_diff();
        let app = TuiApp::new(
            diff,
            "gemini".to_string(),
            "gemini-2.5-flash".to_string(),
            "feat: initial commit",
        );

        assert_eq!(app.provider_name, "gemini");
        assert_eq!(app.model_name, "gemini-2.5-flash");
        assert_eq!(app.diff_lines.len(), 5);
        assert_eq!(app.get_commit_message(), "feat: initial commit");
        assert_eq!(app.focus, Focus::Editor);
    }

    #[test]
    fn test_diff_scrolling_bounds() {
        let diff = create_test_diff();
        let mut app = TuiApp::new(
            diff,
            "gemini".to_string(),
            "gemini-2.5-flash".to_string(),
            "feat: test",
        );

        app.scroll_diff_down(3);
        assert_eq!(app.diff_scroll, 3);

        app.scroll_diff_down(10);
        assert_eq!(app.diff_scroll, 4); // Max line index is 4 (5 lines total)

        app.scroll_diff_up(2);
        assert_eq!(app.diff_scroll, 2);

        app.scroll_diff_up(10);
        assert_eq!(app.diff_scroll, 0);
    }

    #[test]
    fn test_toggle_focus() {
        let diff = create_test_diff();
        let mut app = TuiApp::new(
            diff,
            "gemini".to_string(),
            "gemini-2.5-flash".to_string(),
            "feat: test",
        );

        assert_eq!(app.focus, Focus::Editor);
        app.toggle_focus();
        assert_eq!(app.focus, Focus::Diff);
        app.toggle_focus();
        assert_eq!(app.focus, Focus::Editor);
    }
}
