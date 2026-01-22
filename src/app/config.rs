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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.kubeconfig.is_none());
        assert!(config.context.is_none());
        assert!(config.namespace.is_none());
        assert_eq!(config.refresh_interval, 5);
    }

    #[test]
    fn test_config_debug() {
        let config = Config::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("Config"));
    }

    #[test]
    fn test_config_clone() {
        let config = Config {
            kubeconfig: Some(PathBuf::from("/path/to/kubeconfig")),
            context: Some("my-context".to_string()),
            namespace: Some("default".to_string()),
            refresh_interval: 10,
        };
        let cloned = config.clone();
        assert_eq!(config.kubeconfig, cloned.kubeconfig);
        assert_eq!(config.context, cloned.context);
        assert_eq!(config.namespace, cloned.namespace);
        assert_eq!(config.refresh_interval, cloned.refresh_interval);
    }
}
