//! Status bar widget showing keybindings

use ratatui::{prelude::*, widgets::Paragraph};

use crate::app::state::{App, Popup};
use crate::ui::theme::styles;

/// Draw the status bar with keybindings
pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let keybindings = match &app.popup {
        Popup::None => normal_keybindings(),
        Popup::NamespaceFilter { .. } => namespace_keybindings(),
        Popup::ResourceDetails { .. } | Popup::Reconciling { .. } => popup_keybindings(),
        Popup::Error { .. } => error_keybindings(),
    };

    // Build the status bar with styled spans
    let mut spans: Vec<Span> = Vec::new();
    for (i, (key, desc)) in keybindings.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" │ ", styles::muted()));
        }
        spans.push(Span::styled(*key, styles::key()));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(*desc, styles::key_desc()));
    }

    // Add error message if present
    if let Some(ref error) = app.last_error {
        spans.push(Span::styled(
            format!("  │  Error: {}", truncate(error, 40)),
            Style::default().fg(crate::ui::theme::status::FAILED),
        ));
    }

    let paragraph = Paragraph::new(Line::from(spans));

    frame.render_widget(paragraph, area);
}

/// Normal mode keybindings
fn normal_keybindings() -> Vec<(&'static str, &'static str)> {
    vec![
        ("↑↓", "Navigate"),
        ("←→", "Tabs"),
        ("Enter", "Details"),
        ("r", "Reconcile"),
        ("R", "+Source"),
        ("s", "Suspend"),
        ("n", "Namespace"),
        ("F5", "Refresh"),
        ("q", "Quit"),
    ]
}

/// Namespace popup keybindings
fn namespace_keybindings() -> Vec<(&'static str, &'static str)> {
    vec![
        ("↑↓", "Select"),
        ("Enter", "Apply"),
        ("Esc", "Cancel"),
    ]
}

/// Generic popup keybindings
fn popup_keybindings() -> Vec<(&'static str, &'static str)> {
    vec![("Esc", "Close"), ("q", "Quit")]
}

/// Error popup keybindings
fn error_keybindings() -> Vec<(&'static str, &'static str)> {
    vec![("Enter/Esc", "Dismiss"), ("q", "Quit")]
}

/// Truncate a string to a maximum length
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
