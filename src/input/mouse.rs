use crossterm::event::MouseEvent;
use crossterm::event::MouseEventKind;

use crate::app::action::Action;

pub fn map_mouse(event: MouseEvent) -> Option<Action> {
    match event.kind {
        MouseEventKind::Down(_) => Some(Action::MouseClick(event.column, event.row)),
        _ => None,
    }
}

