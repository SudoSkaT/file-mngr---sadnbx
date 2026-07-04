// Nerd Font glyphs — requires a Nerd Font installed in the terminal.
// Without Nerd Font, these render as fallback boxes.

fn glyph(name: &str) -> char {
    let ext = name.rsplit('.').next().map(|s| s.to_lowercase());
    match ext.as_deref() {
        Some("rs") => '\u{E7A8}',
        Some("md") => '\u{F718}',
        Some("toml" | "yaml" | "yml") => '\u{E615}',
        Some("json") => '\u{E60B}',
        Some("txt" | "text" | "log") => '\u{F15C}',
        Some("sh" | "bash" | "zsh") => '\u{F120}',
        Some("py") => '\u{E73C}',
        Some("js") => '\u{E718}',
        Some("ts") => '\u{E628}',
        Some("css" | "scss" | "sass" | "less") => '\u{E61F}',
        Some("html" | "htm") => '\u{E60E}',
        Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "svg") => '\u{F1C5}',
        Some("mp4" | "avi" | "mkv" | "mov" | "webm" | "flv" | "wmv") => '\u{F1C8}',
        Some("mp3" | "wav" | "flac" | "ogg" | "aac") => '\u{F001}',
        Some("zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar") => '\u{F1C6}',
        Some("pdf") => '\u{F1C1}',
        Some("doc" | "docx") => '\u{F1C2}',
        Some("xls" | "xlsx") => '\u{F1C3}',
        Some("csv") => '\u{F1C0}',
        Some("iso" | "img") => '\u{F1CB}',
        Some("deb" | "rpm" | "apk" | "exe" | "msi") => '\u{F1C9}',
        Some("conf" | "cfg" | "ini" | "env") => '\u{F013}',
        Some("lock") => '\u{F023}',
        Some("gitignore" | "gitattributes") => '\u{E702}',
        Some("go" | "mod" | "sum") => '\u{E724}',
        Some("makefile" | "cmake") => '\u{F120}',
        Some("dockerfile") => '\u{E7B0}',
        _ => '\u{F15B}',
    }
}

pub fn glyph_dir() -> char {
    '\u{E5FF}'
}

pub fn icon_name(name: &str, is_dir: bool) -> String {
    if is_dir {
        glyph_dir().to_string()
    } else {
        glyph(name).to_string()
    }
}
