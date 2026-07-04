use std::collections::HashMap;

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;

use crate::app::action::Action;
use crate::app::state::InputMode;

pub fn map_key(key: KeyEvent, mode: &InputMode, keymap: &HashMap<String, String>) -> Option<Action> {
    match mode {
        InputMode::Input { .. } => map_input_key(key),
        InputMode::Confirm { .. } => map_confirm_key(key),
        InputMode::Search(_) => map_search_key(key),
        InputMode::Normal => {
            if let KeyCode::Char(c) = key.code {
                let s = c.to_string();
                if let Some(action) = resolve_action(&s, keymap) {
                    return Some(action);
                }
            }
            map_normal_key(key)
        }
    }
}

fn resolve_action(key: &str, keymap: &HashMap<String, String>) -> Option<Action> {
    for (action_name, bound_key) in keymap {
        if bound_key == key {
            return match action_name.as_str() {
                "quit" => Some(Action::Quit),
                "move_up" => Some(Action::MoveUp),
                "move_down" => Some(Action::MoveDown),
                "parent_dir" => Some(Action::ParentDir),
                "enter_dir" => Some(Action::EnterDir),
                "focus_next" => Some(Action::FocusNext),
                "focus_prev" => Some(Action::FocusPrev),
                "toggle_select" => Some(Action::ToggleSelect),
                "history_back" => Some(Action::HistoryBack),
                "history_forward" => Some(Action::HistoryForward),
                "go_home" => Some(Action::GoHome),
                "go_root" => Some(Action::GoRoot),
                "copy" => Some(Action::CopyToClipboard),
                "cut" => Some(Action::CutToClipboard),
                "paste" => Some(Action::PasteClipboard),
                "rename" => Some(Action::Rename),
                "delete" => Some(Action::Delete),
                "new_file" => Some(Action::NewFile),
                "new_dir" => Some(Action::NewDir),
                "search" => Some(Action::ToggleSearch),
                "cycle_renderer" => Some(Action::CycleRenderer),
                _ => None,
            };
        }
    }
    None
}

fn map_input_key(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Enter => Some(Action::InputSubmit),
        KeyCode::Esc => Some(Action::InputCancel),
        KeyCode::Backspace => Some(Action::InputBackspace),
        KeyCode::Char(c) => Some(Action::InputChar(c)),
        _ => None,
    }
}

fn map_search_key(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Esc => Some(Action::ToggleSearch),
        KeyCode::Up | KeyCode::Char('i') => Some(Action::MoveUp),
        KeyCode::Down | KeyCode::Char('k') => Some(Action::MoveDown),
        KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => Some(Action::EnterDir),
        KeyCode::Char('\t') => Some(Action::CycleSearchKind),
        KeyCode::Backspace => Some(Action::SearchBackspace),
        KeyCode::Char(c) => Some(Action::SearchChar(c)),
        _ => None,
    }
}

fn map_confirm_key(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => Some(Action::ConfirmYes),
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Some(Action::ConfirmNo),
        _ => None,
    }
}

fn map_normal_key(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => Some(Action::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Action::Quit),

        KeyCode::Up | KeyCode::Char('i') => Some(Action::MoveUp),
        KeyCode::Down | KeyCode::Char('k') => Some(Action::MoveDown),
        KeyCode::Left | KeyCode::Char('j') => Some(Action::ParentDir),
        KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => Some(Action::EnterDir),
        KeyCode::Backspace => Some(Action::ParentDir),

        KeyCode::Tab => Some(Action::FocusNext),
        KeyCode::BackTab => Some(Action::FocusPrev),

        KeyCode::Char(' ') => Some(Action::ToggleSelect),
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Action::SelectAll)
        }
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Action::ClearSelection)
        }

        KeyCode::Char('u') => Some(Action::HistoryBack),
        KeyCode::Char('F') => Some(Action::HistoryForward),

        KeyCode::Char('H') => Some(Action::GoHome),
        KeyCode::Char('R') => Some(Action::GoRoot),
        KeyCode::Home => Some(Action::GoToFirst),
        KeyCode::End => Some(Action::GoToLast),

        KeyCode::Char('c') => Some(Action::CopyToClipboard),
        KeyCode::Char('m') => Some(Action::CutToClipboard),
        KeyCode::Char('p') => Some(Action::PasteClipboard),
        KeyCode::Char('g') => Some(Action::CycleRenderer),
        KeyCode::Char('r') => Some(Action::Rename),
        KeyCode::Char('d') | KeyCode::Delete => Some(Action::Delete),
        KeyCode::Char('/') => Some(Action::ToggleSearch),
        KeyCode::Char('n') => Some(Action::NewFile),
        KeyCode::Char('N') => Some(Action::NewDir),

        _ => None,
    }
}
