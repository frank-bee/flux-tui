//! HelmChart resource definition

use super::{FluxResource, ResourceStatus};

/// Flux HelmChart resource
#[derive(Debug, Clone)]
pub struct HelmChart {
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

    /// Source reference (HelmRepository name)
    pub source_ref: String,

    /// Last fetched revision
    pub revision: Option<String>,
}

impl HelmChart {
    /// Create a new HelmChart from raw K8s data
    pub fn from_kube(name: String, namespace: String, spec: &serde_json::Value, status: &serde_json::Value) -> Self {
        let chart = spec
            .get("chart")
            .and_then(|c| c.as_str())
            .unwrap_or("unknown")
            .to_string();

        let version = spec
            .get("version")
            .and_then(|v| v.as_str())
            .map(String::from);

        let source_ref = spec
            .get("sourceRef")
            .and_then(|sr| {
                let kind = sr.get("kind").and_then(|k| k.as_str()).unwrap_or("HelmRepository");
                let name = sr.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                Some(format!("{}/{}", kind, name))
            })
            .unwrap_or_else(|| "unknown".to_string());

        let revision = status
            .get("artifact")
            .and_then(|a| a.get("revision"))
            .and_then(|r| r.as_str())
            .map(String::from);

        let (resource_status, status_message) = parse_status(status);

        Self {
            name,
            namespace,
            status: resource_status,
            status_message,
            chart,
            version,
            source_ref,
            revision,
        }
    }
}

impl FluxResource for HelmChart {
    fn name(&self) -> &str {
        &self.name
    }

    fn namespace(&self) -> &str {
        &self.namespace
    }

    fn kind(&self) -> &str {
        "HelmChart"
    }

    fn status(&self) -> &ResourceStatus {
        &self.status
    }

    fn status_message(&self) -> &str {
        &self.status_message
    }

    fn is_suspended(&self) -> bool {
        false // HelmCharts don't have suspend field
    }

    fn revision(&self) -> Option<&str> {
        self.revision.as_deref()
    }
}

/// Parse the status conditions to determine resource status
fn parse_status(status: &serde_json::Value) -> (ResourceStatus, String) {
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
                    Some("False") => return (ResourceStatus::Failed, message),
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
