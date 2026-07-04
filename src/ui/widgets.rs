use ratatui::style::Stylize;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;

use crate::config::ThemeColors;

pub fn panel_block(title: impl Into<String>, theme: &ThemeColors, active: bool) -> Block<'static> {
    let color = theme.border(active);
    Block::default()
        .title(title.into())
        .borders(Borders::ALL)
        .fg(color)
}
