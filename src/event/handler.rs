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
        Popup::NamespaceFilter {
            namespaces,
            selected,
        } => handle_namespace_popup_keys(key, namespaces, *selected),
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
fn handle_namespace_popup_keys(key: KeyEvent, namespaces: &[String], selected: usize) -> Action {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn key_with_mod(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn test_handle_normal_keys_quit() {
        assert_eq!(handle_normal_keys(key(KeyCode::Char('q'))), Action::Quit);
        assert_eq!(handle_normal_keys(key(KeyCode::Esc)), Action::Quit);
        assert_eq!(
            handle_normal_keys(key_with_mod(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            Action::Quit
        );
    }

    #[test]
    fn test_handle_normal_keys_navigation() {
        assert_eq!(handle_normal_keys(key(KeyCode::Up)), Action::Up);
        assert_eq!(handle_normal_keys(key(KeyCode::Char('k'))), Action::Up);
        assert_eq!(handle_normal_keys(key(KeyCode::Down)), Action::Down);
        assert_eq!(handle_normal_keys(key(KeyCode::Char('j'))), Action::Down);
        assert_eq!(handle_normal_keys(key(KeyCode::Home)), Action::Top);
        assert_eq!(handle_normal_keys(key(KeyCode::Char('g'))), Action::Top);
        assert_eq!(handle_normal_keys(key(KeyCode::End)), Action::Bottom);
        assert_eq!(handle_normal_keys(key(KeyCode::Char('G'))), Action::Bottom);
    }

    #[test]
    fn test_handle_normal_keys_tab_navigation() {
        assert_eq!(handle_normal_keys(key(KeyCode::Left)), Action::PreviousTab);
        assert_eq!(
            handle_normal_keys(key(KeyCode::Char('h'))),
            Action::PreviousTab
        );
        assert_eq!(
            handle_normal_keys(key(KeyCode::BackTab)),
            Action::PreviousTab
        );
        assert_eq!(handle_normal_keys(key(KeyCode::Right)), Action::NextTab);
        assert_eq!(handle_normal_keys(key(KeyCode::Char('l'))), Action::NextTab);
        assert_eq!(handle_normal_keys(key(KeyCode::Tab)), Action::NextTab);
    }

    #[test]
    fn test_handle_normal_keys_actions() {
        assert_eq!(handle_normal_keys(key(KeyCode::Enter)), Action::Select);
        assert_eq!(
            handle_normal_keys(key(KeyCode::Char('r'))),
            Action::Reconcile
        );
        assert_eq!(
            handle_normal_keys(key(KeyCode::Char('R'))),
            Action::ReconcileWithSource
        );
        assert_eq!(
            handle_normal_keys(key(KeyCode::Char('s'))),
            Action::ToggleSuspend
        );
        assert_eq!(
            handle_normal_keys(key(KeyCode::Char('n'))),
            Action::FilterNamespace
        );
        assert_eq!(handle_normal_keys(key(KeyCode::F(5))), Action::Refresh);
    }

    #[test]
    fn test_handle_normal_keys_noop() {
        assert_eq!(handle_normal_keys(key(KeyCode::Char('x'))), Action::Noop);
        assert_eq!(handle_normal_keys(key(KeyCode::F(1))), Action::Noop);
    }

    #[test]
    fn test_handle_namespace_popup_keys_close() {
        let namespaces = vec!["All namespaces".to_string(), "default".to_string()];
        assert_eq!(
            handle_namespace_popup_keys(key(KeyCode::Esc), &namespaces, 0),
            Action::ClosePopup
        );
        assert_eq!(
            handle_namespace_popup_keys(key(KeyCode::Char('q')), &namespaces, 0),
            Action::Quit
        );
    }

    #[test]
    fn test_handle_namespace_popup_keys_select_all() {
        let namespaces = vec!["All namespaces".to_string(), "default".to_string()];
        assert_eq!(
            handle_namespace_popup_keys(key(KeyCode::Enter), &namespaces, 0),
            Action::SetNamespace(None)
        );
    }

    #[test]
    fn test_handle_namespace_popup_keys_select_specific() {
        let namespaces = vec![
            "All namespaces".to_string(),
            "default".to_string(),
            "kube-system".to_string(),
        ];
        assert_eq!(
            handle_namespace_popup_keys(key(KeyCode::Enter), &namespaces, 1),
            Action::SetNamespace(Some("default".to_string()))
        );
        assert_eq!(
            handle_namespace_popup_keys(key(KeyCode::Enter), &namespaces, 2),
            Action::SetNamespace(Some("kube-system".to_string()))
        );
    }

    #[test]
    fn test_handle_namespace_popup_keys_navigation() {
        let namespaces = vec!["All namespaces".to_string(), "default".to_string()];
        assert_eq!(
            handle_namespace_popup_keys(key(KeyCode::Up), &namespaces, 1),
            Action::Noop
        );
        assert_eq!(
            handle_namespace_popup_keys(key(KeyCode::Char('k')), &namespaces, 1),
            Action::Noop
        );
        assert_eq!(
            handle_namespace_popup_keys(key(KeyCode::Down), &namespaces, 0),
            Action::Noop
        );
        assert_eq!(
            handle_namespace_popup_keys(key(KeyCode::Char('j')), &namespaces, 0),
            Action::Noop
        );
    }

    #[test]
    fn test_handle_details_popup_keys() {
        assert_eq!(
            handle_details_popup_keys(key(KeyCode::Esc)),
            Action::ClosePopup
        );
        assert_eq!(
            handle_details_popup_keys(key(KeyCode::Enter)),
            Action::ClosePopup
        );
        assert_eq!(
            handle_details_popup_keys(key(KeyCode::Char('q'))),
            Action::Quit
        );
        assert_eq!(
            handle_details_popup_keys(key(KeyCode::Char('x'))),
            Action::Noop
        );
    }

    #[test]
    fn test_handle_reconciling_popup_keys() {
        assert_eq!(
            handle_reconciling_popup_keys(key(KeyCode::Char('q'))),
            Action::Quit
        );
        assert_eq!(
            handle_reconciling_popup_keys(key(KeyCode::Esc)),
            Action::Noop
        );
        assert_eq!(
            handle_reconciling_popup_keys(key(KeyCode::Enter)),
            Action::Noop
        );
    }

    #[test]
    fn test_handle_error_popup_keys() {
        assert_eq!(
            handle_error_popup_keys(key(KeyCode::Esc)),
            Action::ClosePopup
        );
        assert_eq!(
            handle_error_popup_keys(key(KeyCode::Enter)),
            Action::ClosePopup
        );
        assert_eq!(
            handle_error_popup_keys(key(KeyCode::Char('q'))),
            Action::Quit
        );
        assert_eq!(
            handle_error_popup_keys(key(KeyCode::Char('x'))),
            Action::Noop
        );
    }

    #[test]
    fn test_handle_namespace_popup_keys_up_at_zero() {
        let namespaces = vec!["All namespaces".to_string(), "default".to_string()];
        // At position 0, up should return Noop (can't go higher)
        assert_eq!(
            handle_namespace_popup_keys(key(KeyCode::Up), &namespaces, 0),
            Action::Noop
        );
    }

    // ===== Integration tests for handle_key_event =====

    #[test]
    fn test_handle_key_event_normal_mode() {
        let app = App::for_testing(
            crate::app::state::Tab::Kustomizations,
            vec![],
            vec![],
            vec![],
        );

        assert_eq!(
            handle_key_event(key(KeyCode::Char('q')), &app),
            Action::Quit
        );
        assert_eq!(handle_key_event(key(KeyCode::Up), &app), Action::Up);
        assert_eq!(handle_key_event(key(KeyCode::Enter), &app), Action::Select);
    }

    #[test]
    fn test_handle_key_event_namespace_popup() {
        let mut app = App::for_testing(
            crate::app::state::Tab::Kustomizations,
            vec![],
            vec![],
            vec![],
        );
        app.popup = Popup::NamespaceFilter {
            namespaces: vec!["All".to_string(), "default".to_string()],
            selected: 1,
        };

        assert_eq!(
            handle_key_event(key(KeyCode::Esc), &app),
            Action::ClosePopup
        );
        assert_eq!(
            handle_key_event(key(KeyCode::Enter), &app),
            Action::SetNamespace(Some("default".to_string()))
        );
    }

    #[test]
    fn test_handle_key_event_details_popup() {
        use crate::kubernetes::resources::{Kustomization, ResourceStatus};

        let mut app = App::for_testing(
            crate::app::state::Tab::Kustomizations,
            vec![],
            vec![],
            vec![],
        );
        app.popup = Popup::ResourceDetails {
            resource: Box::new(Kustomization {
                name: "test".to_string(),
                namespace: "ns".to_string(),
                status: ResourceStatus::Ready,
                status_message: "OK".to_string(),
                revision: None,
                suspended: false,
                source_ref: "".to_string(),
                path: "".to_string(),
            }),
        };

        assert_eq!(
            handle_key_event(key(KeyCode::Esc), &app),
            Action::ClosePopup
        );
        assert_eq!(
            handle_key_event(key(KeyCode::Enter), &app),
            Action::ClosePopup
        );
    }

    #[test]
    fn test_handle_key_event_reconciling_popup() {
        let mut app = App::for_testing(
            crate::app::state::Tab::Kustomizations,
            vec![],
            vec![],
            vec![],
        );
        app.popup = Popup::Reconciling {
            name: "test".to_string(),
            namespace: "ns".to_string(),
        };

        assert_eq!(
            handle_key_event(key(KeyCode::Char('q')), &app),
            Action::Quit
        );
        assert_eq!(handle_key_event(key(KeyCode::Esc), &app), Action::Noop);
    }

    #[test]
    fn test_handle_key_event_error_popup() {
        let mut app = App::for_testing(
            crate::app::state::Tab::Kustomizations,
            vec![],
            vec![],
            vec![],
        );
        app.popup = Popup::Error {
            message: "test error".to_string(),
        };

        assert_eq!(
            handle_key_event(key(KeyCode::Esc), &app),
            Action::ClosePopup
        );
        assert_eq!(
            handle_key_event(key(KeyCode::Enter), &app),
            Action::ClosePopup
        );
        assert_eq!(
            handle_key_event(key(KeyCode::Char('q')), &app),
            Action::Quit
        );
    }
}
