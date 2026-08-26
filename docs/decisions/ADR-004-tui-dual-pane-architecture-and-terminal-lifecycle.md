# ADR-004: Dual-Pane TUI Architecture and Terminal Lifecycle

## Status
Accepted

## Date
2026-08-20

## Context
Developers reviewing complex Git diffs benefit from seeing the staged diff and the generated commit message side-by-side. The interface must:
1. Render colorized unified diff syntax with fast vertical scrolling.
2. Provide an interactive in-place commit message editor (`tui-textarea`).
3. Support in-app AI regeneration with custom prompts.
4. Support seamless handoff to external editors (`$EDITOR` / `$VISUAL`) without corrupting terminal state.
5. Reliably restore the user's terminal upon exit.

## Decision
1. Implement a dual-pane layout using `ratatui 0.29.0` and `tui-textarea 0.7.0`:
   - Left Pane (55%): Colorized diff view with `j`/`k`/`d`/`u` scrolling.
   - Right Pane (45%): Editable `TextArea` buffer.
2. Structure the TUI into discrete layers:
   - `src/ui/tui/app.rs`: Pure state model (`TuiApp`, `Focus`, `AppStatus`).
   - `src/ui/tui/ui.rs`: Frame rendering and syntax highlighting.
   - `src/ui/tui/events.rs`: Terminal raw mode management and non-blocking event polling.
3. Manage the terminal lifecycle with `crossterm::terminal::{enable_raw_mode, EnterAlternateScreen}` and explicit teardown handlers.
4. For `$EDITOR` handoffs, temporarily disable raw mode and leave the alternate screen, invoke the editor process, then re-enter the alternate screen and re-enable raw mode.

## Consequences
- Responsive, zero-latency visual diff inspection.
- In-place editing with arrow keys, cursor positioning, and full unicode support.
- Clean terminal restoration on normal and cancelled exits.
