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
    /// Kubernetes client
    pub client: KubeClient,

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

    /// Refresh all data from the cluster
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
