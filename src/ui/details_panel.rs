use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Paragraph;

use super::widgets;

use crate::app::focus::Focus;
use crate::app::state::AppState;
use crate::fs::metadata::FileMetadata;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let focus_name = match state.focus {
        Focus::Tree => "TREE",
        Focus::Directory => "DIR",
        Focus::Preview => "PREVIEW",
    };

    let sel_count = state.selection.count();
    let error_info = state
        .errors
        .latest()
        .map(|e| format!(" Error: {}", e))
        .unwrap_or_default();

    let path_str = state.tree.current_dir.display().to_string();
    let max_path = (area.width as usize).saturating_sub(30);
    let path_str = if path_str.len() > max_path {
        format!("...{}", &path_str[path_str.len().saturating_sub(max_path.saturating_sub(3))..])
    } else {
        path_str
    };

    let status_line = Line::from(vec![
        Span::styled(
            format!(" [{}] ", focus_name),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(path_str, Style::default().fg(Color::Cyan)),
        Span::raw(" "),
        Span::styled(
            format!("| Sel: {}", sel_count),
            Style::default().fg(Color::Yellow),
        ),
        Span::styled(error_info, Style::default().fg(Color::Red)),
    ]);

    let mut lines: Vec<Line> = Vec::new();

    if let Some(ref meta) = state.selected_metadata {
        build_meta_lines(meta, area.width, &mut lines);
    } else {
        lines.push(Line::from(
            Span::styled(" (no file selected)", Style::default().fg(Color::DarkGray)),
        ));
        lines.push(Line::from(""));
        lines.push(Line::from(""));
    }

    lines.push(Line::from(""));
    lines.push(status_line);

    let block = widgets::panel_block(" Details ", &state.config.theme, true);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn build_meta_lines(meta: &FileMetadata, width: u16, lines: &mut Vec<Line>) {
    let w = width as usize;

    let name = if meta.name.len() > w.saturating_sub(30) {
        let max = w.saturating_sub(33).max(5);
        format!("{}...", &meta.name[..max])
    } else {
        meta.name.clone()
    };

    let info = format!(" {} | {} | {}", name, meta.size, meta.modified);
    lines.push(Line::from(Span::styled(
        info,
        Style::default().fg(Color::White),
    )));

    let perms_style = Style::default().fg(Color::Magenta);
    let user_group = format!(" {} | {}:{}", meta.permissions, meta.user_name, meta.group_name);
    lines.push(Line::from(Span::styled(user_group, perms_style)));

    let type_entry = if let Some(count) = meta.entry_count {
        format!(" type: {} | entries: {}", meta.file_type, count)
    } else {
        format!(" type: {}", meta.file_type)
    };
    let type_style = Style::default().fg(Color::Cyan);
    lines.push(Line::from(Span::styled(type_entry, type_style)));

    if let Some(ref fs) = meta.fs_info {
        let fs_line = format!(
            " FS: {} | Total: {} | Used: {} | Avail: {}",
            fs.fs_type, fs.total, fs.used, fs.avail
        );
        let fs_style = Style::default().fg(Color::Blue);
        lines.push(Line::from(Span::styled(fs_line, fs_style)));
    } else {
        lines.push(Line::from(""));
    }
}
