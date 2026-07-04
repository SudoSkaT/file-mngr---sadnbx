use std::fs;
use std::path::Path;

use color_eyre::Result;

const MAX_BYTES: usize = 8192;
const MAX_LINES: usize = 500;
const BINARY_SCAN: usize = 1024;

pub struct Preview {
    pub lines: Vec<String>,
    pub total_lines: usize,
    pub truncated: bool,
    pub file_type: String,
    #[allow(dead_code)]
    pub is_binary: bool,
    pub is_image: bool,
}

fn detect_type(path: &Path) -> &'static str {
    if path.is_dir() {
        return "directory";
    }
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "txt" | "log" | "text" => "text",
        "md" | "markdown" => "markdown",
        "json" => "json",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "rs" | "py" | "js" | "ts" | "go" | "c" | "h" | "cpp" | "hpp" | "java"
        | "rb" | "sh" | "bash" | "zsh" | "fish" | "lua" | "zig" | "swift"
        | "kt" | "scala" | "php" | "pl" | "r" | "m" | "mm" | "s" | "asm"
        | "sql" | "css" | "scss" | "less" | "html" | "xml" | "svg"
        | "ini" | "cfg" | "conf" | "env" | "gitignore"
        | "dockerfile" | "makefile" | "cmake" | "gradle" | "lock" => "source",
        "mp4" | "avi" | "mkv" | "mov" | "webm" | "flv" | "wmv" => "video",
        _ => "unknown",
    }
}

pub fn preview_file(path: &Path) -> Result<Preview> {
    if path.is_dir() {
        let entries: Vec<_> = fs::read_dir(path)
            .map(|d| {
                d.filter_map(|e| e.ok())
                    .map(|e| e.file_name().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();

        let total = entries.len();
        let truncated = total > MAX_LINES;
        let lines: Vec<String> = entries
            .iter()
            .take(MAX_LINES)
            .map(|n| format!(" {}", n))
            .collect();

        return Ok(Preview {
            lines,
            total_lines: total,
            truncated,
            file_type: "directory".to_string(),
            is_binary: false,
            is_image: false,
        });
    }

    if is_image_extension(path) {
        return Ok(Preview {
            lines: vec![format!("[IMAGE] {}x?", "?")],
            total_lines: 1,
            truncated: false,
            file_type: "image".to_string(),
            is_binary: false,
            is_image: true,
        });
    }

    let data = fs::read(path)?;
    let is_binary = data.iter().take(BINARY_SCAN).any(|&b| b == 0);

    let ft = detect_type(path).to_string();

    if is_binary {
        return Ok(Preview {
            lines: vec!["[BINARY FILE]".to_string()],
            total_lines: 1,
            truncated: false,
            file_type: ft,
            is_binary: true,
            is_image: false,
        });
    }

    let content = String::from_utf8_lossy(&data);
    let total_lines = content.lines().count();
    let truncated = data.len() > MAX_BYTES || total_lines > MAX_LINES;

    let lines: Vec<String> = content
        .lines()
        .take(MAX_LINES)
        .map(|l| {
            if l.len() > 2000 {
                format!("{}...", &l[..2000])
            } else {
                l.to_string()
            }
        })
        .collect();

    Ok(Preview {
        lines,
        total_lines,
        truncated,
        file_type: ft,
        is_binary: false,
        is_image: false,
    })
}

fn is_image_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| {
            matches!(
                e.to_lowercase().as_str(),
                "png" | "jpg" | "jpeg" | "gif" | "webp"
                    | "mp4" | "avi" | "mkv" | "mov" | "webm" | "flv" | "wmv"
            )
        })
}
