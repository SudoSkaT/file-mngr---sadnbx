use std::cell::RefCell;
use std::path::PathBuf;
use std::time::Instant;

use crate::fs::metadata::FileMetadata;
use crate::fs::tree::Tree;
use crate::config::Config;
use crate::image::cache::ImageCache;
use crate::image::render::RendererKind;

use super::search::SearchState;

use super::errors::Errors;
use super::focus::Focus;
use super::history::History;
use super::panel::PanelState;
use super::selection::Selection;

#[derive(Clone, Debug, PartialEq)]
pub enum ClipOp {
    Copy,
    Cut,
}

#[derive(Clone, Debug)]
pub struct Clipboard {
    pub items: Vec<PathBuf>,
    pub operation: ClipOp,
}

impl Clipboard {
    pub fn new(items: Vec<PathBuf>, operation: ClipOp) -> Self {
        Self { items, operation }
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[derive(Clone, Debug)]
pub enum PendingAction {
    Rename(PathBuf),
    NewFile,
    NewDir,
    Delete(Vec<PathBuf>),
    Paste(Clipboard),
}

#[derive(Clone, Debug)]
pub enum InputMode {
    Normal,
    Input {
        prompt: String,
        buffer: String,
        action: PendingAction,
    },
    Confirm {
        message: String,
        action: PendingAction,
    },
    Search(SearchState),
}

pub struct PreviewState {
    pub cached_path: Option<PathBuf>,
    pub lines: Vec<String>,
    pub total_lines: usize,
    pub truncated: bool,
    pub file_type: String,
    pub is_image: bool,
}

impl PreviewState {
    pub fn new() -> Self {
        Self {
            cached_path: None,
            lines: Vec::new(),
            total_lines: 0,
            truncated: false,
            file_type: String::new(),
            is_image: false,
        }
    }
}

pub struct AnimationState {
    pub current_frame: usize,
    pub last_update: Instant,
}

impl AnimationState {
    pub fn new() -> Self {
        Self {
            current_frame: 0,
            last_update: Instant::now(),
        }
    }
}

pub struct AppState {
    pub running: bool,
    pub focus: Focus,
    pub tree: Tree,
    pub dir_ui: PanelState,
    pub tree_ui: PanelState,
    pub preview_ui: PanelState,
    pub history: History,
    pub selection: Selection,
    pub errors: Errors,
    pub preview: PreviewState,
    pub selected_metadata: Option<FileMetadata>,
    pub clipboard: Option<Clipboard>,
    pub mode: InputMode,
    pub image_cache: RefCell<ImageCache>,
    pub animation: AnimationState,
    pub renderer_kind: RendererKind,
    pub config: Config,
}

impl AppState {
    pub fn new(tree: Tree) -> Self {
        Self {
            running: true,
            focus: Focus::Directory,
            tree,
            dir_ui: PanelState::new(),
            tree_ui: PanelState::new(),
            preview_ui: PanelState::new(),
            history: History::new(),
            selection: Selection::new(),
            errors: Errors::new(),
            preview: PreviewState::new(),
            selected_metadata: None,
            clipboard: None,
            mode: InputMode::Normal,
            image_cache: RefCell::new(ImageCache::new(32)),
            animation: AnimationState::new(),
            renderer_kind: RendererKind::Normal,
            config: Config::load(),
        }
    }
}
