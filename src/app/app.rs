use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

use color_eyre::Result;
use ratatui::backend::Backend;
use ratatui::Terminal;

use super::action::Action;
use super::state::AppState;
use super::state::ClipOp;
use super::state::Clipboard;
use super::state::InputMode;
use super::state::PendingAction;
use super::focus::Focus;
use crate::fs::metadata;
use crate::fs::operations;
use crate::fs::preview;
use crate::fs::tree::Tree;
use crate::input::keyboard;
use crate::input::mouse;
use crate::ui::layout;

pub struct App {
    pub state: AppState,
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

impl App {
    pub fn new() -> Result<Self> {
        let tree = Tree::new()?;
        let mut app = Self {
            state: AppState::new(tree),
        };
        app.update_preview();
        Ok(app)
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()>
    where
        B::Error: std::error::Error + Send + Sync + 'static,
    {
        while self.state.running {
            terminal.draw(|frame| layout::render(frame, &self.state))?;
            self.handle_events()?;
        }
        self.persist();
        Ok(())
    }

    fn persist(&self) {
        let _ = self.state.config.save();
    }

    fn exit_search(&mut self) {
        if matches!(self.state.mode, InputMode::Search(_)) {
            self.state.mode = InputMode::Normal;
        }
    }

    fn update_preview(&mut self) {
        let cursor = self.state.dir_ui.cursor;
        let should_update = match self.state.preview.cached_path.as_ref() {
            None => true,
            Some(cached) => self
                .state
                .tree
                .selected_node(cursor)
                .is_none_or(|n| &n.path != cached),
        };
        if should_update {
            if let Some(node) = self.state.tree.selected_node(cursor) {
                match preview::preview_file(&node.path) {
                    Ok(p) => {
                        self.state.preview.cached_path = Some(node.path.clone());
                        self.state.preview.lines = p.lines;
                        self.state.preview.total_lines = p.total_lines;
                        self.state.preview.truncated = p.truncated;
                        self.state.preview.file_type = p.file_type;
                        self.state.preview.is_image = p.is_image;
                    }
                    Err(e) => {
                        self.state.preview.cached_path = Some(node.path.clone());
                        self.state.preview.lines = vec![format!("[ERROR] {}", e)];
                        self.state.preview.total_lines = 1;
                        self.state.preview.truncated = false;
                        self.state.preview.file_type = "error".to_string();
                        self.state.preview.is_image = false;
                    }
                }
                self.state.selected_metadata = metadata::get_metadata(&node.path).ok();
            } else {
                self.state.preview.cached_path = None;
                self.state.preview.lines.clear();
                self.state.preview.total_lines = 0;
                self.state.preview.truncated = false;
                self.state.preview.file_type.clear();
                self.state.preview.is_image = false;
                self.state.selected_metadata = None;
            }
        }
    }

    fn reload_tree(&mut self) {
        let dir = self.state.tree.current_dir.clone();
        self.state.tree.invalidate_cache(&dir);
        if let Err(e) = self.state.tree.reload() {
            self.state.errors.push(format!("reload error: {}", e));
        }
        self.update_preview();
    }

    fn advance_animation(&mut self) {
        let Some(path) = self.state.preview.cached_path.clone() else {
            return;
        };
        if !self.state.preview.is_image {
            return;
        }
        let mut cache = self.state.image_cache.borrow_mut();
        let Ok(decoded) = cache.get_or_load(&path) else {
            return;
        };
        if !decoded.is_animated || decoded.frames.is_empty() {
            return;
        }
        let current = self.state.animation.current_frame;
        if current >= decoded.frames.len() {
            self.state.animation.current_frame = 0;
            return;
        }
        let delay = decoded.frames[current].delay_ms;
        let elapsed = self.state.animation.last_update.elapsed();
        if elapsed >= Duration::from_millis(delay as u64) {
            let next = (current + 1) % decoded.frames.len();
            self.state.animation.current_frame = next;
            self.state.animation.last_update = Instant::now();
        }
    }

    fn handle_events(&mut self) -> Result<()> {
        let poll_ms = if self.needs_tick() { 50 } else { 100 };
        if crossterm::event::poll(Duration::from_millis(poll_ms))? {
            let event = crossterm::event::read()?;
            match event {
                crossterm::event::Event::Key(key) => {
                    if let Some(action) = keyboard::map_key(key, &self.state.mode, &self.state.config.keys) {
                        self.dispatch(action)?;
                    }
                }
                crossterm::event::Event::Mouse(me) => {
                    if let Some(action) = mouse::map_mouse(me) {
                        self.dispatch(action)?;
                    }
                }
                _ => {}
            }
            return Ok(());
        }
        if self.needs_tick() {
            self.dispatch(Action::Tick)?;
        }
        Ok(())
    }

    fn needs_tick(&self) -> bool {
        if !self.state.preview.is_image {
            return false;
        }
        let Some(path) = &self.state.preview.cached_path else {
            return false;
        };
        let mut cache = self.state.image_cache.borrow_mut();
        cache.is_path_animated(path)
    }

    fn navigate_to(&mut self, target: PathBuf) {
        self.exit_search();
        let prev = self.state.tree.current_dir.clone();
        if self.state.tree.navigate_to(target).is_ok() {
            self.state.history.visit(prev);
            self.state.dir_ui.reset();
            self.state.tree_ui.reset();
        }
    }

    fn cursor_file(&self) -> Option<PathBuf> {
        self.state
            .tree
            .selected_node(self.state.dir_ui.cursor)
            .map(|n| n.path.clone())
    }

    fn target_paths(&self) -> Vec<PathBuf> {
        if self.state.selection.is_empty() {
            self.cursor_file().into_iter().collect()
        } else {
            self.state.selection.items().cloned().collect()
        }
    }

    fn execute_pending(&mut self, input: Option<String>) {
        let action = match &self.state.mode {
            InputMode::Input { action, .. } => action.clone(),
            InputMode::Confirm { action, .. } => action.clone(),
            InputMode::Normal => return,
            InputMode::Search(_) => return,
        };

        let result = match (action, input) {
            (PendingAction::Rename(path), Some(new_name)) => {
                operations::rename(&path, &new_name).map_err(|e| format!("rename: {}", e))
            }
            (PendingAction::NewFile, Some(name)) => operations::create_file(
                &self.state.tree.current_dir,
                &name,
            )
            .map_err(|e| format!("create file: {}", e)),
            (PendingAction::NewDir, Some(name)) => operations::create_dir(
                &self.state.tree.current_dir,
                &name,
            )
            .map_err(|e| format!("create dir: {}", e)),
            (PendingAction::Delete(paths), None) => {
                operations::delete(&paths).map_err(|e| format!("delete: {}", e))
            }
            (PendingAction::Paste(clip), None) => self.paste_clipboard(clip),
            _ => Ok(()),
        };

        match result {
            Ok(()) => {
                self.reload_tree();
            }
            Err(e) => {
                self.state.errors.push(e);
            }
        }

        self.state.mode = InputMode::Normal;
    }

    fn paste_clipboard(&self, clip: Clipboard) -> std::result::Result<(), String> {
        let dst = &self.state.tree.current_dir;
        for item in &clip.items {
            let result = match clip.operation {
                ClipOp::Copy => operations::copy_to(item, dst),
                ClipOp::Cut => operations::move_to(item, dst),
            };
            if let Err(e) = result {
                return Err(format!("paste {}: {}", item.display(), e));
            }
        }
        Ok(())
    }

    fn dispatch(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Quit => {
                self.persist();
                self.state.running = false;
            }
            Action::Tick => {
                self.advance_animation();
            }

            Action::FocusNext => {
                self.state.focus = self.state.focus.next();
            }
            Action::FocusPrev => {
                self.state.focus = self.state.focus.prev();
            }
            Action::MouseClick(col, row) => {
                let Ok((term_w, term_h)) = crossterm::terminal::size() else {
                    return Ok(());
                };
                if row >= 1 && row < term_h.saturating_sub(6) {
                    let x_pct = (col as f64 / term_w.max(1) as f64) * 100.0;
                    self.state.focus = if x_pct < 20.0 {
                        Focus::Tree
                    } else if x_pct < 70.0 {
                        Focus::Directory
                    } else {
                        Focus::Preview
                    };
                }
            }

            Action::MoveUp => {
                if let InputMode::Search(s) = &mut self.state.mode {
                    s.navigate_up();
                    return Ok(());
                }
                let prev_cursor = self.state.dir_ui.cursor;
                match self.state.focus {
                    Focus::Directory => self.state.dir_ui.navigate_up(),
                    Focus::Tree => self.state.tree_ui.navigate_up(),
                    Focus::Preview => self.state.preview_ui.navigate_up(),
                }
                if self.state.focus == Focus::Directory
                    && self.state.dir_ui.cursor != prev_cursor
                {
                    self.update_preview();
                }
            }
            Action::MoveDown => {
                if let InputMode::Search(s) = &mut self.state.mode {
                    s.navigate_down();
                    return Ok(());
                }
                let prev_cursor = self.state.dir_ui.cursor;
                let total = match self.state.focus {
                    Focus::Directory => self.state.tree.nodes_len(),
                    Focus::Tree => self.state.tree.path_components().len(),
                    Focus::Preview => self.state.preview.lines.len(),
                };
                match self.state.focus {
                    Focus::Directory => self.state.dir_ui.navigate_down(total),
                    Focus::Tree => self.state.tree_ui.navigate_down(total),
                    Focus::Preview => self.state.preview_ui.navigate_down(total),
                }
                if self.state.focus == Focus::Directory
                    && self.state.dir_ui.cursor != prev_cursor
                {
                    self.update_preview();
                }
            }

            Action::EnterDir => match self.state.focus {
                Focus::Directory => {
                    self.exit_search();
                    let cursor = self.state.dir_ui.cursor;
                    let prev_dir = self.state.tree.current_dir.clone();
                    if self.state.tree.enter_dir(cursor)? {
                        self.state.history.visit(prev_dir);
                        self.state.dir_ui.reset();
                        self.state.tree_ui.reset();
                        self.update_preview();
                    }
                }
                Focus::Tree => {
                    self.exit_search();
                    let components = self.state.tree.path_components();
                    let cursor = self.state.tree_ui.cursor;
                    if let Some(path) = components.get(cursor)
                        && *path != self.state.tree.current_dir
                    {
                        let prev = self.state.tree.current_dir.clone();
                        if self.state.tree.navigate_to(path.clone()).is_ok() {
                            self.state.history.visit(prev);
                            self.state.dir_ui.reset();
                            self.state.tree_ui.reset();
                            self.update_preview();
                        }
                    }
                }
                Focus::Preview => {}
            },
            Action::ParentDir => {
                self.exit_search();
                let name = self.state.tree.parent_dir()?;
                self.state.dir_ui.reset();
                self.state.tree_ui.reset();
                if let Some(ref name) = name
                    && let Some(pos) =
                        self.state.tree.nodes.iter().position(|n| n.name == *name)
                {
                    self.state.dir_ui.cursor = pos;
                }
                self.update_preview();
            }

            Action::ToggleSelect => {
                if let Some(node) =
                    self.state.tree.selected_node(self.state.dir_ui.cursor)
                {
                    self.state.selection.toggle(node.path.clone());
                }
            }
            Action::SelectAll => {
                for node in &self.state.tree.nodes {
                    self.state.selection.add(node.path.clone());
                }
            }
            Action::ClearSelection => {
                self.state.selection.clear();
            }

            Action::HistoryBack => {
                self.exit_search();
                let current = self.state.tree.current_dir.clone();
                if let Some(target) = self.state.history.go_back(current) {
                    match self.state.tree.navigate_to(target) {
                        Err(e) => self.state.errors.push(format!("{}", e)),
                        Ok(()) => {
                            self.state.dir_ui.reset();
                            self.state.tree_ui.reset();
                            self.update_preview();
                        }
                    }
                }
            }
            Action::HistoryForward => {
                self.exit_search();
                let current = self.state.tree.current_dir.clone();
                if let Some(target) = self.state.history.go_forward(current) {
                    match self.state.tree.navigate_to(target) {
                        Err(e) => self.state.errors.push(format!("{}", e)),
                        Ok(()) => {
                            self.state.dir_ui.reset();
                            self.state.tree_ui.reset();
                            self.update_preview();
                        }
                    }
                }
            }

            Action::GoHome => {
                if let Some(home) = home_dir() {
                    self.navigate_to(home);
                }
                self.update_preview();
            }
            Action::GoRoot => {
                self.navigate_to(PathBuf::from("/"));
                self.update_preview();
            }
            Action::GoToFirst => {
                self.state.dir_ui.cursor = 0;
                self.state.dir_ui.scroll.set(0);
                self.update_preview();
            }
            Action::GoToLast => {
                let total = self.state.tree.nodes_len();
                if total > 0 {
                    self.state.dir_ui.cursor = total - 1;
                }
                self.update_preview();
            }

            Action::CopyToClipboard => {
                let paths = self.target_paths();
                if paths.is_empty() {
                    self.state.errors.push("nothing to copy".to_string());
                } else {
                    let msg = if paths.len() == 1 {
                        "copied 1 item to clipboard".to_string()
                    } else {
                        format!("copied {} items to clipboard", paths.len())
                    };
                    self.state.clipboard = Some(Clipboard::new(paths, ClipOp::Copy));
                    self.state.errors.push(msg);
                }
            }
            Action::CutToClipboard => {
                let paths = self.target_paths();
                if paths.is_empty() {
                    self.state.errors.push("nothing to cut".to_string());
                } else {
                    let msg = if paths.len() == 1 {
                        "cut 1 item to clipboard".to_string()
                    } else {
                        format!("cut {} items to clipboard", paths.len())
                    };
                    self.state.clipboard = Some(Clipboard::new(paths, ClipOp::Cut));
                    self.state.errors.push(msg);
                }
            }
            Action::PasteClipboard => {
                let clip = self.state.clipboard.clone();
                match clip {
                    None => {
                        self.state.errors.push("clipboard is empty".to_string());
                    }
                    Some(c) if c.is_empty() => {
                        self.state.errors.push("clipboard is empty".to_string());
                    }
                    Some(c) => {
                        self.state.mode = InputMode::Confirm {
                            message: format!("Paste {} {}items? [y/N]", c.items.len(), 
                                match c.operation {
                                    ClipOp::Copy => "",
                                    ClipOp::Cut => "(move) ",
                                }),
                            action: PendingAction::Paste(c),
                        };
                    }
                }
            }
            Action::Rename => {
                let path = match self.cursor_file() {
                    Some(p) => p,
                    None => {
                        self.state.errors.push("no file selected".to_string());
                        return Ok(());
                    }
                };
                let current_name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                self.state.mode = InputMode::Input {
                    prompt: "Rename: ".to_string(),
                    buffer: current_name,
                    action: PendingAction::Rename(path),
                };
            }
            Action::Delete => {
                let paths = self.target_paths();
                if paths.is_empty() {
                    self.state.errors.push("nothing to delete".to_string());
                    return Ok(());
                }
                let msg = if paths.len() == 1 {
                    format!(
                        "Delete \"{}\"? [y/N]",
                        paths[0]
                            .file_name()
                            .map(|n| n.to_string_lossy())
                            .unwrap_or_default()
                    )
                } else {
                    format!("Delete {} items? [y/N]", paths.len())
                };
                self.state.mode = InputMode::Confirm {
                    message: msg,
                    action: PendingAction::Delete(paths),
                };
            }
            Action::NewFile => {
                self.state.mode = InputMode::Input {
                    prompt: "New file: ".to_string(),
                    buffer: String::new(),
                    action: PendingAction::NewFile,
                };
            }
            Action::NewDir => {
                self.state.mode = InputMode::Input {
                    prompt: "New directory: ".to_string(),
                    buffer: String::new(),
                    action: PendingAction::NewDir,
                };
            }

            Action::ToggleSearch => {
                self.state.mode = match &self.state.mode {
                    InputMode::Search(_) => InputMode::Normal,
                    _ => {
                        let mut search = crate::app::search::SearchState::new();
                        search.execute(&self.state.tree.nodes);
                        InputMode::Search(search)
                    }
                };
            }
            Action::SearchChar(c) => {
                if let InputMode::Search(ref mut s) = self.state.mode {
                    s.query.push(c);
                    s.execute(&self.state.tree.nodes);
                }
            }
            Action::SearchBackspace => {
                if let InputMode::Search(ref mut s) = self.state.mode {
                    s.query.pop();
                    s.execute(&self.state.tree.nodes);
                }
            }
            Action::CycleSearchKind => {
                if let InputMode::Search(ref mut s) = self.state.mode {
                    s.kind = s.kind.next();
                    s.execute(&self.state.tree.nodes);
                }
            }

            Action::CycleRenderer => {
                self.state.renderer_kind = self.state.renderer_kind.next();
                self.state.errors.push(format!(
                    "Renderer: {}",
                    self.state.renderer_kind.name()
                ));
                self.update_preview();
            }

            Action::InputChar(c) => {
                if let InputMode::Input { ref mut buffer, .. } = self.state.mode {
                    buffer.push(c);
                }
            }
            Action::InputBackspace => {
                if let InputMode::Input { ref mut buffer, .. } = self.state.mode {
                    buffer.pop();
                }
            }
            Action::InputSubmit => {
                let input = match &self.state.mode {
                    InputMode::Input { buffer, .. } => Some(buffer.clone()),
                    _ => None,
                };
                self.execute_pending(input);
            }
            Action::InputCancel => {
                self.state.mode = InputMode::Normal;
            }
            Action::ConfirmYes => {
                self.execute_pending(None);
            }
            Action::ConfirmNo => {
                self.state.mode = InputMode::Normal;
            }
        }
        Ok(())
    }
}
