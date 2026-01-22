//! Application configuration

use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Config {
    /// Path to kubeconfig file (None = use default)
    pub kubeconfig: Option<PathBuf>,

    /// Kubernetes context to use (None = use current context)
    pub context: Option<String>,

    /// Default namespace filter (None = all namespaces)
    pub namespace: Option<String>,

    /// Auto-refresh interval in seconds
    pub refresh_interval: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            kubeconfig: None,
            context: None,
            namespace: None,
            refresh_interval: 5,
        }
    }
}
