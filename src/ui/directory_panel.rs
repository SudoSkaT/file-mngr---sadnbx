use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;

use crate::app::focus::Focus;
use crate::app::state::AppState;
use crate::app::state::InputMode;
use crate::app::state::ClipOp;
use crate::fs::icons;

use super::widgets;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let tree = &state.tree;
    let ui = &state.dir_ui;
    let is_active = state.focus == Focus::Directory;
    let view_height = (area.height as usize).saturating_sub(2);

    let (items, title) = match &state.mode {
        InputMode::Search(s) => {
            let total = s.results.len();
            let cursor = s.cursor;
            let mut search = s.clone();
            search.ensure_visible(view_height);
            let range = self_visible_range(search.scroll, view_height, total);
            let items: Vec<ListItem> = s.results[range]
                .iter()
                .enumerate()
                .map(|(i, &global_idx)| {
                    let node = &tree.nodes[global_idx];
                    let is_selected = search.scroll + i == cursor;
                    let is_marked = state.selection.contains(&node.path);
                    let is_cut = state.clipboard.as_ref().is_some_and(|c| {
                        c.operation == ClipOp::Cut && c.items.contains(&node.path)
                    });

                    let mut style = if is_selected {
                        Style::default()
                            .fg(Color::White)
                            .bg(Color::Blue)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                            .fg(if node.is_dir { Color::Cyan } else { Color::White })
                    };
                    if is_marked {
                        style = style.add_modifier(Modifier::REVERSED);
                    }
                    if is_cut {
                        style = style.add_modifier(Modifier::DIM);
                    }

                    let icon = icons::icon_name(&node.name, node.is_dir);
                    let content = format!(" {} {}", icon, node.name);
                    ListItem::new(content).style(style)
                })
                .collect();
            let title = format!(" search: {} ({}/{}) ", s.query, total, tree.nodes_len());
            (items, title)
        }
        _ => {
            ui.ensure_visible(view_height, tree.nodes_len());
            let range = ui.visible_range(view_height, tree.nodes_len());
            let items: Vec<ListItem> = tree.nodes[range.clone()]
                .iter()
                .enumerate()
                .map(|(i, node)| {
                    let global_idx = range.start + i;
                    let is_selected = global_idx == ui.cursor;
                    let is_marked = state.selection.contains(&node.path);
                    let is_cut = state.clipboard.as_ref().is_some_and(|c| {
                        c.operation == ClipOp::Cut && c.items.contains(&node.path)
                    });

                    let mut style = if is_selected {
                        Style::default()
                            .fg(Color::White)
                            .bg(Color::Blue)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                            .fg(if node.is_dir { Color::Cyan } else { Color::White })
                    };
                    if is_marked {
                        style = style.add_modifier(Modifier::REVERSED);
                    }
                    if is_cut {
                        style = style.add_modifier(Modifier::DIM);
                    }

                    let icon = icons::icon_name(&node.name, node.is_dir);
                    let content = format!(" {} {}", icon, node.name);
                    ListItem::new(content).style(style)
                })
                .collect();
            let title = format!(
                " {} ({} items) ",
                tree.current_dir.display(),
                tree.nodes_len()
            );
            (items, title)
        }
    };

    let block = widgets::panel_block(title, &state.config.theme, is_active);
    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn self_visible_range(scroll: usize, view_height: usize, total: usize) -> std::ops::Range<usize> {
    if total == 0 || view_height == 0 {
        return 0..0;
    }
    let start = scroll.min(total.saturating_sub(1));
    let end = (start + view_height).min(total);
    start..end
}
