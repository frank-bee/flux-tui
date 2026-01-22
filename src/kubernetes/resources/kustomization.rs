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
    pub fn from_kube(name: String, namespace: String, spec: &serde_json::Value, status: &serde_json::Value) -> Self {
        let suspended = spec.get("suspend").and_then(|v| v.as_bool()).unwrap_or(false);

        let source_ref = spec
            .get("sourceRef")
            .and_then(|sr| {
                let kind = sr.get("kind").and_then(|k| k.as_str()).unwrap_or("GitRepository");
                let name = sr.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                Some(format!("{}/{}", kind, name))
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
            .map(|s| truncate_revision(s));

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
