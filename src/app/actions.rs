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
