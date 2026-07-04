use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

use color_eyre::Result;

use super::decoder::DecodedImage;

struct Entry {
    image: DecodedImage,
    modified: SystemTime,
}

pub struct ImageCache {
    entries: HashMap<PathBuf, Entry>,
    max_entries: usize,
}

impl ImageCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: HashMap::new(),
            max_entries,
        }
    }

    pub fn get_or_load(&mut self, path: &Path) -> Result<DecodedImage> {
        let modified = std::fs::metadata(path)
            .ok()
            .and_then(|m| m.modified().ok());

        if let Some(entry) = self.entries.get(path)
            && modified.is_none_or(|m| entry.modified == m)
        {
            return Ok(entry.image.clone());
        }

        let image = super::decoder::decode(path)?;
        let modified = modified.unwrap_or(SystemTime::UNIX_EPOCH);

        if self.entries.len() >= self.max_entries {
            self.evict_one();
        }

        self.entries.insert(
            path.to_path_buf(),
            Entry {
                image: image.clone(),
                modified,
            },
        );

        Ok(image)
    }

    pub fn is_path_animated(&mut self, path: &Path) -> bool {
        if let Some(entry) = self.entries.get(path) {
            return entry.image.is_animated;
        }
        if let Ok(img) = super::decoder::decode(path) {
            let animated = img.is_animated;
            let modified = std::fs::metadata(path)
                .ok()
                .and_then(|m| m.modified().ok())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            if self.entries.len() >= self.max_entries {
                self.evict_one();
            }
            self.entries.insert(
                path.to_path_buf(),
                Entry {
                    image: img,
                    modified,
                },
            );
            return animated;
        }
        false
    }

    fn evict_one(&mut self) {
        if let Some(key) = self.entries.keys().next().cloned() {
            self.entries.remove(&key);
        }
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}
