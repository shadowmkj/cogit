// ==============================================================================
// TUI Event Handling and Execution Loop
// ==============================================================================
//
// Manages crossterm raw mode lifecycle, key event dispatch, async LLM regeneration,
// external editor handoff, and commit actions.

use crate::config::ProviderConfig;
use crate::git::StagedDiff;
use crate::llm::{LlmClient, build_prompt};
use crate::ui::editor::edit_message_in_editor;
use crate::ui::tui::app::{AppStatus, Focus, TuiApp};
use crate::ui::tui::ui::render;
use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::{self, Stdout};
use std::time::Duration;

/// Read-only context passed into TUI event handlers.
struct TuiContext<'a> {
    llm_client: &'a LlmClient,
    provider_name: &'a str,
    provider_config: &'a ProviderConfig,
    detailed: bool,
    configured_editor: Option<&'a str>,
}

/// Runs the full TUI session until commit or cancellation.
pub async fn run_tui(
    staged_diff: StagedDiff,
    provider_name: String,
    provider_config: ProviderConfig,
    initial_commit_message: String,
    detailed: bool,
    configured_editor: Option<&str>,
) -> Result<AppStatus> {
    // 1. Setup terminal in raw mode with alternate screen
    enable_raw_mode().context("enable terminal raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).context("enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("initialize ratatui terminal")?;

    let mut app = TuiApp::new(
        staged_diff,
        provider_name.clone(),
        provider_config.model.clone(),
        &initial_commit_message,
    );

    let llm_client = LlmClient::new();
    let ctx = TuiContext {
        llm_client: &llm_client,
        provider_name: &provider_name,
        provider_config: &provider_config,
        detailed,
        configured_editor,
    };

    // 2. Main event loop
    let result = run_event_loop(&mut terminal, &mut app, &ctx).await;

    // 3. Restore terminal state cleanly
    disable_raw_mode().context("disable terminal raw mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).context("leave alternate screen")?;
    terminal.show_cursor().context("show terminal cursor")?;

    result
}

/// Inner event loop processing redraws and key inputs.
async fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut TuiApp<'_>,
    ctx: &TuiContext<'_>,
) -> Result<AppStatus> {
    loop {
        terminal
            .draw(|frame| render(frame, app))
            .context("draw TUI frame")?;

        if let AppStatus::Committed(_) | AppStatus::Cancelled = app.status {
            return Ok(app.status.clone());
        }

        // Poll for user keyboard events with 100ms timeout
        if event::poll(Duration::from_millis(100)).context("poll terminal events")?
            && let Event::Key(key) = event::read().context("read key event")?
        {
            handle_key_event(terminal, app, key, ctx).await?;
        }
    }
}

/// Dispatches key event depending on the currently focused widget or popup.
async fn handle_key_event(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut TuiApp<'_>,
    key: KeyEvent,
    ctx: &TuiContext<'_>,
) -> Result<()> {
    // Clear any transient error on next keypress
    app.error_message = None;

    match app.focus {
        Focus::RegenPopup => {
            handle_regen_popup_key(app, key, ctx).await?;
        }
        Focus::HelpPopup => match key.code {
            KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => {
                app.focus = Focus::Diff;
            }
            _ => {}
        },
        Focus::Diff => {
            handle_diff_pane_key(terminal, app, key, ctx.configured_editor)?;
        }
        Focus::Editor => {
            handle_editor_pane_key(terminal, app, key, ctx.configured_editor)?;
        }
    }

    Ok(())
}

/// Handles keyboard input inside the Diff pane.
fn handle_diff_pane_key(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut TuiApp<'_>,
    key: KeyEvent,
    configured_editor: Option<&str>,
) -> Result<()> {
    match key.code {
        KeyCode::Tab | KeyCode::BackTab => {
            app.focus = Focus::Editor;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.scroll_diff_down(1);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.scroll_diff_up(1);
        }
        KeyCode::Char('d') | KeyCode::PageDown => {
            app.scroll_diff_down(10);
        }
        KeyCode::Char('u') | KeyCode::PageUp => {
            app.scroll_diff_up(10);
        }
        KeyCode::Enter => {
            let msg = app.get_commit_message();
            if !msg.is_empty() {
                app.status = AppStatus::Committed(msg);
            } else {
                app.error_message = Some("Commit message cannot be empty".to_string());
            }
        }
        KeyCode::Char('r') => {
            app.focus = Focus::RegenPopup;
            app.regen_input.clear();
        }
        KeyCode::Char('e') => {
            launch_external_editor(terminal, app, configured_editor)?;
        }
        KeyCode::Char('?') => {
            app.focus = Focus::HelpPopup;
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.status = AppStatus::Cancelled;
        }
        _ => {}
    }
    Ok(())
}

/// Handles keyboard input inside the Commit Message textarea.
fn handle_editor_pane_key(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut TuiApp<'_>,
    key: KeyEvent,
    configured_editor: Option<&str>,
) -> Result<()> {
    // Global shortcut intercepts inside editor pane
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('s') => {
                let msg = app.get_commit_message();
                if !msg.is_empty() {
                    app.status = AppStatus::Committed(msg);
                } else {
                    app.error_message = Some("Commit message cannot be empty".to_string());
                }
                return Ok(());
            }
            KeyCode::Char('r') => {
                app.focus = Focus::RegenPopup;
                app.regen_input.clear();
                return Ok(());
            }
            KeyCode::Char('e') => {
                launch_external_editor(terminal, app, configured_editor)?;
                return Ok(());
            }
            _ => {}
        }
    }

    match key.code {
        KeyCode::Tab | KeyCode::BackTab | KeyCode::Esc => {
            app.focus = Focus::Diff;
        }
        _ => {
            // Forward input event to tui-textarea
            app.textarea.input(key);
        }
    }

    Ok(())
}

/// Handles keyboard input inside the regeneration modal popup.
async fn handle_regen_popup_key(
    app: &mut TuiApp<'_>,
    key: KeyEvent,
    ctx: &TuiContext<'_>,
) -> Result<()> {
    match key.code {
        KeyCode::Enter => {
            let hint = app.regen_input.trim().to_string();
            let optional_hint = if hint.is_empty() {
                None
            } else {
                Some(hint.as_str())
            };

            app.status = AppStatus::Regenerating;
            app.focus = Focus::Editor;

            let (sys_p, usr_p) = build_prompt(
                &app.staged_diff.content,
                &app.staged_diff.staged_files,
                ctx.detailed,
                optional_hint,
            );

            match ctx
                .llm_client
                .generate_commit(ctx.provider_name, ctx.provider_config, &sys_p, &usr_p)
                .await
            {
                Ok(new_msg) => {
                    app.set_commit_message(&new_msg);
                    app.status = AppStatus::Editing;
                }
                Err(err) => {
                    app.error_message = Some(format!("Regeneration failed: {}", err));
                    app.status = AppStatus::Editing;
                }
            }
        }
        KeyCode::Esc => {
            app.focus = Focus::Diff;
        }
        KeyCode::Char(c) => {
            app.regen_input.push(c);
        }
        KeyCode::Backspace => {
            app.regen_input.pop();
        }
        _ => {}
    }

    Ok(())
}

/// Temporarily leaves raw mode / alternate screen to launch system $EDITOR.
fn launch_external_editor(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut TuiApp<'_>,
    configured_editor: Option<&str>,
) -> Result<()> {
    disable_raw_mode().context("disable raw mode for external editor")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .context("leave alternate screen for editor")?;

    let current_msg = app.get_commit_message();
    let edit_result = edit_message_in_editor(&current_msg, configured_editor);

    enable_raw_mode().context("re-enable raw mode after editor exit")?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)
        .context("re-enter alternate screen after editor exit")?;
    terminal.clear().context("clear terminal screen")?;

    match edit_result {
        Ok(edited) => {
            if !edited.trim().is_empty() {
                app.set_commit_message(&edited);
            }
        }
        Err(err) => {
            app.error_message = Some(format!("Editor error: {}", err));
        }
    }

    Ok(())
}
