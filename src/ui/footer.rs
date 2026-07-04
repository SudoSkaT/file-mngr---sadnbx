use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Paragraph;

use crate::app::state::AppState;
use crate::app::state::InputMode;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    match &state.mode {
        InputMode::Input { prompt, buffer, .. } => {
            render_input(frame, area, prompt, buffer);
        }
        InputMode::Confirm { message, .. } => {
            render_confirm(frame, area, message);
        }
        InputMode::Search(s) => {
            render_search(frame, area, s);
        }
        InputMode::Normal => {
            render_normal(frame, area, state);
        }
    }
}

fn render_input(frame: &mut Frame, area: Rect, prompt: &str, buffer: &str) {
    let style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let text = format!("{}{}", prompt, buffer);
    let line = Line::from(Span::styled(text, style));
    let block = Paragraph::new(line)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));
    frame.render_widget(block, area);
}

fn render_search(frame: &mut Frame, area: Rect, s: &crate::app::search::SearchState) {
    let style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    let text = format!("/{} [{}] {} results", s.query, s.kind.name(), s.results.len());
    let line = Line::from(Span::styled(text, style));
    let block = Paragraph::new(line)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));
    frame.render_widget(block, area);
}

fn render_confirm(frame: &mut Frame, area: Rect, message: &str) {
    let style = Style::default()
        .fg(Color::Red)
        .add_modifier(Modifier::BOLD);
    let line = Line::from(Span::styled(message, style));
    let block = Paragraph::new(line)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));
    frame.render_widget(block, area);
}

fn render_normal(frame: &mut Frame, area: Rect, state: &AppState) {
    let style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::DIM);

    let line = Line::from(vec![
        Span::styled(" i↑/k↓ ", style),
        Span::raw("│"),
        Span::styled(" j←/l→ ", style),
        Span::raw("│"),
        Span::styled(" Enter ", style),
        Span::raw("│"),
        Span::styled(" Tab ", style),
        Span::raw("│"),
        Span::styled(" Space:sel ", style),
        Span::raw("│"),
        Span::styled(" u:back F:forw ", style),
        Span::raw("│"),
        Span::styled(" c:copy m:cut p:pst ", style),
        Span::raw("│"),
        Span::styled(" r:ren d:del ", style),
        Span::raw("│"),
        Span::styled(" n:fil N:dir ", style),
        Span::raw("│"),
        Span::styled(" H:home R:root ", style),
        Span::raw("│"),
        Span::styled(" /:search ", style),
        Span::raw("│"),
        Span::styled(" g:mode ", style),
        Span::raw("│"),
        Span::styled(
            format!(" {} ", state.renderer_kind.name()),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}
