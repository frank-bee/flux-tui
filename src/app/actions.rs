//! Application actions (messages in The Elm Architecture)
//!
//! These actions represent all possible state transitions in the application.

/// Actions that can be performed in the application
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Quit the application
    Quit,

    /// Navigate to the next tab
    NextTab,

    /// Navigate to the previous tab
    PreviousTab,

    /// Move selection up in the current list
    Up,

    /// Move selection down in the current list
    Down,

    /// Move to the top of the list
    Top,

    /// Move to the bottom of the list
    Bottom,

    /// Select/Enter - view details or confirm action
    Select,

    /// Reconcile the selected resource
    Reconcile,

    /// Reconcile the selected resource with its source
    ReconcileWithSource,

    /// Open namespace filter popup
    FilterNamespace,

    /// Set namespace filter (None = all namespaces)
    SetNamespace(Option<String>),

    /// Close any open popup
    ClosePopup,

    /// Refresh data manually
    Refresh,

    /// Toggle suspend on selected resource
    ToggleSuspend,

    /// No operation (used for unhandled keys)
    Noop,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_debug() {
        assert_eq!(format!("{:?}", Action::Quit), "Quit");
        assert_eq!(format!("{:?}", Action::NextTab), "NextTab");
        assert_eq!(format!("{:?}", Action::PreviousTab), "PreviousTab");
        assert_eq!(format!("{:?}", Action::Up), "Up");
        assert_eq!(format!("{:?}", Action::Down), "Down");
        assert_eq!(format!("{:?}", Action::Top), "Top");
        assert_eq!(format!("{:?}", Action::Bottom), "Bottom");
        assert_eq!(format!("{:?}", Action::Select), "Select");
        assert_eq!(format!("{:?}", Action::Reconcile), "Reconcile");
        assert_eq!(
            format!("{:?}", Action::ReconcileWithSource),
            "ReconcileWithSource"
        );
        assert_eq!(format!("{:?}", Action::FilterNamespace), "FilterNamespace");
        assert_eq!(format!("{:?}", Action::ClosePopup), "ClosePopup");
        assert_eq!(format!("{:?}", Action::Refresh), "Refresh");
        assert_eq!(format!("{:?}", Action::ToggleSuspend), "ToggleSuspend");
        assert_eq!(format!("{:?}", Action::Noop), "Noop");
    }

    #[test]
    fn test_action_clone() {
        let action = Action::Quit;
        let cloned = action.clone();
        assert_eq!(action, cloned);
    }

    #[test]
    fn test_action_equality() {
        assert_eq!(Action::Quit, Action::Quit);
        assert_ne!(Action::Quit, Action::Noop);
    }

    #[test]
    fn test_set_namespace_action() {
        let action1 = Action::SetNamespace(None);
        let action2 = Action::SetNamespace(Some("default".to_string()));
        let action3 = Action::SetNamespace(Some("default".to_string()));

        assert_ne!(action1, action2);
        assert_eq!(action2, action3);

        if let Action::SetNamespace(ns) = action1 {
            assert!(ns.is_none());
        } else {
            panic!("Expected SetNamespace");
        }

        if let Action::SetNamespace(ns) = action2 {
            assert_eq!(ns, Some("default".to_string()));
        } else {
            panic!("Expected SetNamespace");
        }
    }
}
