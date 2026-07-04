use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Paragraph;

use crate::app::focus::Focus;
use crate::app::state::AppState;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let mode = match state.focus {
        Focus::Tree => "TREE",
        Focus::Directory => "DIR",
        Focus::Preview => "PREVIEW",
    };

    let path = state.tree.current_dir.display().to_string();
    let max_path = (area.width as usize).saturating_sub(30);
    let path = if path.len() > max_path {
        format!("...{}", &path[path.len().saturating_sub(max_path.saturating_sub(3))..])
    } else {
        path
    };

    let line = Line::from(vec![
        Span::styled(
            " FILE-MNGR ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" │ "),
        Span::styled(path, Style::default().fg(Color::Cyan)),
        Span::raw(" │ "),
        Span::styled(
            mode,
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}
