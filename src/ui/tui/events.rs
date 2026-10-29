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
use tokio::task::JoinHandle;

struct PendingGeneration(Option<JoinHandle<Result<String>>>);

impl Drop for PendingGeneration {
    fn drop(&mut self) {
        if let Some(task) = self.0.take() {
            task.abort();
        }
    }
}

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
    let mut pending = PendingGeneration(None);
    loop {
        if let AppStatus::Committed(_) | AppStatus::Cancelled = app.status {
            return Ok(app.status.clone());
        }
        if pending.0.as_ref().is_some_and(JoinHandle::is_finished) {
            let task = pending.0.take().expect("finished generation task");
            let result = task.await.unwrap_or_else(|err| Err(err.into()));
            finish_regeneration(app, result);
        }
        terminal
            .draw(|frame| render(frame, app))
            .context("draw TUI frame")?;

        // Poll for user keyboard events with 100ms timeout
        if event::poll(Duration::from_millis(100)).context("poll terminal events")?
            && let Event::Key(key) = event::read().context("read key event")?
        {
            if app.status == AppStatus::Regenerating {
                handle_generating_key(app, key);
            } else {
                pending.0 = handle_key_event(terminal, app, key, ctx)?;
            }
        }
    }
}

fn handle_generating_key(app: &mut TuiApp<'_>, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.status = AppStatus::Cancelled,
        KeyCode::Tab | KeyCode::BackTab => app.toggle_focus(),
        KeyCode::Char('j') | KeyCode::Down if app.focus == Focus::Diff => app.scroll_diff_down(1),
        KeyCode::Char('k') | KeyCode::Up if app.focus == Focus::Diff => app.scroll_diff_up(1),
        KeyCode::Char('d') | KeyCode::PageDown if app.focus == Focus::Diff => {
            app.scroll_diff_down(10)
        }
        KeyCode::Char('u') | KeyCode::PageUp if app.focus == Focus::Diff => app.scroll_diff_up(10),
        _ => {}
    }
}

/// Dispatches key event depending on the currently focused widget or popup.
fn handle_key_event(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut TuiApp<'_>,
    key: KeyEvent,
    ctx: &TuiContext<'_>,
) -> Result<Option<JoinHandle<Result<String>>>> {
    // Clear any transient error on next keypress
    app.error_message = None;

    match app.focus {
        Focus::RegenPopup => {
            return Ok(handle_regen_popup_key(app, key, ctx));
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

    Ok(None)
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
fn handle_regen_popup_key(
    app: &mut TuiApp<'_>,
    key: KeyEvent,
    ctx: &TuiContext<'_>,
) -> Option<JoinHandle<Result<String>>> {
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

            let client = ctx.llm_client.clone();
            let provider_name = ctx.provider_name.to_owned();
            let provider_config = ctx.provider_config.clone();
            return Some(tokio::spawn(async move {
                client
                    .generate_commit(&provider_name, &provider_config, &sys_p, &usr_p)
                    .await
            }));
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

    None
}

fn finish_regeneration(app: &mut TuiApp<'_>, result: Result<String>) {
    match result {
        Ok(message) => app.set_commit_message(&message),
        Err(err) => app.error_message = Some(format!("Regeneration failed: {}", err)),
    }
    app.status = AppStatus::Editing;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> TuiApp<'static> {
        TuiApp::new(
            StagedDiff {
                content: "diff".into(),
                staged_files: vec!["file".into()],
                omitted_files: vec![],
                is_truncated: false,
            },
            "test".into(),
            "model".into(),
            "original",
        )
    }

    #[test]
    fn regeneration_completion_preserves_original_on_failure() {
        let mut app = app();
        app.status = AppStatus::Regenerating;
        finish_regeneration(&mut app, Err(anyhow::anyhow!("unavailable")));
        assert_eq!(app.status, AppStatus::Editing);
        assert_eq!(app.get_commit_message(), "original");
        assert!(
            app.error_message
                .as_deref()
                .unwrap()
                .contains("unavailable")
        );

        app.status = AppStatus::Regenerating;
        app.error_message = None;
        finish_regeneration(&mut app, Ok("new message".into()));
        assert_eq!(app.status, AppStatus::Editing);
        assert_eq!(app.get_commit_message(), "new message");
    }

    #[test]
    fn generating_allows_navigation_and_cancel_without_editing() {
        let mut app = app();
        app.status = AppStatus::Regenerating;
        handle_generating_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE),
        );
        assert_eq!(app.get_commit_message(), "original");
        handle_generating_key(&mut app, KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(app.focus, Focus::Diff);
        handle_generating_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(app.status, AppStatus::Cancelled);
    }

    #[tokio::test]
    async fn dropping_pending_generation_aborts_request() {
        let task = tokio::spawn(std::future::pending::<Result<String>>());
        let abort = task.abort_handle();
        drop(PendingGeneration(Some(task)));
        tokio::task::yield_now().await;
        assert!(abort.is_finished());
    }

    #[tokio::test]
    async fn regeneration_renders_while_request_is_pending() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::sync::mpsc;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let (release, wait) = mpsc::channel();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut request = [0; 4096];
            assert!(stream.read(&mut request).unwrap() > 0);
            wait.recv_timeout(Duration::from_secs(5)).unwrap();
            let body = r#"{"choices":[{"message":{"content":"fix: generated"}}]}"#;
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            )
            .unwrap();
        });
        let client = LlmClient::new();
        let config = ProviderConfig {
            kind: Some(crate::config::ProviderKind::OpenAi),
            base_url: Some(format!("http://{address}/v1")),
            model: "test".into(),
            api_key: None,
        };
        let ctx = TuiContext {
            llm_client: &client,
            provider_name: "test",
            provider_config: &config,
            detailed: false,
            configured_editor: None,
        };
        let mut app = app();
        app.focus = Focus::RegenPopup;
        let task = handle_regen_popup_key(
            &mut app,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            &ctx,
        )
        .unwrap();
        assert_eq!(app.status, AppStatus::Regenerating);
        let mut terminal = Terminal::new(ratatui::backend::TestBackend::new(100, 20)).unwrap();
        terminal.draw(|frame| render(frame, &mut app)).unwrap();
        let screen = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(screen.contains("[Generating...]"));
        release.send(()).unwrap();
        finish_regeneration(&mut app, task.await.unwrap());
        assert_eq!(app.get_commit_message(), "fix: generated");
        server.join().unwrap();
    }
}
