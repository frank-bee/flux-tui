//! Flux CD resource definitions

mod helmchart;
mod helmrelease;
mod kustomization;

pub use helmchart::HelmChart;
pub use helmrelease::HelmRelease;
pub use kustomization::Kustomization;

use std::fmt::Debug;

/// Status of a Flux resource
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceStatus {
    /// Resource is ready and reconciled
    Ready,
    /// Resource reconciliation failed
    Failed,
    /// Resource is currently reconciling
    Reconciling,
    /// Resource is suspended
    Suspended,
    /// Status is unknown
    Unknown,
}

/// Trait for Flux resources
pub trait FluxResource: Debug + Send + Sync {
    /// Get the resource name
    fn name(&self) -> &str;

    /// Get the resource namespace
    fn namespace(&self) -> &str;

    /// Get the resource kind (Kustomization, HelmRelease, HelmChart)
    fn kind(&self) -> &str;

    /// Get the current status
    fn status(&self) -> &ResourceStatus;

    /// Get the status message
    fn status_message(&self) -> &str;

    /// Check if the resource is ready
    fn is_ready(&self) -> bool {
        matches!(self.status(), ResourceStatus::Ready)
    }

    /// Check if the resource is suspended
    fn is_suspended(&self) -> bool;

    /// Get the current revision (if available)
    fn revision(&self) -> Option<&str>;
}

impl Clone for Box<dyn FluxResource> {
    fn clone(&self) -> Self {
        // This is a workaround - ideally we'd have Clone on the trait
        // but that makes it not object-safe
        panic!("Cannot clone boxed FluxResource");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_status_debug() {
        assert_eq!(format!("{:?}", ResourceStatus::Ready), "Ready");
        assert_eq!(format!("{:?}", ResourceStatus::Failed), "Failed");
        assert_eq!(format!("{:?}", ResourceStatus::Reconciling), "Reconciling");
        assert_eq!(format!("{:?}", ResourceStatus::Suspended), "Suspended");
        assert_eq!(format!("{:?}", ResourceStatus::Unknown), "Unknown");
    }

    #[test]
    fn test_resource_status_clone() {
        let status = ResourceStatus::Ready;
        let cloned = status.clone();
        assert_eq!(status, cloned);
    }

    #[test]
    fn test_resource_status_equality() {
        assert_eq!(ResourceStatus::Ready, ResourceStatus::Ready);
        assert_ne!(ResourceStatus::Ready, ResourceStatus::Failed);
    }

    #[test]
    fn test_is_ready_default_impl() {
        let k = Kustomization {
            name: "test".to_string(),
            namespace: "ns".to_string(),
            status: ResourceStatus::Ready,
            status_message: "OK".to_string(),
            revision: None,
            suspended: false,
            source_ref: "".to_string(),
            path: "".to_string(),
        };
        assert!(k.is_ready());

        let k_failed = Kustomization {
            name: "test".to_string(),
            namespace: "ns".to_string(),
            status: ResourceStatus::Failed,
            status_message: "Error".to_string(),
            revision: None,
            suspended: false,
            source_ref: "".to_string(),
            path: "".to_string(),
        };
        assert!(!k_failed.is_ready());
    }

    #[test]
    fn test_is_ready_all_statuses() {
        let test_cases = [
            (ResourceStatus::Ready, true),
            (ResourceStatus::Failed, false),
            (ResourceStatus::Reconciling, false),
            (ResourceStatus::Suspended, false),
            (ResourceStatus::Unknown, false),
        ];

        for (status, expected) in test_cases {
            let k = Kustomization {
                name: "test".to_string(),
                namespace: "ns".to_string(),
                status,
                status_message: "msg".to_string(),
                revision: None,
                suspended: false,
                source_ref: "".to_string(),
                path: "".to_string(),
            };
            assert_eq!(k.is_ready(), expected);
        }
    }

    #[test]
    fn test_flux_resource_trait_kustomization() {
        let k = Kustomization {
            name: "my-ks".to_string(),
            namespace: "flux-system".to_string(),
            status: ResourceStatus::Ready,
            status_message: "Applied".to_string(),
            revision: Some("main@sha256:abc123".to_string()),
            suspended: true,
            source_ref: "GitRepository/flux-system".to_string(),
            path: "./clusters".to_string(),
        };

        assert_eq!(k.name(), "my-ks");
        assert_eq!(k.namespace(), "flux-system");
        assert_eq!(k.kind(), "Kustomization");
        assert_eq!(*k.status(), ResourceStatus::Ready);
        assert_eq!(k.status_message(), "Applied");
        assert!(k.is_ready());
        assert!(k.is_suspended());
        assert_eq!(k.revision(), Some("main@sha256:abc123".as_ref()));
    }

    #[test]
    fn test_flux_resource_trait_helm_release() {
        let hr = HelmRelease {
            name: "my-release".to_string(),
            namespace: "default".to_string(),
            status: ResourceStatus::Failed,
            status_message: "upgrade failed".to_string(),
            chart: "nginx".to_string(),
            version: Some("1.2.3".to_string()),
            suspended: false,
            revision: Some("5".to_string()),
        };

        assert_eq!(hr.name(), "my-release");
        assert_eq!(hr.namespace(), "default");
        assert_eq!(hr.kind(), "HelmRelease");
        assert_eq!(*hr.status(), ResourceStatus::Failed);
        assert_eq!(hr.status_message(), "upgrade failed");
        assert!(!hr.is_ready());
        assert!(!hr.is_suspended());
        assert_eq!(hr.revision(), Some("5".as_ref()));
    }

    #[test]
    fn test_flux_resource_trait_helm_chart() {
        let hc = HelmChart {
            name: "my-chart".to_string(),
            namespace: "flux-system".to_string(),
            status: ResourceStatus::Reconciling,
            status_message: "pulling chart".to_string(),
            chart: "nginx".to_string(),
            version: Some("15.0.0".to_string()),
            source_ref: "HelmRepository/bitnami".to_string(),
            revision: None,
        };

        assert_eq!(hc.name(), "my-chart");
        assert_eq!(hc.namespace(), "flux-system");
        assert_eq!(hc.kind(), "HelmChart");
        assert_eq!(*hc.status(), ResourceStatus::Reconciling);
        assert_eq!(hc.status_message(), "pulling chart");
        assert!(!hc.is_ready());
        assert!(hc.revision().is_none());
    }
}
