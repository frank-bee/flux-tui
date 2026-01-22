//! Key event handler - maps keyboard input to actions

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{
    actions::Action,
    state::{App, Popup},
};

/// Convert a key event to an application action
pub fn handle_key_event(key: KeyEvent, app: &App) -> Action {
    // Handle popup-specific keys first
    match &app.popup {
        Popup::None => handle_normal_keys(key),
        Popup::NamespaceFilter { namespaces, selected } => {
            handle_namespace_popup_keys(key, namespaces, *selected)
        }
        Popup::ResourceDetails { .. } => handle_details_popup_keys(key),
        Popup::Reconciling { .. } => handle_reconciling_popup_keys(key),
        Popup::Error { .. } => handle_error_popup_keys(key),
    }
}

/// Handle keys in normal mode (no popup)
fn handle_normal_keys(key: KeyEvent) -> Action {
    match key.code {
        // Quit
        KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,

        // Navigation
        KeyCode::Up | KeyCode::Char('k') => Action::Up,
        KeyCode::Down | KeyCode::Char('j') => Action::Down,
        KeyCode::Home | KeyCode::Char('g') => Action::Top,
        KeyCode::End | KeyCode::Char('G') => Action::Bottom,

        // Tab navigation
        KeyCode::Left | KeyCode::Char('h') => Action::PreviousTab,
        KeyCode::Right | KeyCode::Char('l') => Action::NextTab,
        KeyCode::Tab => Action::NextTab,
        KeyCode::BackTab => Action::PreviousTab,

        // Actions
        KeyCode::Enter => Action::Select,
        KeyCode::Char('r') => Action::Reconcile,
        KeyCode::Char('R') => Action::ReconcileWithSource,
        KeyCode::Char('s') => Action::ToggleSuspend,
        KeyCode::Char('n') => Action::FilterNamespace,
        KeyCode::F(5) => Action::Refresh,

        _ => Action::Noop,
    }
}

/// Handle keys in namespace filter popup
fn handle_namespace_popup_keys(
    key: KeyEvent,
    namespaces: &[String],
    selected: usize,
) -> Action {
    match key.code {
        KeyCode::Esc => Action::ClosePopup,
        KeyCode::Up | KeyCode::Char('k') => {
            // Move selection up in popup (handled via SetNamespace with current selection)
            if selected > 0 {
                // For simplicity, we close and re-open with new selection
                // A more sophisticated approach would have dedicated popup navigation actions
                Action::Noop
            } else {
                Action::Noop
            }
        }
        KeyCode::Down | KeyCode::Char('j') => Action::Noop,
        KeyCode::Enter => {
            // Apply selected namespace
            if selected == 0 {
                Action::SetNamespace(None) // "All namespaces"
            } else {
                Action::SetNamespace(namespaces.get(selected).cloned())
            }
        }
        KeyCode::Char('q') => Action::Quit,
        _ => Action::Noop,
    }
}

/// Handle keys in resource details popup
fn handle_details_popup_keys(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc | KeyCode::Enter => Action::ClosePopup,
        KeyCode::Char('q') => Action::Quit,
        _ => Action::Noop,
    }
}

/// Handle keys while reconciling (mostly just wait)
fn handle_reconciling_popup_keys(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') => Action::Quit,
        _ => Action::Noop,
    }
}

/// Handle keys in error popup
fn handle_error_popup_keys(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc | KeyCode::Enter => Action::ClosePopup,
        KeyCode::Char('q') => Action::Quit,
        _ => Action::Noop,
    }
}
