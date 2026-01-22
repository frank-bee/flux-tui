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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_layout_new() {
        let area = Rect::new(0, 0, 100, 50);
        let layout = MainLayout::new(area);

        // Header should be at the top with height 1
        assert_eq!(layout.header.y, 0);
        assert_eq!(layout.header.height, 1);

        // Tabs should be below header with height 3
        assert_eq!(layout.tabs.y, 1);
        assert_eq!(layout.tabs.height, 3);

        // Status bar should be at the bottom with height 1
        assert_eq!(layout.status_bar.height, 1);
        assert_eq!(layout.status_bar.y, 49); // Last row

        // Content should fill the remaining space
        assert_eq!(layout.content.y, 4); // After header (1) + tabs (3)
        assert_eq!(layout.content.height, 45); // 50 - 1 - 3 - 1
    }

    #[test]
    fn test_main_layout_widths() {
        let area = Rect::new(0, 0, 100, 50);
        let layout = MainLayout::new(area);

        // All sections should span full width
        assert_eq!(layout.header.width, 100);
        assert_eq!(layout.tabs.width, 100);
        assert_eq!(layout.content.width, 100);
        assert_eq!(layout.status_bar.width, 100);
    }

    #[test]
    fn test_popup_area_centered() {
        let area = Rect::new(0, 0, 100, 100);
        let popup = popup_area(area, 50, 50);

        // Popup should be roughly centered
        // With 50% width/height, it should be at approximately 25% from edges
        assert!(popup.x >= 20 && popup.x <= 30);
        assert!(popup.y >= 20 && popup.y <= 30);
        assert!(popup.width >= 45 && popup.width <= 55);
        assert!(popup.height >= 45 && popup.height <= 55);
    }

    #[test]
    fn test_popup_area_small() {
        let area = Rect::new(0, 0, 100, 100);
        let popup = popup_area(area, 20, 20);

        // Smaller popup should be more centered with larger margins
        assert!(popup.x >= 35 && popup.x <= 45);
        assert!(popup.y >= 35 && popup.y <= 45);
        assert!(popup.width <= 25);
        assert!(popup.height <= 25);
    }

    #[test]
    fn test_main_layout_small_terminal() {
        // Test with a small terminal size
        let area = Rect::new(0, 0, 40, 10);
        let layout = MainLayout::new(area);

        // Should still create valid layout
        assert_eq!(layout.header.height, 1);
        assert_eq!(layout.tabs.height, 3);
        assert_eq!(layout.status_bar.height, 1);
        // Content gets minimum 5 lines
        assert!(layout.content.height >= 5);
    }
}
