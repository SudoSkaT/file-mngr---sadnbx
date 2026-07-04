use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;

use crate::app::focus::Focus;
use crate::app::state::AppState;

use super::widgets;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let is_active = state.focus == Focus::Tree;
    let components = state.tree.path_components();
    let ui = &state.tree_ui;
    let view_height = (area.height as usize).saturating_sub(2);

    ui.ensure_visible(view_height, components.len());

    let range = ui.visible_range(view_height, components.len());
    let items: Vec<ListItem> = components[range.clone()]
        .iter()
        .enumerate()
        .map(|(i, comp)| {
            let global_idx = range.start + i;
            let is_selected = global_idx == ui.cursor;
            let depth = global_idx;
            let indent = "  ".repeat(depth);

            let display = if global_idx == 0 {
                "/".to_string()
            } else {
                let name = comp
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                format!("{}/", name)
            };

            let line = format!("{}{}", indent, display);

            let style = if is_selected {
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD)
            } else if global_idx == components.len() - 1 {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(line).style(style)
        })
        .collect();

    let title = if is_active {
        " Tree (active) "
    } else {
        " Tree "
    };
    let block = widgets::panel_block(title, &state.config.theme, is_active);
    let list = List::new(items).block(block);

    frame.render_widget(list, area);
}
