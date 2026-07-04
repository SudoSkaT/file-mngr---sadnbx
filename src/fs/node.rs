use std::fs::DirEntry;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Node {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
}

impl Node {
    pub fn from_entry(entry: DirEntry) -> std::io::Result<Self> {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let is_dir = entry.file_type()?.is_dir();
        Ok(Self { path, name, is_dir })
    }
}
