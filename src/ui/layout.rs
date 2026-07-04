use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Paragraph;

use crate::app::state::AppState;

use super::details_panel;
use super::directory_panel;
use super::footer;
use super::header;
use super::preview_panel;
use super::tree_panel;

pub fn render(frame: &mut Frame, state: &AppState) {
    let area = frame.area();

    if area.width < 50 || area.height < 12 {
        let msg = format!("Terminal too small — need 50×12, have {}×{}", area.width, area.height);
        let text = Line::from(Span::styled(
            msg,
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ));
        frame.render_widget(
            Paragraph::new(text).alignment(Alignment::Center),
            area,
        );
        return;
    }

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(5),
            Constraint::Length(1),
        ])
        .split(area);

    header::render(frame, vertical[0], state);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(50),
            Constraint::Percentage(30),
        ])
        .split(vertical[1]);

    tree_panel::render(frame, horizontal[0], state);
    directory_panel::render(frame, horizontal[1], state);
    preview_panel::render(frame, horizontal[2], state);
    details_panel::render(frame, vertical[2], state);
    footer::render(frame, vertical[3], state);
}
