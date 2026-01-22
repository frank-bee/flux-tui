//! Tab bar widget

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Tabs as RataTabs},
};

use crate::app::state::Tab;
use crate::ui::theme::styles;

/// Draw the tab bar
pub fn draw(frame: &mut Frame, area: Rect, current_tab: Tab) {
    let titles: Vec<Line> = Tab::all()
        .iter()
        .map(|t| {
            let style = if *t == current_tab {
                styles::tab_active()
            } else {
                styles::tab()
            };
            Line::from(format!(" {} ", t.name())).style(style)
        })
        .collect();

    let tabs = RataTabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(styles::border()),
        )
        .select(current_tab as usize)
        .divider(" │ ")
        .highlight_style(styles::tab_active());

    frame.render_widget(tabs, area);
}
