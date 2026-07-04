use serde::Deserialize;
use serde::Serialize;

use ratatui::style::Color;

fn color_from(name: &str) -> Color {
    match name.to_lowercase().as_str() {
        "reset" => Color::Reset,
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "darkgray" => Color::DarkGray,
        "lightred" => Color::LightRed,
        "lightgreen" => Color::LightGreen,
        "lightyellow" => Color::LightYellow,
        "lightblue" => Color::LightBlue,
        "lightmagenta" => Color::LightMagenta,
        "lightcyan" => Color::LightCyan,
        _ => Color::White,
    }
}

fn default_active_border() -> String {
    "yellow".to_string()
}
fn default_inactive_border() -> String {
    "darkgray".to_string()
}
fn default_selected_bg() -> String {
    "blue".to_string()
}
fn default_selected_fg() -> String {
    "white".to_string()
}
fn default_dir_fg() -> String {
    "cyan".to_string()
}
fn default_file_fg() -> String {
    "white".to_string()
}
fn default_header_bg() -> String {
    "yellow".to_string()
}
fn default_header_fg() -> String {
    "black".to_string()
}
fn default_focus_fg() -> String {
    "green".to_string()
}
fn default_error_fg() -> String {
    "red".to_string()
}
fn default_search_fg() -> String {
    "cyan".to_string()
}
fn default_input_fg() -> String {
    "yellow".to_string()
}
fn default_marked_modifier() -> String {
    "reversed".to_string()
}
fn default_cut_modifier() -> String {
    "dim".to_string()
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ThemeColors {
    #[serde(default = "default_active_border")]
    pub active_border: String,
    #[serde(default = "default_inactive_border")]
    pub inactive_border: String,
    #[serde(default = "default_selected_bg")]
    pub selected_bg: String,
    #[serde(default = "default_selected_fg")]
    pub selected_fg: String,
    #[serde(default = "default_dir_fg")]
    pub dir_fg: String,
    #[serde(default = "default_file_fg")]
    pub file_fg: String,
    #[serde(default = "default_header_bg")]
    pub header_bg: String,
    #[serde(default = "default_header_fg")]
    pub header_fg: String,
    #[serde(default = "default_focus_fg")]
    pub focus_fg: String,
    #[serde(default = "default_error_fg")]
    pub error_fg: String,
    #[serde(default = "default_search_fg")]
    pub search_fg: String,
    #[serde(default = "default_input_fg")]
    pub input_fg: String,
    #[serde(default = "default_marked_modifier")]
    pub marked_modifier: String,
    #[serde(default = "default_cut_modifier")]
    pub cut_modifier: String,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            active_border: default_active_border(),
            inactive_border: default_inactive_border(),
            selected_bg: default_selected_bg(),
            selected_fg: default_selected_fg(),
            dir_fg: default_dir_fg(),
            file_fg: default_file_fg(),
            header_bg: default_header_bg(),
            header_fg: default_header_fg(),
            focus_fg: default_focus_fg(),
            error_fg: default_error_fg(),
            search_fg: default_search_fg(),
            input_fg: default_input_fg(),
            marked_modifier: default_marked_modifier(),
            cut_modifier: default_cut_modifier(),
        }
    }
}

#[allow(dead_code)]
impl ThemeColors {
    pub fn border(&self, active: bool) -> Color {
        if active {
            color_from(&self.active_border)
        } else {
            color_from(&self.inactive_border)
        }
    }

    pub fn selected_style(&self) -> ratatui::style::Style {
        ratatui::style::Style::default()
            .fg(color_from(&self.selected_fg))
            .bg(color_from(&self.selected_bg))
            .add_modifier(ratatui::style::Modifier::BOLD)
    }

    pub fn dir_style(&self) -> ratatui::style::Style {
        ratatui::style::Style::default().fg(color_from(&self.dir_fg))
    }

    pub fn file_style(&self) -> ratatui::style::Style {
        ratatui::style::Style::default().fg(color_from(&self.file_fg))
    }
}
