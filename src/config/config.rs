use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use super::theme::ThemeColors;

fn xdg_config_home() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let home = std::env::var_os("HOME").unwrap_or_default();
            PathBuf::from(home).join(".config")
        })
}

fn config_dir() -> PathBuf {
    xdg_config_home().join("file-mang-snbx")
}

fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

fn default_hidden() -> bool {
    false
}
fn default_sort_by() -> String {
    "name".to_string()
}
fn default_sort_dirs_first() -> bool {
    true
}
fn default_renderer() -> String {
    "normal".to_string()
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "default_hidden")]
    pub show_hidden: bool,
    #[serde(default = "default_sort_by")]
    pub sort_by: String,
    #[serde(default = "default_sort_dirs_first")]
    pub sort_dirs_first: bool,
    #[serde(default = "default_renderer")]
    pub renderer: String,
    #[serde(default)]
    pub theme: ThemeColors,
    #[serde(default = "default_keymap")]
    pub keys: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            show_hidden: default_hidden(),
            sort_by: default_sort_by(),
            sort_dirs_first: default_sort_dirs_first(),
            renderer: default_renderer(),
            theme: ThemeColors::default(),
            keys: default_keymap(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = config_path();
        if !path.exists() {
            let cfg = Config::default();
            let _ = cfg.save();
            return cfg;
        }
        match std::fs::read_to_string(&path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(cfg) => cfg,
                Err(e) => {
                    eprintln!("config parse error: {e}, using defaults");
                    Config::default()
                }
            },
            Err(e) => {
                eprintln!("config read error: {e}, using defaults");
                Config::default()
            }
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let dir = config_dir();
        std::fs::create_dir_all(&dir).map_err(|e| format!("create config dir: {e}"))?;
        let content = toml::to_string_pretty(self).map_err(|e| format!("serialize: {e}"))?;
        std::fs::write(config_path(), content).map_err(|e| format!("write config: {e}"))?;
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_default_roundtrip() {
        let cfg = Config::default();
        let toml_str = toml::to_string_pretty(&cfg).expect("serialize");
        let parsed: Config = toml::from_str(&toml_str).expect("deserialize");
        assert_eq!(parsed.show_hidden, cfg.show_hidden);
        assert_eq!(parsed.sort_by, cfg.sort_by);
        assert_eq!(parsed.renderer, cfg.renderer);
        assert!(!parsed.keys.is_empty());
    }

    #[test]
    fn config_toml_parse_custom() {
        let toml_str = r#"
show_hidden = true
sort_by = "date"
renderer = "retro"

[theme]
active_border = "red"
dir_fg = "green"
"#;
        let cfg: Config = toml::from_str(toml_str).expect("parse");
        assert!(cfg.show_hidden);
        assert_eq!(cfg.sort_by, "date");
        assert_eq!(cfg.renderer, "retro");
        assert_eq!(cfg.theme.active_border, "red");
        assert_eq!(cfg.theme.dir_fg, "green");
        assert_eq!(cfg.theme.inactive_border, "darkgray");
        assert!(cfg.keys.contains_key("quit"));
    }
}

fn default_keymap() -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("quit".to_string(), "q".to_string());
    m.insert("move_up".to_string(), "i".to_string());
    m.insert("move_down".to_string(), "k".to_string());
    m.insert("parent_dir".to_string(), "j".to_string());
    m.insert("enter_dir".to_string(), "l".to_string());
    m.insert("focus_next".to_string(), "Tab".to_string());
    m.insert("focus_prev".to_string(), "BackTab".to_string());
    m.insert("toggle_select".to_string(), " ".to_string());
    m.insert("history_back".to_string(), "u".to_string());
    m.insert("history_forward".to_string(), "F".to_string());
    m.insert("go_home".to_string(), "H".to_string());
    m.insert("go_root".to_string(), "R".to_string());
    m.insert("copy".to_string(), "c".to_string());
    m.insert("cut".to_string(), "m".to_string());
    m.insert("paste".to_string(), "p".to_string());
    m.insert("rename".to_string(), "r".to_string());
    m.insert("delete".to_string(), "d".to_string());
    m.insert("new_file".to_string(), "n".to_string());
    m.insert("new_dir".to_string(), "N".to_string());
    m.insert("search".to_string(), "/".to_string());
    m.insert("cycle_renderer".to_string(), "g".to_string());
    m
}
