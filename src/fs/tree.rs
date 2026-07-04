use std::collections::HashMap;
use std::path::PathBuf;

use color_eyre::Result;

use super::node::Node;
use super::walker;

pub struct Tree {
    pub current_dir: PathBuf,
    pub nodes: Vec<Node>,
    cache: HashMap<PathBuf, Vec<Node>>,
    max_cache: usize,
}

impl Tree {
    pub fn new() -> Result<Self> {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        let mut tree = Self {
            current_dir,
            nodes: Vec::new(),
            cache: HashMap::new(),
            max_cache: 64,
        };
        tree.reload()?;
        Ok(tree)
    }

    pub fn reload(&mut self) -> Result<()> {
        if let Some(cached) = self.cache.get(&self.current_dir) {
            self.nodes = cached.clone();
            return Ok(());
        }
        self.nodes = walker::read_dir(&self.current_dir)?;
        self.cache.insert(self.current_dir.clone(), self.nodes.clone());
        if self.cache.len() > self.max_cache {
            self.cache.clear();
        }
        Ok(())
    }

    pub fn invalidate_cache(&mut self, path: &PathBuf) {
        self.cache.remove(path);
    }

    pub fn selected_node(&self, cursor: usize) -> Option<&Node> {
        self.nodes.get(cursor)
    }

    pub fn nodes_len(&self) -> usize {
        self.nodes.len()
    }

    pub fn enter_dir(&mut self, cursor: usize) -> Result<bool> {
        if let Some(node) = self.nodes.get(cursor)
            && node.is_dir
        {
            self.current_dir = node.path.clone();
            self.reload()?;
            return Ok(true);
        }
        Ok(false)
    }

    pub fn parent_dir(&mut self) -> Result<Option<String>> {
        let previous = self
            .current_dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned());

        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.reload()?;
        }
        Ok(previous)
    }

    pub fn path_components(&self) -> Vec<PathBuf> {
        let mut components = Vec::new();
        let mut current = Some(self.current_dir.as_path());
        while let Some(path) = current {
            if path.as_os_str().is_empty() {
                break;
            }
            components.push(path.to_path_buf());
            current = path.parent();
        }
        components.reverse();
        components
    }

    pub fn navigate_to(&mut self, path: PathBuf) -> Result<()> {
        self.current_dir = path;
        self.reload()
    }
}
