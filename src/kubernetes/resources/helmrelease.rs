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
    pub fn from_kube(name: String, namespace: String, spec: &serde_json::Value, status: &serde_json::Value) -> Self {
        let suspended = spec.get("suspend").and_then(|v| v.as_bool()).unwrap_or(false);

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
