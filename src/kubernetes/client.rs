//! Kubernetes client wrapper using kube-rs

use color_eyre::Result;
use k8s_openapi::api::core::v1::Namespace;
use kube::{
    api::{Api, DynamicObject, ListParams},
    discovery::{ApiCapabilities, ApiResource, Scope},
    Client, Config,
};

use super::resources::{HelmChart, HelmRelease, Kustomization};

/// API version and kind for Flux Kustomization
const KUSTOMIZATION_API: (&str, &str) = ("kustomize.toolkit.fluxcd.io/v1", "Kustomization");

/// API version and kind for Flux HelmRelease
const HELMRELEASE_API: (&str, &str) = ("helm.toolkit.fluxcd.io/v2", "HelmRelease");

/// API version and kind for Flux HelmChart
const HELMCHART_API: (&str, &str) = ("source.toolkit.fluxcd.io/v1", "HelmChart");

/// Kubernetes client wrapper for Flux resources
pub struct KubeClient {
    /// The underlying kube client
    client: Client,

    /// Current cluster name
    cluster_name: String,
}

impl KubeClient {
    /// Create a new KubeClient
    pub async fn new() -> Result<Self> {
        let config = Config::infer().await?;
        let cluster_name = config
            .cluster_url
            .host()
            .map(|h: &str| {
                // Try to extract a meaningful name from the URL
                h.split('.')
                    .next()
                    .unwrap_or(h)
                    .replace("api-", "")
                    .replace("-server", "")
            })
            .unwrap_or_else(|| "unknown".to_string());

        let client = Client::try_from(config)?;

        Ok(Self {
            client,
            cluster_name,
        })
    }

    /// Get the cluster name
    pub fn cluster_name(&self) -> &str {
        &self.cluster_name
    }

    /// List all namespaces
    pub async fn list_namespaces(&self) -> Result<Vec<String>> {
        let api: Api<Namespace> = Api::all(self.client.clone());
        let namespaces = api.list(&ListParams::default()).await?;

        Ok(namespaces
            .items
            .into_iter()
            .filter_map(|ns| ns.metadata.name)
            .collect())
    }

    /// List Kustomizations
    pub async fn list_kustomizations(&self, namespace: Option<&str>) -> Result<Vec<Kustomization>> {
        let api = self.create_dynamic_api(KUSTOMIZATION_API.0, KUSTOMIZATION_API.1, namespace);
        let list = api.list(&ListParams::default()).await?;

        Ok(list
            .items
            .into_iter()
            .filter_map(|obj| {
                let name = obj.metadata.name.clone()?;
                let ns = obj.metadata.namespace.clone().unwrap_or_default();
                let spec = obj.data.get("spec").cloned().unwrap_or_default();
                let status = obj.data.get("status").cloned().unwrap_or_default();

                Some(Kustomization::from_kube(name, ns, &spec, &status))
            })
            .collect())
    }

    /// List HelmReleases
    pub async fn list_helm_releases(&self, namespace: Option<&str>) -> Result<Vec<HelmRelease>> {
        let api = self.create_dynamic_api(HELMRELEASE_API.0, HELMRELEASE_API.1, namespace);
        let list = api.list(&ListParams::default()).await?;

        Ok(list
            .items
            .into_iter()
            .filter_map(|obj| {
                let name = obj.metadata.name.clone()?;
                let ns = obj.metadata.namespace.clone().unwrap_or_default();
                let spec = obj.data.get("spec").cloned().unwrap_or_default();
                let status = obj.data.get("status").cloned().unwrap_or_default();

                Some(HelmRelease::from_kube(name, ns, &spec, &status))
            })
            .collect())
    }

    /// List HelmCharts
    pub async fn list_helm_charts(&self, namespace: Option<&str>) -> Result<Vec<HelmChart>> {
        let api = self.create_dynamic_api(HELMCHART_API.0, HELMCHART_API.1, namespace);
        let list = api.list(&ListParams::default()).await?;

        Ok(list
            .items
            .into_iter()
            .filter_map(|obj| {
                let name = obj.metadata.name.clone()?;
                let ns = obj.metadata.namespace.clone().unwrap_or_default();
                let spec = obj.data.get("spec").cloned().unwrap_or_default();
                let status = obj.data.get("status").cloned().unwrap_or_default();

                Some(HelmChart::from_kube(name, ns, &spec, &status))
            })
            .collect())
    }

    /// Create a dynamic API for a custom resource
    fn create_dynamic_api(
        &self,
        api_version: &str,
        kind: &str,
        namespace: Option<&str>,
    ) -> Api<DynamicObject> {
        let (group, version) = parse_api_version(api_version);

        let ar = ApiResource {
            group: group.to_string(),
            version: version.to_string(),
            kind: kind.to_string(),
            api_version: api_version.to_string(),
            plural: format!("{}s", kind.to_lowercase()),
        };

        let _caps = ApiCapabilities {
            scope: Scope::Namespaced,
            subresources: vec![],
            operations: vec![],
        };

        match namespace {
            Some(ns) => Api::namespaced_with(self.client.clone(), ns, &ar),
            None => Api::all_with(self.client.clone(), &ar),
        }
    }
}

/// Parse an API version string into group and version
fn parse_api_version(api_version: &str) -> (&str, &str) {
    if let Some(idx) = api_version.rfind('/') {
        (&api_version[..idx], &api_version[idx + 1..])
    } else {
        ("", api_version)
    }
}
