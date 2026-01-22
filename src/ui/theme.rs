//! Theme and color definitions
//!
//! Uses a minimalist, modern color palette inspired by Tailwind CSS.

use ratatui::style::{Color, Modifier, Style};

/// Status colors for resource states
pub mod status {
    use super::*;

    /// Ready/Success status (green)
    pub const READY: Color = Color::Rgb(34, 197, 94); // green-500

    /// Failed/Error status (red)
    pub const FAILED: Color = Color::Rgb(239, 68, 68); // red-500

    /// Reconciling/In-progress status (yellow/amber)
    pub const RECONCILING: Color = Color::Rgb(245, 158, 11); // amber-500

    /// Suspended status (gray)
    pub const SUSPENDED: Color = Color::Rgb(107, 114, 128); // gray-500

    /// Unknown status (slate)
    pub const UNKNOWN: Color = Color::Rgb(148, 163, 184); // slate-400
}

/// UI element colors
pub mod ui {
    use super::*;

    /// Primary accent color (blue)
    pub const PRIMARY: Color = Color::Rgb(59, 130, 246); // blue-500

    /// Secondary/muted color
    #[allow(dead_code)]
    pub const SECONDARY: Color = Color::Rgb(148, 163, 184); // slate-400

    /// Background color
    #[allow(dead_code)]
    pub const BG: Color = Color::Reset;

    /// Border color
    pub const BORDER: Color = Color::Rgb(71, 85, 105); // slate-600

    /// Highlighted border
    pub const BORDER_HIGHLIGHT: Color = Color::Rgb(59, 130, 246); // blue-500

    /// Text color
    pub const TEXT: Color = Color::Rgb(226, 232, 240); // slate-200

    /// Muted text color
    pub const TEXT_MUTED: Color = Color::Rgb(148, 163, 184); // slate-400

    /// Selection background
    pub const SELECTION_BG: Color = Color::Rgb(30, 41, 59); // slate-800

    /// Tab active background
    pub const TAB_ACTIVE_BG: Color = Color::Rgb(30, 58, 138); // blue-900

    /// Header background
    pub const HEADER_BG: Color = Color::Rgb(15, 23, 42); // slate-900
}

/// Status icons
pub mod icons {
    /// Ready icon
    pub const READY: &str = "✓";

    /// Failed icon
    pub const FAILED: &str = "✗";

    /// Reconciling icon
    pub const RECONCILING: &str = "●";

    /// Suspended icon
    pub const SUSPENDED: &str = "⏸";

    /// Unknown icon
    pub const UNKNOWN: &str = "?";
}

/// Pre-defined styles for common UI elements
pub mod styles {
    use super::*;

    /// Default text style
    pub fn text() -> Style {
        Style::default().fg(ui::TEXT)
    }

    /// Muted text style
    pub fn muted() -> Style {
        Style::default().fg(ui::TEXT_MUTED)
    }

    /// Header style
    pub fn header() -> Style {
        Style::default()
            .fg(ui::TEXT)
            .bg(ui::HEADER_BG)
            .add_modifier(Modifier::BOLD)
    }

    /// Selected row style
    pub fn selected() -> Style {
        Style::default().bg(ui::SELECTION_BG).fg(ui::TEXT)
    }

    /// Border style
    pub fn border() -> Style {
        Style::default().fg(ui::BORDER)
    }

    /// Highlighted border style
    pub fn border_highlight() -> Style {
        Style::default().fg(ui::BORDER_HIGHLIGHT)
    }

    /// Tab style (inactive)
    pub fn tab() -> Style {
        Style::default().fg(ui::TEXT_MUTED)
    }

    /// Tab style (active)
    pub fn tab_active() -> Style {
        Style::default()
            .fg(ui::TEXT)
            .bg(ui::TAB_ACTIVE_BG)
            .add_modifier(Modifier::BOLD)
    }

    /// Status style based on resource state
    pub fn status_ready() -> Style {
        Style::default().fg(status::READY)
    }

    pub fn status_failed() -> Style {
        Style::default().fg(status::FAILED)
    }

    pub fn status_reconciling() -> Style {
        Style::default().fg(status::RECONCILING)
    }

    pub fn status_suspended() -> Style {
        Style::default().fg(status::SUSPENDED)
    }

    pub fn status_unknown() -> Style {
        Style::default().fg(status::UNKNOWN)
    }

    /// Title style
    pub fn title() -> Style {
        Style::default()
            .fg(ui::PRIMARY)
            .add_modifier(Modifier::BOLD)
    }

    /// Keybinding key style
    pub fn key() -> Style {
        Style::default()
            .fg(ui::PRIMARY)
            .add_modifier(Modifier::BOLD)
    }

    /// Keybinding description style
    pub fn key_desc() -> Style {
        Style::default().fg(ui::TEXT_MUTED)
    }
}
