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
    pub fn from_kube(
        name: String,
        namespace: String,
        spec: &serde_json::Value,
        status: &serde_json::Value,
    ) -> Self {
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
            .map(|sr| {
                let kind = sr
                    .get("kind")
                    .and_then(|k| k.as_str())
                    .unwrap_or("HelmRepository");
                let name = sr.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                format!("{}/{}", kind, name)
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_status_ready() {
        let status = json!({
            "conditions": [
                {
                    "type": "Ready",
                    "status": "True",
                    "message": "stored artifact: revision '1.2.3'"
                }
            ]
        });
        let (resource_status, message) = parse_status(&status);
        assert_eq!(resource_status, ResourceStatus::Ready);
        assert_eq!(message, "stored artifact: revision '1.2.3'");
    }

    #[test]
    fn test_parse_status_failed() {
        let status = json!({
            "conditions": [
                {
                    "type": "Ready",
                    "status": "False",
                    "message": "chart pull error"
                }
            ]
        });
        let (resource_status, message) = parse_status(&status);
        assert_eq!(resource_status, ResourceStatus::Failed);
        assert_eq!(message, "chart pull error");
    }

    #[test]
    fn test_parse_status_unknown() {
        let status = json!({
            "conditions": [
                {
                    "type": "Ready",
                    "status": "Unknown",
                    "message": "Waiting for pull"
                }
            ]
        });
        let (resource_status, message) = parse_status(&status);
        assert_eq!(resource_status, ResourceStatus::Reconciling);
        assert_eq!(message, "Waiting for pull");
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
        let (resource_status, message) = parse_status(&status);
        assert_eq!(resource_status, ResourceStatus::Reconciling);
        assert_eq!(message, "Reconciling");
    }

    #[test]
    fn test_parse_status_no_conditions() {
        let status = json!({});
        let (resource_status, message) = parse_status(&status);
        assert_eq!(resource_status, ResourceStatus::Unknown);
        assert_eq!(message, "Status unknown");
    }

    #[test]
    fn test_helmchart_from_kube_basic() {
        let spec = json!({
            "chart": "nginx",
            "version": "1.2.3",
            "sourceRef": {
                "kind": "HelmRepository",
                "name": "bitnami"
            }
        });
        let status = json!({
            "artifact": {
                "revision": "1.2.3"
            },
            "conditions": [
                {
                    "type": "Ready",
                    "status": "True",
                    "message": "stored artifact"
                }
            ]
        });

        let hc = HelmChart::from_kube(
            "default-nginx".to_string(),
            "flux-system".to_string(),
            &spec,
            &status,
        );

        assert_eq!(hc.name, "default-nginx");
        assert_eq!(hc.namespace, "flux-system");
        assert_eq!(hc.chart, "nginx");
        assert_eq!(hc.version, Some("1.2.3".to_string()));
        assert_eq!(hc.source_ref, "HelmRepository/bitnami");
        assert_eq!(hc.revision, Some("1.2.3".to_string()));
        assert_eq!(hc.status, ResourceStatus::Ready);
    }

    #[test]
    fn test_helmchart_from_kube_defaults() {
        let spec = json!({});
        let status = json!({});

        let hc = HelmChart::from_kube("minimal".to_string(), "default".to_string(), &spec, &status);

        assert_eq!(hc.chart, "unknown");
        assert_eq!(hc.version, None);
        assert_eq!(hc.source_ref, "unknown");
        assert_eq!(hc.revision, None);
    }

    #[test]
    fn test_helmchart_flux_resource_trait() {
        let hc = HelmChart {
            name: "test".to_string(),
            namespace: "ns".to_string(),
            status: ResourceStatus::Ready,
            status_message: "OK".to_string(),
            chart: "nginx".to_string(),
            version: Some("1.0.0".to_string()),
            source_ref: "HelmRepository/bitnami".to_string(),
            revision: Some("rev".to_string()),
        };

        assert_eq!(hc.name(), "test");
        assert_eq!(hc.namespace(), "ns");
        assert_eq!(hc.kind(), "HelmChart");
        assert_eq!(hc.status(), &ResourceStatus::Ready);
        assert_eq!(hc.status_message(), "OK");
        assert!(hc.is_ready());
        assert!(!hc.is_suspended()); // HelmCharts always return false
        assert_eq!(hc.revision(), Some("rev"));
    }
}
