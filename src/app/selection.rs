use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Selection {
    items: HashSet<PathBuf>,
}

impl Selection {
    pub fn new() -> Self {
        Self {
            items: HashSet::new(),
        }
    }

    pub fn add(&mut self, path: PathBuf) {
        self.items.insert(path);
    }

    pub fn toggle(&mut self, path: PathBuf) {
        if !self.items.remove(&path) {
            self.items.insert(path);
        }
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn contains(&self, path: &Path) -> bool {
        self.items.contains(path)
    }

    pub fn count(&self) -> usize {
        self.items.len()
    }

    #[allow(dead_code)]
    pub fn items(&self) -> impl Iterator<Item = &PathBuf> {
        self.items.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
