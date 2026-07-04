use std::path::Path;

use color_eyre::Result;

use super::node::Node;

#[cfg(test)]
mod bench {
    use std::time::Instant;
    use super::read_dir;

    #[test]
    fn bench_read_dir() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let start = Instant::now();
        let nodes = read_dir(&path).expect("read_dir");
        let elapsed = start.elapsed();
        println!("[bench] read_dir {} items: {:?}", nodes.len(), elapsed);
        assert!(elapsed.as_millis() < 500, "read_dir too slow: {:?}", elapsed);
    }
}

pub fn read_dir(path: &Path) -> Result<Vec<Node>> {
    let entries = std::fs::read_dir(path)?;
    let entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    let mut nodes = Vec::with_capacity(entries.len());

    for entry in entries {
        if let Ok(node) = Node::from_entry(entry) {
            nodes.push(node);
        }
    }

    nodes.sort_by_cached_key(|n| (!n.is_dir, n.name.to_lowercase()));

    Ok(nodes)
}
