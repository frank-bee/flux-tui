//! Resource table widget for displaying Flux resources

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Row, Table, TableState},
};

use crate::kubernetes::resources::{HelmChart, HelmRelease, Kustomization, ResourceStatus};
use crate::ui::theme::{icons, styles};

/// Draw the Kustomizations table
pub fn draw_kustomizations(
    frame: &mut Frame,
    area: Rect,
    kustomizations: &[Kustomization],
    selected: usize,
) {
    let header_cells = ["NAME", "NAMESPACE", "READY", "STATUS", "REVISION", "SUS"]
        .iter()
        .map(|h| Cell::from(*h).style(styles::header()));
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = kustomizations
        .iter()
        .map(|k| {
            let (icon, style) = status_icon_style(&k.status);
            Row::new([
                Cell::from(k.name.clone()),
                Cell::from(k.namespace.clone()),
                Cell::from(icon).style(style),
                Cell::from(truncate(&k.status_message, 30)),
                Cell::from(truncate(
                    &k.revision.clone().unwrap_or_else(|| "-".to_string()),
                    15,
                )),
                Cell::from(if k.suspended { "⏸" } else { "-" }),
            ])
        })
        .collect();

    let widths = [
        Constraint::Min(20),
        Constraint::Min(15),
        Constraint::Length(5),
        Constraint::Min(30),
        Constraint::Min(15),
        Constraint::Length(3),
    ];

    render_table(
        frame,
        area,
        header,
        rows,
        widths,
        selected,
        "Kustomizations",
    );
}

/// Draw the HelmReleases table
pub fn draw_helm_releases(
    frame: &mut Frame,
    area: Rect,
    helm_releases: &[HelmRelease],
    selected: usize,
) {
    let header_cells = [
        "NAME",
        "NAMESPACE",
        "READY",
        "STATUS",
        "CHART",
        "VERSION",
        "SUS",
    ]
    .iter()
    .map(|h| Cell::from(*h).style(styles::header()));
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = helm_releases
        .iter()
        .map(|h| {
            let (icon, style) = status_icon_style(&h.status);
            Row::new([
                Cell::from(h.name.clone()),
                Cell::from(h.namespace.clone()),
                Cell::from(icon).style(style),
                Cell::from(truncate(&h.status_message, 25)),
                Cell::from(h.chart.clone()),
                Cell::from(h.version.clone().unwrap_or_else(|| "-".to_string())),
                Cell::from(if h.suspended { "⏸" } else { "-" }),
            ])
        })
        .collect();

    let widths = [
        Constraint::Min(20),
        Constraint::Min(15),
        Constraint::Length(5),
        Constraint::Min(25),
        Constraint::Min(15),
        Constraint::Min(10),
        Constraint::Length(3),
    ];

    render_table(frame, area, header, rows, widths, selected, "HelmReleases");
}

/// Draw the HelmCharts table
pub fn draw_helm_charts(frame: &mut Frame, area: Rect, helm_charts: &[HelmChart], selected: usize) {
    let header_cells = [
        "NAME",
        "NAMESPACE",
        "READY",
        "STATUS",
        "CHART",
        "VERSION",
        "SOURCE",
    ]
    .iter()
    .map(|h| Cell::from(*h).style(styles::header()));
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = helm_charts
        .iter()
        .map(|h| {
            let (icon, style) = status_icon_style(&h.status);
            Row::new([
                Cell::from(h.name.clone()),
                Cell::from(h.namespace.clone()),
                Cell::from(icon).style(style),
                Cell::from(truncate(&h.status_message, 25)),
                Cell::from(h.chart.clone()),
                Cell::from(h.version.clone().unwrap_or_else(|| "-".to_string())),
                Cell::from(truncate(&h.source_ref, 20)),
            ])
        })
        .collect();

    let widths = [
        Constraint::Min(20),
        Constraint::Min(15),
        Constraint::Length(5),
        Constraint::Min(25),
        Constraint::Min(15),
        Constraint::Min(10),
        Constraint::Min(20),
    ];

    render_table(frame, area, header, rows, widths, selected, "HelmCharts");
}

/// Render a table with the given configuration
fn render_table<'a>(
    frame: &mut Frame,
    area: Rect,
    header: Row<'a>,
    rows: Vec<Row<'a>>,
    widths: impl IntoIterator<Item = Constraint>,
    selected: usize,
    _title: &str,
) {
    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(styles::border_highlight()),
        )
        .row_highlight_style(styles::selected())
        .highlight_symbol("▶ ");

    let mut state = TableState::default();
    state.select(Some(selected));

    frame.render_stateful_widget(table, area, &mut state);
}

/// Get the status icon and style for a resource status
fn status_icon_style(status: &ResourceStatus) -> (&'static str, Style) {
    match status {
        ResourceStatus::Ready => (icons::READY, styles::status_ready()),
        ResourceStatus::Failed => (icons::FAILED, styles::status_failed()),
        ResourceStatus::Reconciling => (icons::RECONCILING, styles::status_reconciling()),
        ResourceStatus::Suspended => (icons::SUSPENDED, styles::status_suspended()),
        ResourceStatus::Unknown => (icons::UNKNOWN, styles::status_unknown()),
    }
}

/// Truncate a string to a maximum length
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_short_string() {
        let result = truncate("hello", 10);
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_truncate_exact_length() {
        let result = truncate("hello", 5);
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_truncate_long_string() {
        let result = truncate("hello world", 8);
        assert_eq!(result, "hello...");
    }

    #[test]
    fn test_status_icon_style_ready() {
        let (icon, style) = status_icon_style(&ResourceStatus::Ready);
        assert_eq!(icon, icons::READY);
        assert_eq!(style.fg, Some(crate::ui::theme::status::READY));
    }

    #[test]
    fn test_status_icon_style_failed() {
        let (icon, style) = status_icon_style(&ResourceStatus::Failed);
        assert_eq!(icon, icons::FAILED);
        assert_eq!(style.fg, Some(crate::ui::theme::status::FAILED));
    }

    #[test]
    fn test_status_icon_style_reconciling() {
        let (icon, style) = status_icon_style(&ResourceStatus::Reconciling);
        assert_eq!(icon, icons::RECONCILING);
        assert_eq!(style.fg, Some(crate::ui::theme::status::RECONCILING));
    }

    #[test]
    fn test_status_icon_style_suspended() {
        let (icon, style) = status_icon_style(&ResourceStatus::Suspended);
        assert_eq!(icon, icons::SUSPENDED);
        assert_eq!(style.fg, Some(crate::ui::theme::status::SUSPENDED));
    }

    #[test]
    fn test_status_icon_style_unknown() {
        let (icon, style) = status_icon_style(&ResourceStatus::Unknown);
        assert_eq!(icon, icons::UNKNOWN);
        assert_eq!(style.fg, Some(crate::ui::theme::status::UNKNOWN));
    }

    #[test]
    fn test_truncate_edge_cases() {
        // Empty string
        assert_eq!(truncate("", 10), "");

        // Very short max_len edge cases
        assert_eq!(truncate("abcdefghij", 5), "ab...");
        assert_eq!(truncate("abcdefghij", 4), "a...");

        // Unicode should be handled
        let result = truncate("Hello, 世界!", 8);
        assert!(result.len() <= 8 || result.ends_with("..."));
    }

    #[test]
    fn test_truncate_boundary() {
        // Test boundary between truncate and no-truncate
        assert_eq!(truncate("12345", 5), "12345");
        assert_eq!(truncate("123456", 5), "12...");
    }
}
