//! Kustomization resource definition

use super::{FluxResource, ResourceStatus};

/// Flux Kustomization resource
#[derive(Debug, Clone)]
pub struct Kustomization {
    /// Resource name
    pub name: String,

    /// Resource namespace
    pub namespace: String,

    /// Current status
    pub status: ResourceStatus,

    /// Status message
    pub status_message: String,

    /// Current revision
    pub revision: Option<String>,

    /// Whether the resource is suspended
    pub suspended: bool,

    /// Source reference
    #[allow(dead_code)]
    pub source_ref: String,

    /// Path within the source
    #[allow(dead_code)]
    pub path: String,
}

impl Kustomization {
    /// Create a new Kustomization from raw K8s data
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

        let source_ref = spec
            .get("sourceRef")
            .map(|sr| {
                let kind = sr
                    .get("kind")
                    .and_then(|k| k.as_str())
                    .unwrap_or("GitRepository");
                let name = sr.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                format!("{}/{}", kind, name)
            })
            .unwrap_or_else(|| "unknown".to_string());

        let path = spec
            .get("path")
            .and_then(|p| p.as_str())
            .unwrap_or("./")
            .to_string();

        let revision = status
            .get("lastAppliedRevision")
            .and_then(|r| r.as_str())
            .map(truncate_revision);

        let (resource_status, status_message) = parse_status(status, suspended);

        Self {
            name,
            namespace,
            status: resource_status,
            status_message,
            revision,
            suspended,
            source_ref,
            path,
        }
    }
}

impl FluxResource for Kustomization {
    fn name(&self) -> &str {
        &self.name
    }

    fn namespace(&self) -> &str {
        &self.namespace
    }

    fn kind(&self) -> &str {
        "Kustomization"
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
        // Look for Ready condition
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
                        if reason == Some("Progressing") {
                            return (ResourceStatus::Reconciling, message);
                        }
                        return (ResourceStatus::Failed, message);
                    }
                    Some("Unknown") => return (ResourceStatus::Reconciling, message),
                    _ => {}
                }
            }

            // Check for Reconciling condition
            if condition_type == Some("Reconciling") && condition_status == Some("True") {
                return (ResourceStatus::Reconciling, message);
            }
        }
    }

    (ResourceStatus::Unknown, "Status unknown".to_string())
}

/// Truncate git revision to a readable format
fn truncate_revision(revision: &str) -> String {
    if revision.contains('@') {
        // Format: branch@sha
        let parts: Vec<&str> = revision.split('@').collect();
        if parts.len() == 2 {
            let branch = parts[0];
            let sha = &parts[1][..7.min(parts[1].len())];
            return format!("{}@{}", branch, sha);
        }
    }
    // Just return first 12 chars
    revision.chars().take(12).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_truncate_revision_with_branch_and_sha() {
        let revision = "main@abc1234567890";
        assert_eq!(truncate_revision(revision), "main@abc1234");
    }

    #[test]
    fn test_truncate_revision_short_sha() {
        let revision = "main@abc";
        assert_eq!(truncate_revision(revision), "main@abc");
    }

    #[test]
    fn test_truncate_revision_no_at_sign() {
        let revision = "abc1234567890xyz";
        assert_eq!(truncate_revision(revision), "abc123456789");
    }

    #[test]
    fn test_truncate_revision_short_no_at() {
        let revision = "short";
        assert_eq!(truncate_revision(revision), "short");
    }

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
                    "message": "Applied revision: main@abc1234"
                }
            ]
        });
        let (resource_status, message) = parse_status(&status, false);
        assert_eq!(resource_status, ResourceStatus::Ready);
        assert_eq!(message, "Applied revision: main@abc1234");
    }

    #[test]
    fn test_parse_status_failed() {
        let status = json!({
            "conditions": [
                {
                    "type": "Ready",
                    "status": "False",
                    "reason": "ReconciliationFailed",
                    "message": "kustomization error"
                }
            ]
        });
        let (resource_status, message) = parse_status(&status, false);
        assert_eq!(resource_status, ResourceStatus::Failed);
        assert_eq!(message, "kustomization error");
    }

    #[test]
    fn test_parse_status_progressing() {
        let status = json!({
            "conditions": [
                {
                    "type": "Ready",
                    "status": "False",
                    "reason": "Progressing",
                    "message": "Reconciliation in progress"
                }
            ]
        });
        let (resource_status, message) = parse_status(&status, false);
        assert_eq!(resource_status, ResourceStatus::Reconciling);
        assert_eq!(message, "Reconciliation in progress");
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
    fn test_parse_status_empty_conditions() {
        let status = json!({"conditions": []});
        let (resource_status, message) = parse_status(&status, false);
        assert_eq!(resource_status, ResourceStatus::Unknown);
        assert_eq!(message, "Status unknown");
    }

    #[test]
    fn test_kustomization_from_kube_basic() {
        let spec = json!({
            "sourceRef": {
                "kind": "GitRepository",
                "name": "my-repo"
            },
            "path": "./clusters/production"
        });
        let status = json!({
            "lastAppliedRevision": "main@abc1234567890",
            "conditions": [
                {
                    "type": "Ready",
                    "status": "True",
                    "message": "Applied"
                }
            ]
        });

        let k = Kustomization::from_kube(
            "my-app".to_string(),
            "flux-system".to_string(),
            &spec,
            &status,
        );

        assert_eq!(k.name, "my-app");
        assert_eq!(k.namespace, "flux-system");
        assert_eq!(k.status, ResourceStatus::Ready);
        assert_eq!(k.revision, Some("main@abc1234".to_string()));
        assert!(!k.suspended);
    }

    #[test]
    fn test_kustomization_from_kube_suspended() {
        let spec = json!({
            "suspend": true,
            "sourceRef": {
                "name": "my-repo"
            },
            "path": "./"
        });
        let status = json!({});

        let k = Kustomization::from_kube(
            "suspended-app".to_string(),
            "default".to_string(),
            &spec,
            &status,
        );

        assert!(k.suspended);
        assert_eq!(k.status, ResourceStatus::Suspended);
    }

    #[test]
    fn test_kustomization_from_kube_defaults() {
        let spec = json!({});
        let status = json!({});

        let k =
            Kustomization::from_kube("minimal".to_string(), "default".to_string(), &spec, &status);

        assert_eq!(k.source_ref, "unknown");
        assert_eq!(k.path, "./");
        assert_eq!(k.revision, None);
        assert!(!k.suspended);
    }

    #[test]
    fn test_kustomization_flux_resource_trait() {
        let k = Kustomization {
            name: "test".to_string(),
            namespace: "ns".to_string(),
            status: ResourceStatus::Ready,
            status_message: "OK".to_string(),
            revision: Some("rev".to_string()),
            suspended: false,
            source_ref: "GitRepository/test".to_string(),
            path: "./".to_string(),
        };

        assert_eq!(k.name(), "test");
        assert_eq!(k.namespace(), "ns");
        assert_eq!(k.kind(), "Kustomization");
        assert_eq!(k.status(), &ResourceStatus::Ready);
        assert_eq!(k.status_message(), "OK");
        assert!(k.is_ready());
        assert!(!k.is_suspended());
        assert_eq!(k.revision(), Some("rev"));
    }
}
