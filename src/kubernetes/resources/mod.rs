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
