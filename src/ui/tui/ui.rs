// ==============================================================================
// TUI Layout and Rendering Engine
// ==============================================================================
//
// Draws the dual-pane Ratatui interface: colorized diff on the left,
// interactive textarea editor on the right, status bar, and modal popups.

use crate::ui::tui::app::{AppStatus, Focus, TuiApp};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap};

/// Renders the complete TUI screen for the current frame.
pub fn render(frame: &mut Frame, app: &mut TuiApp) {
    let size = frame.area();

    // 1. Divide main vertical space: Header (3), Body (Flexible), Footer (3)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(size);

    render_header(frame, chunks[0], app);
    render_body(frame, chunks[1], app);
    render_footer(frame, chunks[2], app);

    // 2. Render modal overlays if active
    if app.focus == Focus::RegenPopup {
        render_regen_popup(frame, size, app);
    } else if app.focus == Focus::HelpPopup {
        render_help_popup(frame, size);
    }
}

/// Renders top application title and provider metadata banner.
fn render_header(frame: &mut Frame, area: Rect, app: &TuiApp) {
    let files_summary = format!("Staged Files ({})", app.staged_diff.staged_files.len());
    let provider_info = format!("[{}:{}]", app.provider_name, app.model_name);

    let status_span = match &app.status {
        AppStatus::Editing => Span::styled(" [Editing] ", Style::default().fg(Color::Green)),
        AppStatus::Regenerating => Span::styled(
            " [Generating...] ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::RAPID_BLINK),
        ),
        AppStatus::Committed(_) => Span::styled(" [Committed] ", Style::default().fg(Color::Cyan)),
        AppStatus::Cancelled => Span::styled(" [Cancelled] ", Style::default().fg(Color::DarkGray)),
    };

    let title_line = Line::from(vec![
        Span::styled(
            " 🦀 Cogit ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("— AI Conventional Commit Assistant "),
        status_span,
        Span::styled(provider_info, Style::default().fg(Color::Magenta)),
        Span::raw(" | "),
        Span::styled(files_summary, Style::default().fg(Color::Yellow)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Blue));

    let paragraph = Paragraph::new(title_line)
        .block(block)
        .alignment(Alignment::Left);
    frame.render_widget(paragraph, area);
}

/// Renders the dual-pane body: Left Diff Viewer (55%) + Right Textarea Editor (45%).
fn render_body(frame: &mut Frame, area: Rect, app: &mut TuiApp) {
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    render_diff_pane(frame, body_chunks[0], app);
    render_editor_pane(frame, body_chunks[1], app);
}

/// Renders colorized unified diff view with scroll indicator.
fn render_diff_pane(frame: &mut Frame, area: Rect, app: &TuiApp) {
    let is_focused = app.focus == Focus::Diff;
    let border_color = if is_focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            if is_focused {
                " 📄 Staged Diff (Focused: j/k to scroll) "
            } else {
                " 📄 Staged Diff "
            },
            Style::default()
                .fg(if is_focused {
                    Color::Cyan
                } else {
                    Color::White
                })
                .add_modifier(Modifier::BOLD),
        ));

    let inner_height = area.height.saturating_sub(2) as usize;
    let visible_lines: Vec<Line> = app
        .diff_lines
        .iter()
        .skip(app.diff_scroll)
        .take(inner_height)
        .map(|raw_line| colorize_diff_line(raw_line))
        .collect();

    let paragraph = Paragraph::new(visible_lines).block(block);
    frame.render_widget(paragraph, area);
}

/// Colorizes unified git diff syntax elements with zero-allocation slice borrowing.
fn colorize_diff_line<'a>(line: &'a str) -> Line<'a> {
    let style = if line.starts_with('+') && !line.starts_with("+++") {
        Style::default().fg(Color::Green)
    } else if line.starts_with('-') && !line.starts_with("---") {
        Style::default().fg(Color::Red)
    } else if line.starts_with("@@") {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else if line.starts_with("diff --git") || line.starts_with("index ") {
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD)
    } else if line.starts_with("---") || line.starts_with("+++") {
        Style::default().fg(Color::Yellow)
    } else if line.starts_with('#') {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default()
    };

    Line::from(Span::styled(line, style))
}

/// Renders the editable commit message textarea pane.
fn render_editor_pane(frame: &mut Frame, area: Rect, app: &mut TuiApp) {
    let is_focused = app.focus == Focus::Editor;
    let border_color = if is_focused {
        Color::Green
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            if is_focused {
                " ✏️ Commit Message (Focused: Type or Edit) "
            } else {
                " ✏️ Commit Message "
            },
            Style::default()
                .fg(if is_focused {
                    Color::Green
                } else {
                    Color::White
                })
                .add_modifier(Modifier::BOLD),
        ));

    app.textarea.set_block(block);
    frame.render_widget(&app.textarea, area);
}

/// Renders the footer with keyboard shortcuts and any error notifications.
fn render_footer(frame: &mut Frame, area: Rect, app: &TuiApp) {
    let footer_content = if let Some(ref err) = app.error_message {
        Line::from(vec![
            Span::styled(
                " Error: ",
                Style::default()
                    .bg(Color::Red)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!(" {} ", err), Style::default().fg(Color::Red)),
        ])
    } else {
        Line::from(vec![
            Span::styled(
                "[Tab]",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Switch Pane | "),
            Span::styled(
                "[Enter]",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Commit | "),
            Span::styled(
                "[r]",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Regenerate | "),
            Span::styled(
                "[e]",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" $EDITOR | "),
            Span::styled(
                "[?]",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Help | "),
            Span::styled(
                "[Esc/q]",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Cancel"),
        ])
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(footer_content)
        .block(block)
        .alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}

/// Renders modal overlay for entering regeneration guidance/context.
fn render_regen_popup(frame: &mut Frame, area: Rect, app: &TuiApp) {
    let popup_area = centered_rect(60, 25, area);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            " 🔄 Regenerate Commit Message ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));

    let text = vec![
        Line::from(Span::raw(
            "Enter additional guidance or instructions for the LLM:",
        )),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            format!("> {}_", app.regen_input),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::raw("")),
        Line::from(vec![
            Span::styled(
                "[Enter]",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Submit | "),
            Span::styled(
                "[Esc]",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Cancel"),
        ]),
    ];

    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, popup_area);
}

/// Renders modal overlay displaying full keyboard shortcuts cheat sheet.
fn render_help_popup(frame: &mut Frame, area: Rect) {
    let popup_area = centered_rect(50, 45, area);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Yellow))
        .title(Span::styled(
            " 💡 Keyboard Shortcuts ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));

    let text = vec![
        Line::from(vec![
            Span::styled(
                " Tab / BackTab   ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Switch between Diff & Commit Editor"),
        ]),
        Line::from(vec![
            Span::styled(
                " Enter / Ctrl+S  ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Commit staged changes with current text"),
        ]),
        Line::from(vec![
            Span::styled(
                " r               ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Open LLM regeneration popup (when not in editor)"),
        ]),
        Line::from(vec![
            Span::styled(
                " e               ",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Open external $EDITOR on message buffer"),
        ]),
        Line::from(vec![
            Span::styled(
                " j / k / Arrows  ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Scroll diff view when Diff pane is focused"),
        ]),
        Line::from(vec![
            Span::styled(
                " ?               ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("Toggle this help popup"),
        ]),
        Line::from(vec![
            Span::styled(
                " Esc / q         ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw("Close popup or abort/cancel cogit"),
        ]),
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            "Press [Esc] to dismiss help",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(text).block(block).alignment(Alignment::Left);
    frame.render_widget(paragraph, popup_area);
}

/// Helper function to center a popup rect within the parent area.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
