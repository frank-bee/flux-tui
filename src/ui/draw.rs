//! Main drawing function (View in The Elm Architecture)

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::app::state::{App, Popup, Tab};

use super::{
    layout::{popup_area, MainLayout},
    theme::{styles, ui},
    widgets::{resource_table, status_bar, tabs},
};

/// Main draw function - renders the entire UI
pub fn draw(frame: &mut Frame, app: &App) {
    let layout = MainLayout::new(frame.area());

    // Draw header
    draw_header(frame, layout.header, app);

    // Draw tabs
    tabs::draw(frame, layout.tabs, app.tab);

    // Draw main content (resource table)
    draw_content(frame, layout.content, app);

    // Draw status bar
    status_bar::draw(frame, layout.status_bar, app);

    // Draw popup if active
    draw_popup(frame, app);
}

/// Draw the header bar
fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let ns_display = app
        .namespace_filter
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("all");

    let header_text = format!(
        " flux-tui                                              cluster: {} │ ns: {}",
        app.cluster_name, ns_display
    );

    let header = Paragraph::new(header_text)
        .style(styles::header())
        .alignment(Alignment::Left);

    frame.render_widget(header, area);
}

/// Draw the main content area
fn draw_content(frame: &mut Frame, area: Rect, app: &App) {
    match app.tab {
        Tab::Kustomizations => {
            resource_table::draw_kustomizations(
                frame,
                area,
                &app.kustomizations,
                app.current_selected(),
            );
        }
        Tab::HelmReleases => {
            resource_table::draw_helm_releases(
                frame,
                area,
                &app.helm_releases,
                app.current_selected(),
            );
        }
        Tab::HelmCharts => {
            resource_table::draw_helm_charts(frame, area, &app.helm_charts, app.current_selected());
        }
    }

    // Show loading indicator if loading
    if app.loading {
        let loading_area = Rect::new(area.x + area.width - 12, area.y, 10, 1);
        let loading = Paragraph::new(" Loading... ").style(styles::muted());
        frame.render_widget(loading, loading_area);
    }
}

/// Draw popup if one is active
fn draw_popup(frame: &mut Frame, app: &App) {
    match &app.popup {
        Popup::None => {}
        Popup::NamespaceFilter {
            namespaces,
            selected,
        } => {
            draw_namespace_popup(frame, namespaces, *selected);
        }
        Popup::ResourceDetails { resource } => {
            draw_details_popup(frame, resource.as_ref());
        }
        Popup::Reconciling { name, namespace } => {
            draw_reconciling_popup(frame, name, namespace);
        }
        Popup::Error { message } => {
            draw_error_popup(frame, message);
        }
    }
}

/// Draw namespace filter popup
fn draw_namespace_popup(frame: &mut Frame, namespaces: &[String], selected: usize) {
    let area = popup_area(frame.area(), 40, 60);

    // Clear the background
    frame.render_widget(Clear, area);

    let items: Vec<Line> = namespaces
        .iter()
        .enumerate()
        .map(|(i, ns)| {
            if i == selected {
                Line::from(format!(" ▶ {} ", ns)).style(styles::selected())
            } else {
                Line::from(format!("   {} ", ns)).style(styles::text())
            }
        })
        .collect();

    let block = Block::default()
        .title(" Select Namespace ")
        .title_style(styles::title())
        .borders(Borders::ALL)
        .border_style(styles::border_highlight());

    let paragraph = Paragraph::new(items).block(block);

    frame.render_widget(paragraph, area);
}

/// Draw resource details popup
fn draw_details_popup(frame: &mut Frame, resource: &dyn crate::kubernetes::resources::FluxResource) {
    let area = popup_area(frame.area(), 70, 70);

    frame.render_widget(Clear, area);

    let details = format!(
        "Name:      {}\n\
         Namespace: {}\n\
         Ready:     {}\n\
         Status:    {}\n\
         Revision:  {}\n\
         Suspended: {}",
        resource.name(),
        resource.namespace(),
        if resource.is_ready() { "Yes" } else { "No" },
        resource.status_message(),
        resource.revision().unwrap_or("-"),
        if resource.is_suspended() { "Yes" } else { "No" }
    );

    let block = Block::default()
        .title(format!(" {} Details ", resource.kind()))
        .title_style(styles::title())
        .borders(Borders::ALL)
        .border_style(styles::border_highlight());

    let paragraph = Paragraph::new(details)
        .block(block)
        .style(styles::text())
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Draw reconciling popup
fn draw_reconciling_popup(frame: &mut Frame, name: &str, namespace: &str) {
    let area = popup_area(frame.area(), 50, 20);

    frame.render_widget(Clear, area);

    let text = format!("Reconciling {}/{} ...", namespace, name);

    let block = Block::default()
        .title(" Reconciling ")
        .title_style(styles::title())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ui::PRIMARY));

    let paragraph = Paragraph::new(text)
        .block(block)
        .style(styles::text())
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

/// Draw error popup
fn draw_error_popup(frame: &mut Frame, message: &str) {
    let area = popup_area(frame.area(), 60, 30);

    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Error ")
        .title_style(Style::default().fg(crate::ui::theme::status::FAILED).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(crate::ui::theme::status::FAILED));

    let paragraph = Paragraph::new(message)
        .block(block)
        .style(styles::text())
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
