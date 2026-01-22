//! Layout definitions for the UI

use ratatui::prelude::*;

/// Main layout areas
pub struct MainLayout {
    /// Header area (title bar)
    pub header: Rect,
    /// Tabs area
    pub tabs: Rect,
    /// Main content area (table)
    pub content: Rect,
    /// Status bar area
    pub status_bar: Rect,
}

impl MainLayout {
    /// Calculate the main layout from the terminal area
    pub fn new(area: Rect) -> Self {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Header
                Constraint::Length(3), // Tabs
                Constraint::Min(5),    // Content
                Constraint::Length(1), // Status bar
            ])
            .split(area);

        Self {
            header: chunks[0],
            tabs: chunks[1],
            content: chunks[2],
            status_bar: chunks[3],
        }
    }
}

/// Popup layout - centered on screen
pub fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
