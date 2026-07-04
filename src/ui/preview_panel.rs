use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::widgets::Paragraph;

use crate::app::focus::Focus;
use crate::app::state::AppState;

use super::widgets;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let is_active = state.focus == Focus::Preview;
    let file_type = &state.preview.file_type;

    let title = if file_type.is_empty() {
        " Preview ".to_string()
    } else {
        format!(" Preview [{}] ", file_type)
    };

    let block = widgets::panel_block(title, &state.config.theme, is_active);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if state.preview.is_image {
        render_image_preview(frame, inner, state);
        return;
    }

    if state.preview.lines.is_empty() {
        let text = Paragraph::new("No file selected").fg(Color::DarkGray);
        frame.render_widget(text, inner);
        return;
    }

    let view_height = (inner.height as usize).saturating_sub(1);
    let total = state.preview.lines.len();

    state.preview_ui.ensure_visible(view_height, total);
    let range = state.preview_ui.visible_range(view_height, total);

    let mut text = String::new();
    for i in range {
        text.push_str(&state.preview.lines[i]);
        text.push('\n');
    }

    if state.preview.truncated {
        let info = format!(
            "\n\n-- truncated ({} of {} lines) --",
            view_height.min(total),
            state.preview.total_lines
        );
        text.push_str(&info);
    }

    let style = if state.preview.file_type == "binary" {
        Style::default()
            .fg(Color::Red)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let paragraph = Paragraph::new(text).style(style);
    frame.render_widget(paragraph, inner);
}

fn render_image_preview(frame: &mut Frame, area: Rect, state: &AppState) {
    let path = match &state.preview.cached_path {
        Some(p) => p,
        None => {
            frame.render_widget(
                Paragraph::new("No image file").fg(Color::DarkGray),
                area,
            );
            return;
        }
    };

    let pw = area.width.saturating_sub(1).max(1);
    let ph = area.height.saturating_sub(1).max(1);

    let mut cache = state.image_cache.borrow_mut();
    let decoded = match cache.get_or_load(path) {
        Ok(d) => d,
        Err(e) => {
            frame.render_widget(
                Paragraph::new(format!("[IMAGE ERROR] {}", e))
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
                area,
            );
            return;
        }
    };

    let frame_idx = if decoded.is_animated {
        state.animation.current_frame.min(decoded.frames.len().saturating_sub(1))
    } else {
        0
    };

    let f = &decoded.frames[frame_idx];
    let lines = state.renderer_kind.render(&f.rgba, f.width, f.height, pw, ph);
    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}
