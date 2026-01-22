//! HelmRelease resource definition

use super::{FluxResource, ResourceStatus};

/// Flux HelmRelease resource
#[derive(Debug, Clone)]
pub struct HelmRelease {
    /// Resource name
    pub name: String,

    /// Resource namespace
    pub namespace: String,

    /// Current status
    pub status: ResourceStatus,

    /// Status message
    pub status_message: String,

    /// Chart name
    pub chart: String,

    /// Chart version
    pub version: Option<String>,

    /// Whether the resource is suspended
    pub suspended: bool,

    /// Last applied revision
    pub revision: Option<String>,
}

impl HelmRelease {
    /// Create a new HelmRelease from raw K8s data
    pub fn from_kube(
        name: String,
        namespace: String,
        spec: &serde_json::Value,
        status: &serde_json::Value,
    ) -> Self {
        let suspended = spec
            .get("suspend")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let chart = spec
            .get("chart")
            .and_then(|c| c.get("spec"))
            .and_then(|s| s.get("chart"))
            .and_then(|c| c.as_str())
            .unwrap_or("unknown")
            .to_string();

        let version = spec
            .get("chart")
            .and_then(|c| c.get("spec"))
            .and_then(|s| s.get("version"))
            .and_then(|v| v.as_str())
            .map(String::from);

        let revision = status
            .get("lastAppliedRevision")
            .and_then(|r| r.as_str())
            .map(String::from);

        let (resource_status, status_message) = parse_status(status, suspended);

        Self {
            name,
            namespace,
            status: resource_status,
            status_message,
            chart,
            version,
            suspended,
            revision,
        }
    }
}

impl FluxResource for HelmRelease {
    fn name(&self) -> &str {
        &self.name
    }

    fn namespace(&self) -> &str {
        &self.namespace
    }

    fn kind(&self) -> &str {
        "HelmRelease"
    }

    fn status(&self) -> &ResourceStatus {
        &self.status
    }

    fn status_message(&self) -> &str {
        &self.status_message
    }

    fn is_suspended(&self) -> bool {
        self.suspended
    }

    fn revision(&self) -> Option<&str> {
        self.revision.as_deref()
    }
}

/// Parse the status conditions to determine resource status
fn parse_status(status: &serde_json::Value, suspended: bool) -> (ResourceStatus, String) {
    if suspended {
        return (ResourceStatus::Suspended, "Suspended".to_string());
    }

    let conditions = status.get("conditions").and_then(|c| c.as_array());

    if let Some(conditions) = conditions {
        for condition in conditions {
            let condition_type = condition.get("type").and_then(|t| t.as_str());
            let condition_status = condition.get("status").and_then(|s| s.as_str());
            let message = condition
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown")
                .to_string();

            if condition_type == Some("Ready") {
                match condition_status {
                    Some("True") => return (ResourceStatus::Ready, message),
                    Some("False") => {
                        let reason = condition.get("reason").and_then(|r| r.as_str());
                        if reason == Some("Progressing") || reason == Some("ArtifactFailed") {
                            return (ResourceStatus::Reconciling, message);
                        }
                        return (ResourceStatus::Failed, message);
                    }
                    Some("Unknown") => return (ResourceStatus::Reconciling, message),
                    _ => {}
                }
            }

            if condition_type == Some("Reconciling") && condition_status == Some("True") {
                return (ResourceStatus::Reconciling, message);
            }
        }
    }

    (ResourceStatus::Unknown, "Status unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_status_suspended() {
        let status = json!({});
        let (resource_status, message) = parse_status(&status, true);
        assert_eq!(resource_status, ResourceStatus::Suspended);
        assert_eq!(message, "Suspended");
    }

    #[test]
    fn test_parse_status_ready() {
        let status = json!({
            "conditions": [
                {
                    "type": "Ready",
                    "status": "True",
                    "message": "Release reconciliation succeeded"
                }
            ]
        });
        let (resource_status, message) = parse_status(&status, false);
        assert_eq!(resource_status, ResourceStatus::Ready);
        assert_eq!(message, "Release reconciliation succeeded");
    }

    #[test]
    fn test_parse_status_failed() {
        let status = json!({
            "conditions": [
                {
                    "type": "Ready",
                    "status": "False",
                    "reason": "InstallFailed",
                    "message": "Helm install failed"
                }
            ]
        });
        let (resource_status, message) = parse_status(&status, false);
        assert_eq!(resource_status, ResourceStatus::Failed);
        assert_eq!(message, "Helm install failed");
    }

    #[test]
    fn test_parse_status_progressing() {
        let status = json!({
            "conditions": [
                {
                    "type": "Ready",
                    "status": "False",
                    "reason": "Progressing",
                    "message": "Helm upgrade in progress"
                }
            ]
        });
        let (resource_status, message) = parse_status(&status, false);
        assert_eq!(resource_status, ResourceStatus::Reconciling);
        assert_eq!(message, "Helm upgrade in progress");
    }

    #[test]
    fn test_parse_status_artifact_failed() {
        let status = json!({
            "conditions": [
                {
                    "type": "Ready",
                    "status": "False",
                    "reason": "ArtifactFailed",
                    "message": "Waiting for HelmChart"
                }
            ]
        });
        let (resource_status, message) = parse_status(&status, false);
        assert_eq!(resource_status, ResourceStatus::Reconciling);
        assert_eq!(message, "Waiting for HelmChart");
    }

    #[test]
    fn test_parse_status_unknown_ready_status() {
        let status = json!({
            "conditions": [
                {
                    "type": "Ready",
                    "status": "Unknown",
                    "message": "Waiting for reconciliation"
                }
            ]
        });
        let (resource_status, message) = parse_status(&status, false);
        assert_eq!(resource_status, ResourceStatus::Reconciling);
        assert_eq!(message, "Waiting for reconciliation");
    }

    #[test]
    fn test_parse_status_reconciling_condition() {
        let status = json!({
            "conditions": [
                {
                    "type": "Reconciling",
                    "status": "True",
                    "message": "Reconciling"
                }
            ]
        });
        let (resource_status, message) = parse_status(&status, false);
        assert_eq!(resource_status, ResourceStatus::Reconciling);
        assert_eq!(message, "Reconciling");
    }

    #[test]
    fn test_parse_status_no_conditions() {
        let status = json!({});
        let (resource_status, message) = parse_status(&status, false);
        assert_eq!(resource_status, ResourceStatus::Unknown);
        assert_eq!(message, "Status unknown");
    }

    #[test]
    fn test_helmrelease_from_kube_basic() {
        let spec = json!({
            "chart": {
                "spec": {
                    "chart": "nginx",
                    "version": "1.2.3"
                }
            }
        });
        let status = json!({
            "lastAppliedRevision": "1.2.3",
            "conditions": [
                {
                    "type": "Ready",
                    "status": "True",
                    "message": "Release reconciliation succeeded"
                }
            ]
        });

        let hr = HelmRelease::from_kube(
            "my-nginx".to_string(),
            "default".to_string(),
            &spec,
            &status,
        );

        assert_eq!(hr.name, "my-nginx");
        assert_eq!(hr.namespace, "default");
        assert_eq!(hr.chart, "nginx");
        assert_eq!(hr.version, Some("1.2.3".to_string()));
        assert_eq!(hr.revision, Some("1.2.3".to_string()));
        assert_eq!(hr.status, ResourceStatus::Ready);
        assert!(!hr.suspended);
    }

    #[test]
    fn test_helmrelease_from_kube_suspended() {
        let spec = json!({
            "suspend": true,
            "chart": {
                "spec": {
                    "chart": "redis"
                }
            }
        });
        let status = json!({});

        let hr = HelmRelease::from_kube(
            "my-redis".to_string(),
            "default".to_string(),
            &spec,
            &status,
        );

        assert!(hr.suspended);
        assert_eq!(hr.status, ResourceStatus::Suspended);
    }

    #[test]
    fn test_helmrelease_from_kube_defaults() {
        let spec = json!({});
        let status = json!({});

        let hr =
            HelmRelease::from_kube("minimal".to_string(), "default".to_string(), &spec, &status);

        assert_eq!(hr.chart, "unknown");
        assert_eq!(hr.version, None);
        assert_eq!(hr.revision, None);
        assert!(!hr.suspended);
    }

    #[test]
    fn test_helmrelease_flux_resource_trait() {
        let hr = HelmRelease {
            name: "test".to_string(),
            namespace: "ns".to_string(),
            status: ResourceStatus::Ready,
            status_message: "OK".to_string(),
            chart: "nginx".to_string(),
            version: Some("1.0.0".to_string()),
            suspended: false,
            revision: Some("rev".to_string()),
        };

        assert_eq!(hr.name(), "test");
        assert_eq!(hr.namespace(), "ns");
        assert_eq!(hr.kind(), "HelmRelease");
        assert_eq!(hr.status(), &ResourceStatus::Ready);
        assert_eq!(hr.status_message(), "OK");
        assert!(hr.is_ready());
        assert!(!hr.is_suspended());
        assert_eq!(hr.revision(), Some("rev"));
    }
}
