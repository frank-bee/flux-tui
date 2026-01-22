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
    vec![("↑↓", "Select"), ("Enter", "Apply"), ("Esc", "Cancel")]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_short_string() {
        let result = truncate("hello", 10);
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_truncate_exact_length() {
        let result = truncate("hello", 5);
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_truncate_long_string() {
        let result = truncate("hello world", 8);
        assert_eq!(result, "hello...");
    }

    #[test]
    fn test_truncate_very_long_string() {
        let result = truncate("this is a very long string", 15);
        assert_eq!(result, "this is a ve...");
    }

    #[test]
    fn test_normal_keybindings() {
        let bindings = normal_keybindings();
        assert!(!bindings.is_empty());
        // Check first and last bindings
        assert_eq!(bindings[0], ("↑↓", "Navigate"));
        assert_eq!(bindings[bindings.len() - 1], ("q", "Quit"));
    }

    #[test]
    fn test_namespace_keybindings() {
        let bindings = namespace_keybindings();
        assert_eq!(bindings.len(), 3);
        assert_eq!(bindings[0], ("↑↓", "Select"));
    }

    #[test]
    fn test_popup_keybindings() {
        let bindings = popup_keybindings();
        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0], ("Esc", "Close"));
    }

    #[test]
    fn test_error_keybindings() {
        let bindings = error_keybindings();
        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0], ("Enter/Esc", "Dismiss"));
    }

    #[test]
    fn test_normal_keybindings_complete() {
        let bindings = normal_keybindings();
        // Verify all expected bindings are present
        assert_eq!(bindings.len(), 9);

        let expected = [
            ("↑↓", "Navigate"),
            ("←→", "Tabs"),
            ("Enter", "Details"),
            ("r", "Reconcile"),
            ("R", "+Source"),
            ("s", "Suspend"),
            ("n", "Namespace"),
            ("F5", "Refresh"),
            ("q", "Quit"),
        ];

        for (i, (key, desc)) in expected.iter().enumerate() {
            assert_eq!(bindings[i].0, *key);
            assert_eq!(bindings[i].1, *desc);
        }
    }

    #[test]
    fn test_namespace_keybindings_complete() {
        let bindings = namespace_keybindings();
        assert_eq!(bindings[0], ("↑↓", "Select"));
        assert_eq!(bindings[1], ("Enter", "Apply"));
        assert_eq!(bindings[2], ("Esc", "Cancel"));
    }

    #[test]
    fn test_popup_keybindings_complete() {
        let bindings = popup_keybindings();
        assert_eq!(bindings[0], ("Esc", "Close"));
        assert_eq!(bindings[1], ("q", "Quit"));
    }

    #[test]
    fn test_error_keybindings_complete() {
        let bindings = error_keybindings();
        assert_eq!(bindings[0], ("Enter/Esc", "Dismiss"));
        assert_eq!(bindings[1], ("q", "Quit"));
    }

    #[test]
    fn test_truncate_edge_cases() {
        // Empty string
        assert_eq!(truncate("", 10), "");

        // String with exactly max_len - 3 characters
        assert_eq!(truncate("abcde", 8), "abcde");

        // String with exactly max_len - 2 characters
        assert_eq!(truncate("abcdef", 8), "abcdef");

        // String with exactly max_len - 1 characters
        assert_eq!(truncate("abcdefg", 8), "abcdefg");

        // String with exactly max_len characters
        assert_eq!(truncate("abcdefgh", 8), "abcdefgh");

        // String with max_len + 1 characters
        assert_eq!(truncate("abcdefghi", 8), "abcde...");
    }
}
