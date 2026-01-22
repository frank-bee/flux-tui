//! Application state (Model in The Elm Architecture)

use color_eyre::Result;

use crate::kubernetes::{
    client::KubeClient,
    resources::{FluxResource, HelmChart, HelmRelease, Kustomization},
};

use super::actions::Action;

/// The currently active tab
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Kustomizations,
    HelmReleases,
    HelmCharts,
}

impl Tab {
    /// Get all tabs in order
    pub fn all() -> &'static [Tab] {
        &[Tab::Kustomizations, Tab::HelmReleases, Tab::HelmCharts]
    }

    /// Get the display name for this tab
    pub fn name(&self) -> &'static str {
        match self {
            Tab::Kustomizations => "Kustomizations",
            Tab::HelmReleases => "HelmReleases",
            Tab::HelmCharts => "HelmCharts",
        }
    }

    /// Get the next tab
    pub fn next(&self) -> Tab {
        match self {
            Tab::Kustomizations => Tab::HelmReleases,
            Tab::HelmReleases => Tab::HelmCharts,
            Tab::HelmCharts => Tab::Kustomizations,
        }
    }

    /// Get the previous tab
    pub fn previous(&self) -> Tab {
        match self {
            Tab::Kustomizations => Tab::HelmCharts,
            Tab::HelmReleases => Tab::Kustomizations,
            Tab::HelmCharts => Tab::HelmReleases,
        }
    }
}

/// Popup state
#[derive(Debug, Clone, Default)]
pub enum Popup {
    #[default]
    None,
    NamespaceFilter {
        namespaces: Vec<String>,
        selected: usize,
    },
    ResourceDetails {
        resource: Box<dyn FluxResource>,
    },
    Reconciling {
        name: String,
        namespace: String,
    },
    Error {
        message: String,
    },
}

/// Main application state
pub struct App {
    /// Kubernetes client (None only in test mode)
    #[cfg(not(test))]
    pub client: KubeClient,
    #[cfg(test)]
    pub client: Option<KubeClient>,

    /// Current active tab
    pub tab: Tab,

    /// Kustomization resources
    pub kustomizations: Vec<Kustomization>,

    /// HelmRelease resources
    pub helm_releases: Vec<HelmRelease>,

    /// HelmChart resources
    pub helm_charts: Vec<HelmChart>,

    /// Currently selected index for each tab
    pub selected: [usize; 3],

    /// Current namespace filter (None = all namespaces)
    pub namespace_filter: Option<String>,

    /// All available namespaces
    pub namespaces: Vec<String>,

    /// Current popup state
    pub popup: Popup,

    /// Loading state
    pub loading: bool,

    /// Last error message
    pub last_error: Option<String>,

    /// Current cluster name
    pub cluster_name: String,
}

impl App {
    /// Create a new App instance
    #[cfg(not(test))]
    pub async fn new() -> Result<Self> {
        let client = KubeClient::new().await?;
        let cluster_name = client.cluster_name().to_string();

        let mut app = Self {
            client,
            tab: Tab::default(),
            kustomizations: Vec::new(),
            helm_releases: Vec::new(),
            helm_charts: Vec::new(),
            selected: [0; 3],
            namespace_filter: None,
            namespaces: Vec::new(),
            popup: Popup::None,
            loading: true,
            last_error: None,
            cluster_name,
        };

        // Initial data load
        app.refresh_data().await?;

        Ok(app)
    }

    /// Create a new App instance (test version)
    #[cfg(test)]
    pub async fn new() -> Result<Self> {
        let client = KubeClient::new().await?;
        let cluster_name = client.cluster_name().to_string();

        let mut app = Self {
            client: Some(client),
            tab: Tab::default(),
            kustomizations: Vec::new(),
            helm_releases: Vec::new(),
            helm_charts: Vec::new(),
            selected: [0; 3],
            namespace_filter: None,
            namespaces: Vec::new(),
            popup: Popup::None,
            loading: true,
            last_error: None,
            cluster_name,
        };

        // Initial data load
        app.refresh_data().await?;

        Ok(app)
    }

    /// Create an App for testing without requiring a KubeClient
    #[cfg(test)]
    pub fn for_testing(
        tab: Tab,
        kustomizations: Vec<Kustomization>,
        helm_releases: Vec<HelmRelease>,
        helm_charts: Vec<HelmChart>,
    ) -> Self {
        Self {
            client: None,
            tab,
            kustomizations,
            helm_releases,
            helm_charts,
            selected: [0; 3],
            namespace_filter: None,
            namespaces: Vec::new(),
            popup: Popup::None,
            loading: false,
            last_error: None,
            cluster_name: "test-cluster".to_string(),
        }
    }

    /// Refresh all data from the cluster
    #[cfg(not(test))]
    pub async fn refresh_data(&mut self) -> Result<()> {
        self.loading = true;

        // Fetch all resources in parallel
        let ns = self.namespace_filter.as_deref();

        match tokio::try_join!(
            self.client.list_kustomizations(ns),
            self.client.list_helm_releases(ns),
            self.client.list_helm_charts(ns),
            self.client.list_namespaces(),
        ) {
            Ok((kustomizations, helm_releases, helm_charts, namespaces)) => {
                self.kustomizations = kustomizations;
                self.helm_releases = helm_releases;
                self.helm_charts = helm_charts;
                self.namespaces = namespaces;
                self.last_error = None;
            }
            Err(e) => {
                self.last_error = Some(format!("Failed to fetch resources: {e}"));
            }
        }

        self.loading = false;
        Ok(())
    }

    /// Refresh all data from the cluster (test version)
    #[cfg(test)]
    pub async fn refresh_data(&mut self) -> Result<()> {
        self.loading = true;

        // In test mode, only refresh if we have a real client
        if let Some(ref client) = self.client {
            let ns = self.namespace_filter.as_deref();

            match tokio::try_join!(
                client.list_kustomizations(ns),
                client.list_helm_releases(ns),
                client.list_helm_charts(ns),
                client.list_namespaces(),
            ) {
                Ok((kustomizations, helm_releases, helm_charts, namespaces)) => {
                    self.kustomizations = kustomizations;
                    self.helm_releases = helm_releases;
                    self.helm_charts = helm_charts;
                    self.namespaces = namespaces;
                    self.last_error = None;
                }
                Err(e) => {
                    self.last_error = Some(format!("Failed to fetch resources: {e}"));
                }
            }
        }

        self.loading = false;
        Ok(())
    }

    /// Get the current tab index
    pub fn tab_index(&self) -> usize {
        match self.tab {
            Tab::Kustomizations => 0,
            Tab::HelmReleases => 1,
            Tab::HelmCharts => 2,
        }
    }

    /// Get the selected index for the current tab
    pub fn current_selected(&self) -> usize {
        self.selected[self.tab_index()]
    }

    /// Set the selected index for the current tab
    pub fn set_current_selected(&mut self, index: usize) {
        self.selected[self.tab_index()] = index;
    }

    /// Get the number of items in the current tab
    pub fn current_item_count(&self) -> usize {
        match self.tab {
            Tab::Kustomizations => self.kustomizations.len(),
            Tab::HelmReleases => self.helm_releases.len(),
            Tab::HelmCharts => self.helm_charts.len(),
        }
    }

    /// Update the application state based on an action
    pub async fn update(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Quit => {} // Handled in main loop
            Action::NextTab => {
                self.tab = self.tab.next();
            }
            Action::PreviousTab => {
                self.tab = self.tab.previous();
            }
            Action::Up => {
                let selected = self.current_selected();
                if selected > 0 {
                    self.set_current_selected(selected - 1);
                }
            }
            Action::Down => {
                let selected = self.current_selected();
                let count = self.current_item_count();
                if count > 0 && selected < count - 1 {
                    self.set_current_selected(selected + 1);
                }
            }
            Action::Top => {
                self.set_current_selected(0);
            }
            Action::Bottom => {
                let count = self.current_item_count();
                if count > 0 {
                    self.set_current_selected(count - 1);
                }
            }
            Action::Select => {
                // View details of selected resource
                if let Some(resource) = self.get_selected_resource() {
                    self.popup = Popup::ResourceDetails { resource };
                }
            }
            Action::Reconcile => {
                self.reconcile_selected(false).await?;
            }
            Action::ReconcileWithSource => {
                self.reconcile_selected(true).await?;
            }
            Action::FilterNamespace => {
                let mut namespaces = vec!["All namespaces".to_string()];
                namespaces.extend(self.namespaces.clone());
                self.popup = Popup::NamespaceFilter {
                    namespaces,
                    selected: 0,
                };
            }
            Action::SetNamespace(ns) => {
                self.namespace_filter = ns;
                self.popup = Popup::None;
                self.refresh_data().await?;
            }
            Action::ClosePopup => {
                self.popup = Popup::None;
            }
            Action::Refresh => {
                self.refresh_data().await?;
            }
            Action::ToggleSuspend => {
                self.toggle_suspend_selected().await?;
            }
            Action::Noop => {}
        }

        Ok(())
    }

    /// Get the currently selected resource (as a trait object)
    fn get_selected_resource(&self) -> Option<Box<dyn FluxResource>> {
        let selected = self.current_selected();
        match self.tab {
            Tab::Kustomizations => self
                .kustomizations
                .get(selected)
                .map(|k| Box::new(k.clone()) as Box<dyn FluxResource>),
            Tab::HelmReleases => self
                .helm_releases
                .get(selected)
                .map(|h| Box::new(h.clone()) as Box<dyn FluxResource>),
            Tab::HelmCharts => self
                .helm_charts
                .get(selected)
                .map(|h| Box::new(h.clone()) as Box<dyn FluxResource>),
        }
    }

    /// Reconcile the selected resource
    async fn reconcile_selected(&mut self, with_source: bool) -> Result<()> {
        let selected = self.current_selected();

        let (name, namespace, kind) = match self.tab {
            Tab::Kustomizations => {
                if let Some(k) = self.kustomizations.get(selected) {
                    (k.name.clone(), k.namespace.clone(), "kustomization")
                } else {
                    return Ok(());
                }
            }
            Tab::HelmReleases => {
                if let Some(h) = self.helm_releases.get(selected) {
                    (h.name.clone(), h.namespace.clone(), "helmrelease")
                } else {
                    return Ok(());
                }
            }
            Tab::HelmCharts => {
                if let Some(h) = self.helm_charts.get(selected) {
                    (h.name.clone(), h.namespace.clone(), "helmchart")
                } else {
                    return Ok(());
                }
            }
        };

        self.popup = Popup::Reconciling {
            name: name.clone(),
            namespace: namespace.clone(),
        };

        match crate::kubernetes::reconcile::reconcile(&name, &namespace, kind, with_source).await {
            Ok(_) => {
                self.popup = Popup::None;
                // Refresh to show updated status
                self.refresh_data().await?;
            }
            Err(e) => {
                self.popup = Popup::Error {
                    message: format!("Reconcile failed: {e}"),
                };
            }
        }

        Ok(())
    }

    /// Toggle suspend on the selected resource
    async fn toggle_suspend_selected(&mut self) -> Result<()> {
        let selected = self.current_selected();

        let (name, namespace, kind, is_suspended) = match self.tab {
            Tab::Kustomizations => {
                if let Some(k) = self.kustomizations.get(selected) {
                    (
                        k.name.clone(),
                        k.namespace.clone(),
                        "kustomization",
                        k.suspended,
                    )
                } else {
                    return Ok(());
                }
            }
            Tab::HelmReleases => {
                if let Some(h) = self.helm_releases.get(selected) {
                    (
                        h.name.clone(),
                        h.namespace.clone(),
                        "helmrelease",
                        h.suspended,
                    )
                } else {
                    return Ok(());
                }
            }
            Tab::HelmCharts => {
                // HelmCharts cannot be suspended directly
                return Ok(());
            }
        };

        match crate::kubernetes::reconcile::toggle_suspend(&name, &namespace, kind, is_suspended)
            .await
        {
            Ok(_) => {
                self.refresh_data().await?;
            }
            Err(e) => {
                self.popup = Popup::Error {
                    message: format!("Toggle suspend failed: {e}"),
                };
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kubernetes::resources::ResourceStatus;

    fn create_test_kustomization(name: &str, namespace: &str) -> Kustomization {
        Kustomization {
            name: name.to_string(),
            namespace: namespace.to_string(),
            status: ResourceStatus::Ready,
            status_message: "Applied".to_string(),
            revision: Some("main/abc123".to_string()),
            suspended: false,
            source_ref: "GitRepository/flux-system".to_string(),
            path: "./".to_string(),
        }
    }

    fn create_test_helm_release(name: &str, namespace: &str) -> HelmRelease {
        HelmRelease {
            name: name.to_string(),
            namespace: namespace.to_string(),
            status: ResourceStatus::Ready,
            status_message: "Release reconciled".to_string(),
            chart: "nginx".to_string(),
            version: Some("1.0.0".to_string()),
            suspended: false,
            revision: Some("1".to_string()),
        }
    }

    fn create_test_helm_chart(name: &str, namespace: &str) -> HelmChart {
        HelmChart {
            name: name.to_string(),
            namespace: namespace.to_string(),
            status: ResourceStatus::Ready,
            status_message: "Chart pulled".to_string(),
            chart: "nginx".to_string(),
            version: Some("1.0.0".to_string()),
            source_ref: "HelmRepository/bitnami".to_string(),
            revision: Some("1.0.0".to_string()),
        }
    }

    // ===== Tab Index Tests (using real App) =====

    #[test]
    fn test_tab_index_kustomizations() {
        let app = App::for_testing(Tab::Kustomizations, vec![], vec![], vec![]);
        assert_eq!(app.tab_index(), 0);
    }

    #[test]
    fn test_tab_index_helm_releases() {
        let app = App::for_testing(Tab::HelmReleases, vec![], vec![], vec![]);
        assert_eq!(app.tab_index(), 1);
    }

    #[test]
    fn test_tab_index_helm_charts() {
        let app = App::for_testing(Tab::HelmCharts, vec![], vec![], vec![]);
        assert_eq!(app.tab_index(), 2);
    }

    // ===== Selection Tests (using real App) =====

    #[test]
    fn test_current_selected_default() {
        let app = App::for_testing(Tab::Kustomizations, vec![], vec![], vec![]);
        assert_eq!(app.current_selected(), 0);
    }

    #[test]
    fn test_set_current_selected() {
        let mut app = App::for_testing(
            Tab::Kustomizations,
            vec![
                create_test_kustomization("ks1", "default"),
                create_test_kustomization("ks2", "default"),
                create_test_kustomization("ks3", "default"),
            ],
            vec![],
            vec![],
        );

        app.set_current_selected(2);
        assert_eq!(app.current_selected(), 2);
    }

    #[test]
    fn test_selection_per_tab_independent() {
        let mut app = App::for_testing(
            Tab::Kustomizations,
            vec![
                create_test_kustomization("ks1", "default"),
                create_test_kustomization("ks2", "default"),
            ],
            vec![
                create_test_helm_release("hr1", "default"),
                create_test_helm_release("hr2", "default"),
                create_test_helm_release("hr3", "default"),
            ],
            vec![],
        );

        // Set selection for kustomizations tab
        app.set_current_selected(1);
        assert_eq!(app.current_selected(), 1);

        // Switch to helm releases tab
        app.tab = Tab::HelmReleases;
        assert_eq!(app.current_selected(), 0); // Independent selection

        // Set selection for helm releases
        app.set_current_selected(2);
        assert_eq!(app.current_selected(), 2);

        // Switch back to kustomizations - selection preserved
        app.tab = Tab::Kustomizations;
        assert_eq!(app.current_selected(), 1);
    }

    // ===== Item Count Tests (using real App) =====

    #[test]
    fn test_current_item_count_empty() {
        let app = App::for_testing(Tab::Kustomizations, vec![], vec![], vec![]);
        assert_eq!(app.current_item_count(), 0);
    }

    #[test]
    fn test_current_item_count_kustomizations() {
        let app = App::for_testing(
            Tab::Kustomizations,
            vec![
                create_test_kustomization("ks1", "default"),
                create_test_kustomization("ks2", "kube-system"),
            ],
            vec![],
            vec![],
        );
        assert_eq!(app.current_item_count(), 2);
    }

    #[test]
    fn test_current_item_count_helm_releases() {
        let app = App::for_testing(
            Tab::HelmReleases,
            vec![],
            vec![
                create_test_helm_release("hr1", "default"),
                create_test_helm_release("hr2", "default"),
                create_test_helm_release("hr3", "default"),
            ],
            vec![],
        );
        assert_eq!(app.current_item_count(), 3);
    }

    #[test]
    fn test_current_item_count_helm_charts() {
        let app = App::for_testing(
            Tab::HelmCharts,
            vec![],
            vec![],
            vec![create_test_helm_chart("hc1", "default")],
        );
        assert_eq!(app.current_item_count(), 1);
    }

    // ===== Get Selected Resource Tests (using real App) =====

    #[test]
    fn test_get_selected_resource_empty() {
        let app = App::for_testing(Tab::Kustomizations, vec![], vec![], vec![]);
        assert!(app.get_selected_resource().is_none());
    }

    #[test]
    fn test_get_selected_resource_kustomization() {
        let app = App::for_testing(
            Tab::Kustomizations,
            vec![create_test_kustomization("my-ks", "flux-system")],
            vec![],
            vec![],
        );

        let resource = app.get_selected_resource().expect("should have resource");
        assert_eq!(resource.name(), "my-ks");
        assert_eq!(resource.namespace(), "flux-system");
        assert_eq!(resource.kind(), "Kustomization");
    }

    #[test]
    fn test_get_selected_resource_helm_release() {
        let app = App::for_testing(
            Tab::HelmReleases,
            vec![],
            vec![create_test_helm_release("my-release", "default")],
            vec![],
        );

        let resource = app.get_selected_resource().expect("should have resource");
        assert_eq!(resource.name(), "my-release");
        assert_eq!(resource.namespace(), "default");
        assert_eq!(resource.kind(), "HelmRelease");
    }

    #[test]
    fn test_get_selected_resource_helm_chart() {
        let app = App::for_testing(
            Tab::HelmCharts,
            vec![],
            vec![],
            vec![create_test_helm_chart("my-chart", "flux-system")],
        );

        let resource = app.get_selected_resource().expect("should have resource");
        assert_eq!(resource.name(), "my-chart");
        assert_eq!(resource.namespace(), "flux-system");
        assert_eq!(resource.kind(), "HelmChart");
    }

    #[test]
    fn test_get_selected_resource_out_of_bounds() {
        let mut app = App::for_testing(
            Tab::Kustomizations,
            vec![create_test_kustomization("only-one", "default")],
            vec![],
            vec![],
        );

        app.set_current_selected(5); // Out of bounds
        assert!(app.get_selected_resource().is_none());
    }

    // ===== App::for_testing Tests =====

    #[test]
    fn test_for_testing_constructor() {
        let app = App::for_testing(
            Tab::HelmReleases,
            vec![create_test_kustomization("ks1", "default")],
            vec![create_test_helm_release("hr1", "default")],
            vec![create_test_helm_chart("hc1", "default")],
        );

        assert_eq!(app.tab, Tab::HelmReleases);
        assert_eq!(app.kustomizations.len(), 1);
        assert_eq!(app.helm_releases.len(), 1);
        assert_eq!(app.helm_charts.len(), 1);
        assert_eq!(app.cluster_name, "test-cluster");
        assert!(!app.loading);
        assert!(app.client.is_none());
    }

    #[test]
    fn test_for_testing_defaults() {
        let app = App::for_testing(Tab::Kustomizations, vec![], vec![], vec![]);

        assert_eq!(app.selected, [0; 3]);
        assert!(app.namespace_filter.is_none());
        assert!(app.namespaces.is_empty());
        assert!(matches!(app.popup, Popup::None));
        assert!(app.last_error.is_none());
    }

    // ===== Original Tab Tests =====

    #[test]
    fn test_tab_default() {
        let tab = Tab::default();
        assert_eq!(tab, Tab::Kustomizations);
    }

    #[test]
    fn test_tab_all() {
        let tabs = Tab::all();
        assert_eq!(tabs.len(), 3);
        assert_eq!(tabs[0], Tab::Kustomizations);
        assert_eq!(tabs[1], Tab::HelmReleases);
        assert_eq!(tabs[2], Tab::HelmCharts);
    }

    #[test]
    fn test_tab_name() {
        assert_eq!(Tab::Kustomizations.name(), "Kustomizations");
        assert_eq!(Tab::HelmReleases.name(), "HelmReleases");
        assert_eq!(Tab::HelmCharts.name(), "HelmCharts");
    }

    #[test]
    fn test_tab_next() {
        assert_eq!(Tab::Kustomizations.next(), Tab::HelmReleases);
        assert_eq!(Tab::HelmReleases.next(), Tab::HelmCharts);
        assert_eq!(Tab::HelmCharts.next(), Tab::Kustomizations);
    }

    #[test]
    fn test_tab_previous() {
        assert_eq!(Tab::Kustomizations.previous(), Tab::HelmCharts);
        assert_eq!(Tab::HelmReleases.previous(), Tab::Kustomizations);
        assert_eq!(Tab::HelmCharts.previous(), Tab::HelmReleases);
    }

    #[test]
    fn test_tab_cycle_next() {
        let mut tab = Tab::Kustomizations;
        tab = tab.next();
        assert_eq!(tab, Tab::HelmReleases);
        tab = tab.next();
        assert_eq!(tab, Tab::HelmCharts);
        tab = tab.next();
        assert_eq!(tab, Tab::Kustomizations);
    }

    #[test]
    fn test_tab_cycle_previous() {
        let mut tab = Tab::Kustomizations;
        tab = tab.previous();
        assert_eq!(tab, Tab::HelmCharts);
        tab = tab.previous();
        assert_eq!(tab, Tab::HelmReleases);
        tab = tab.previous();
        assert_eq!(tab, Tab::Kustomizations);
    }

    #[test]
    fn test_popup_default() {
        let popup = Popup::default();
        assert!(matches!(popup, Popup::None));
    }

    // ===== Async Update Tests =====

    #[tokio::test]
    async fn test_update_next_tab() {
        let mut app = App::for_testing(Tab::Kustomizations, vec![], vec![], vec![]);

        app.update(Action::NextTab).await.unwrap();
        assert_eq!(app.tab, Tab::HelmReleases);

        app.update(Action::NextTab).await.unwrap();
        assert_eq!(app.tab, Tab::HelmCharts);

        app.update(Action::NextTab).await.unwrap();
        assert_eq!(app.tab, Tab::Kustomizations);
    }

    #[tokio::test]
    async fn test_update_previous_tab() {
        let mut app = App::for_testing(Tab::Kustomizations, vec![], vec![], vec![]);

        app.update(Action::PreviousTab).await.unwrap();
        assert_eq!(app.tab, Tab::HelmCharts);

        app.update(Action::PreviousTab).await.unwrap();
        assert_eq!(app.tab, Tab::HelmReleases);

        app.update(Action::PreviousTab).await.unwrap();
        assert_eq!(app.tab, Tab::Kustomizations);
    }

    #[tokio::test]
    async fn test_update_navigation_up() {
        let mut app = App::for_testing(
            Tab::Kustomizations,
            vec![
                create_test_kustomization("ks1", "default"),
                create_test_kustomization("ks2", "default"),
                create_test_kustomization("ks3", "default"),
            ],
            vec![],
            vec![],
        );

        // Start at index 2
        app.set_current_selected(2);
        assert_eq!(app.current_selected(), 2);

        // Move up
        app.update(Action::Up).await.unwrap();
        assert_eq!(app.current_selected(), 1);

        app.update(Action::Up).await.unwrap();
        assert_eq!(app.current_selected(), 0);

        // Can't go below 0
        app.update(Action::Up).await.unwrap();
        assert_eq!(app.current_selected(), 0);
    }

    #[tokio::test]
    async fn test_update_navigation_down() {
        let mut app = App::for_testing(
            Tab::Kustomizations,
            vec![
                create_test_kustomization("ks1", "default"),
                create_test_kustomization("ks2", "default"),
                create_test_kustomization("ks3", "default"),
            ],
            vec![],
            vec![],
        );

        assert_eq!(app.current_selected(), 0);

        app.update(Action::Down).await.unwrap();
        assert_eq!(app.current_selected(), 1);

        app.update(Action::Down).await.unwrap();
        assert_eq!(app.current_selected(), 2);

        // Can't go beyond last item
        app.update(Action::Down).await.unwrap();
        assert_eq!(app.current_selected(), 2);
    }

    #[tokio::test]
    async fn test_update_navigation_down_empty() {
        let mut app = App::for_testing(Tab::Kustomizations, vec![], vec![], vec![]);

        // Down on empty list should do nothing
        app.update(Action::Down).await.unwrap();
        assert_eq!(app.current_selected(), 0);
    }

    #[tokio::test]
    async fn test_update_top() {
        let mut app = App::for_testing(
            Tab::Kustomizations,
            vec![
                create_test_kustomization("ks1", "default"),
                create_test_kustomization("ks2", "default"),
                create_test_kustomization("ks3", "default"),
            ],
            vec![],
            vec![],
        );

        app.set_current_selected(2);

        app.update(Action::Top).await.unwrap();
        assert_eq!(app.current_selected(), 0);
    }

    #[tokio::test]
    async fn test_update_bottom() {
        let mut app = App::for_testing(
            Tab::Kustomizations,
            vec![
                create_test_kustomization("ks1", "default"),
                create_test_kustomization("ks2", "default"),
                create_test_kustomization("ks3", "default"),
            ],
            vec![],
            vec![],
        );

        app.update(Action::Bottom).await.unwrap();
        assert_eq!(app.current_selected(), 2);
    }

    #[tokio::test]
    async fn test_update_bottom_empty() {
        let mut app = App::for_testing(Tab::Kustomizations, vec![], vec![], vec![]);

        // Bottom on empty list should stay at 0
        app.update(Action::Bottom).await.unwrap();
        assert_eq!(app.current_selected(), 0);
    }

    #[tokio::test]
    async fn test_update_select_opens_details() {
        let mut app = App::for_testing(
            Tab::Kustomizations,
            vec![create_test_kustomization("my-ks", "flux-system")],
            vec![],
            vec![],
        );

        app.update(Action::Select).await.unwrap();

        match &app.popup {
            Popup::ResourceDetails { resource } => {
                assert_eq!(resource.name(), "my-ks");
            }
            _ => panic!("Expected ResourceDetails popup"),
        }
    }

    #[tokio::test]
    async fn test_update_select_empty_no_popup() {
        let mut app = App::for_testing(Tab::Kustomizations, vec![], vec![], vec![]);

        app.update(Action::Select).await.unwrap();

        assert!(matches!(app.popup, Popup::None));
    }

    #[tokio::test]
    async fn test_update_filter_namespace() {
        let mut app = App::for_testing(Tab::Kustomizations, vec![], vec![], vec![]);
        app.namespaces = vec!["default".to_string(), "kube-system".to_string()];

        app.update(Action::FilterNamespace).await.unwrap();

        match &app.popup {
            Popup::NamespaceFilter {
                namespaces,
                selected,
            } => {
                assert_eq!(*selected, 0);
                assert_eq!(namespaces.len(), 3); // "All namespaces" + 2 namespaces
                assert_eq!(namespaces[0], "All namespaces");
                assert_eq!(namespaces[1], "default");
                assert_eq!(namespaces[2], "kube-system");
            }
            _ => panic!("Expected NamespaceFilter popup"),
        }
    }

    #[tokio::test]
    async fn test_update_close_popup() {
        let mut app = App::for_testing(
            Tab::Kustomizations,
            vec![create_test_kustomization("ks1", "default")],
            vec![],
            vec![],
        );

        // Open a popup first
        app.update(Action::Select).await.unwrap();
        assert!(matches!(app.popup, Popup::ResourceDetails { .. }));

        // Close it
        app.update(Action::ClosePopup).await.unwrap();
        assert!(matches!(app.popup, Popup::None));
    }

    #[tokio::test]
    async fn test_update_quit_noop() {
        let mut app = App::for_testing(Tab::Kustomizations, vec![], vec![], vec![]);

        // Quit action does nothing (handled in main loop)
        app.update(Action::Quit).await.unwrap();
        // Should not panic, just return Ok
    }

    #[tokio::test]
    async fn test_update_noop() {
        let mut app = App::for_testing(Tab::Kustomizations, vec![], vec![], vec![]);

        // Noop does nothing
        app.update(Action::Noop).await.unwrap();
        assert_eq!(app.tab, Tab::Kustomizations);
        assert_eq!(app.current_selected(), 0);
    }

    #[tokio::test]
    async fn test_update_set_namespace() {
        let mut app = App::for_testing(Tab::Kustomizations, vec![], vec![], vec![]);
        app.popup = Popup::NamespaceFilter {
            namespaces: vec!["All".to_string(), "default".to_string()],
            selected: 0,
        };

        app.update(Action::SetNamespace(Some("default".to_string())))
            .await
            .unwrap();

        assert_eq!(app.namespace_filter, Some("default".to_string()));
        assert!(matches!(app.popup, Popup::None));
    }

    #[tokio::test]
    async fn test_update_set_namespace_all() {
        let mut app = App::for_testing(Tab::Kustomizations, vec![], vec![], vec![]);
        app.namespace_filter = Some("default".to_string());

        app.update(Action::SetNamespace(None)).await.unwrap();

        assert!(app.namespace_filter.is_none());
    }

    #[tokio::test]
    async fn test_refresh_data_without_client() {
        let mut app = App::for_testing(Tab::Kustomizations, vec![], vec![], vec![]);
        app.loading = true;

        app.refresh_data().await.unwrap();

        // Without a client, refresh just sets loading to false
        assert!(!app.loading);
    }

    // ===== Popup Variant Tests =====

    #[test]
    fn test_popup_namespace_filter() {
        let popup = Popup::NamespaceFilter {
            namespaces: vec!["default".to_string(), "kube-system".to_string()],
            selected: 1,
        };

        if let Popup::NamespaceFilter {
            namespaces,
            selected,
        } = popup
        {
            assert_eq!(namespaces.len(), 2);
            assert_eq!(selected, 1);
        } else {
            panic!("Expected NamespaceFilter");
        }
    }

    #[test]
    fn test_popup_resource_details() {
        let ks = create_test_kustomization("test-ks", "default");
        let popup = Popup::ResourceDetails {
            resource: Box::new(ks),
        };

        if let Popup::ResourceDetails { resource } = popup {
            assert_eq!(resource.name(), "test-ks");
            assert_eq!(resource.namespace(), "default");
        } else {
            panic!("Expected ResourceDetails");
        }
    }

    #[test]
    fn test_popup_reconciling() {
        let popup = Popup::Reconciling {
            name: "my-resource".to_string(),
            namespace: "flux-system".to_string(),
        };

        if let Popup::Reconciling { name, namespace } = popup {
            assert_eq!(name, "my-resource");
            assert_eq!(namespace, "flux-system");
        } else {
            panic!("Expected Reconciling");
        }
    }

    #[test]
    fn test_popup_error() {
        let popup = Popup::Error {
            message: "Something went wrong".to_string(),
        };

        if let Popup::Error { message } = popup {
            assert_eq!(message, "Something went wrong");
        } else {
            panic!("Expected Error");
        }
    }

    // ===== Additional App Field Tests =====

    #[test]
    fn test_app_loading_state() {
        let app = App::for_testing(Tab::Kustomizations, vec![], vec![], vec![]);
        assert!(!app.loading);
    }

    #[test]
    fn test_app_last_error() {
        let mut app = App::for_testing(Tab::Kustomizations, vec![], vec![], vec![]);
        assert!(app.last_error.is_none());

        app.last_error = Some("Test error".to_string());
        assert_eq!(app.last_error, Some("Test error".to_string()));
    }

    #[test]
    fn test_app_cluster_name() {
        let app = App::for_testing(Tab::Kustomizations, vec![], vec![], vec![]);
        assert_eq!(app.cluster_name, "test-cluster");
    }

    #[test]
    fn test_app_multiple_resources() {
        let app = App::for_testing(
            Tab::Kustomizations,
            vec![
                create_test_kustomization("ks1", "ns1"),
                create_test_kustomization("ks2", "ns2"),
            ],
            vec![create_test_helm_release("hr1", "ns1")],
            vec![
                create_test_helm_chart("hc1", "ns1"),
                create_test_helm_chart("hc2", "ns2"),
                create_test_helm_chart("hc3", "ns3"),
            ],
        );

        assert_eq!(app.kustomizations.len(), 2);
        assert_eq!(app.helm_releases.len(), 1);
        assert_eq!(app.helm_charts.len(), 3);
    }

    // ===== Tab Edge Cases =====

    #[test]
    fn test_tab_as_usize() {
        // Verify Tab can be converted to usize for indexing
        assert_eq!(Tab::Kustomizations as usize, 0);
        assert_eq!(Tab::HelmReleases as usize, 1);
        assert_eq!(Tab::HelmCharts as usize, 2);
    }

    #[test]
    fn test_app_selected_array_access() {
        let mut app = App::for_testing(Tab::Kustomizations, vec![], vec![], vec![]);

        // Direct access to selected array
        app.selected[0] = 5;
        app.selected[1] = 10;
        app.selected[2] = 15;

        assert_eq!(app.selected[0], 5);
        assert_eq!(app.selected[1], 10);
        assert_eq!(app.selected[2], 15);

        // Verify current_selected works with direct access
        app.tab = Tab::Kustomizations;
        assert_eq!(app.current_selected(), 5);

        app.tab = Tab::HelmReleases;
        assert_eq!(app.current_selected(), 10);

        app.tab = Tab::HelmCharts;
        assert_eq!(app.current_selected(), 15);
    }

    #[tokio::test]
    async fn test_update_select_helm_release() {
        let mut app = App::for_testing(
            Tab::HelmReleases,
            vec![],
            vec![create_test_helm_release("my-release", "default")],
            vec![],
        );

        app.update(Action::Select).await.unwrap();

        if let Popup::ResourceDetails { resource } = &app.popup {
            assert_eq!(resource.name(), "my-release");
            assert_eq!(resource.kind(), "HelmRelease");
        } else {
            panic!("Expected ResourceDetails popup");
        }
    }

    #[tokio::test]
    async fn test_update_select_helm_chart() {
        let mut app = App::for_testing(
            Tab::HelmCharts,
            vec![],
            vec![],
            vec![create_test_helm_chart("my-chart", "flux-system")],
        );

        app.update(Action::Select).await.unwrap();

        if let Popup::ResourceDetails { resource } = &app.popup {
            assert_eq!(resource.name(), "my-chart");
            assert_eq!(resource.kind(), "HelmChart");
        } else {
            panic!("Expected ResourceDetails popup");
        }
    }
}
