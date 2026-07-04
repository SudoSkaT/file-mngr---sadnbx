use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct FileMetadata {
    pub name: String,
    pub size: String,
    pub modified: String,
    pub permissions: String,
    pub file_type: String,
    pub user_name: String,
    pub group_name: String,
    pub entry_count: Option<usize>,
    pub fs_info: Option<FsInfo>,
}

pub struct FsInfo {
    pub fs_type: String,
    pub total: String,
    pub used: String,
    pub avail: String,
}

pub fn get_metadata(path: &Path) -> std::io::Result<FileMetadata> {
    let meta = fs::symlink_metadata(path)?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());

    let is_dir = path.is_dir();
    let file_type = if path.is_symlink() {
        "symlink".to_string()
    } else if is_dir {
        "directory".to_string()
    } else if path.is_file() {
        "file".to_string()
    } else {
        "other".to_string()
    };

    let entry_count = if is_dir {
        fs::read_dir(path).ok().map(|d| d.count())
    } else {
        None
    };

    let permissions = format_permissions(meta.mode(), is_dir, path.is_symlink());
    let user_name = resolve_user(meta.uid());
    let group_name = resolve_group(meta.gid());
    let size = human_size(meta.len());
    let modified = format_time(meta.modified().unwrap_or(UNIX_EPOCH));
    let fs_info = get_fs_info(path);

    Ok(FileMetadata {
        name,
        size,
        modified,
        permissions,
        file_type,
        user_name,
        group_name,
        entry_count,
        fs_info,
    })
}

fn format_permissions(mode: u32, is_dir: bool, is_symlink: bool) -> String {
    let dir_char = if is_symlink {
        'l'
    } else if is_dir {
        'd'
    } else {
        '-'
    };
    let owner_r = if mode & 0o400 != 0 { 'r' } else { '-' };
    let owner_w = if mode & 0o200 != 0 { 'w' } else { '-' };
    let owner_x = if mode & 0o100 != 0 { 'x' } else { '-' };
    let group_r = if mode & 0o040 != 0 { 'r' } else { '-' };
    let group_w = if mode & 0o020 != 0 { 'w' } else { '-' };
    let group_x = if mode & 0o010 != 0 { 'x' } else { '-' };
    let other_r = if mode & 0o004 != 0 { 'r' } else { '-' };
    let other_w = if mode & 0o002 != 0 { 'w' } else { '-' };
    let other_x = if mode & 0o001 != 0 { 'x' } else { '-' };

    format!(
        "{}{}{}{}{}{}{}{}{}{}",
        dir_char, owner_r, owner_w, owner_x,
        group_r, group_w, group_x,
        other_r, other_w, other_x
    )
}

fn resolve_user(uid: u32) -> String {
    let content = fs::read_to_string("/etc/passwd").unwrap_or_default();
    for line in content.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() > 2
            && let Ok(id) = parts[2].parse::<u32>()
            && id == uid
        {
            return parts[0].to_string();
        }
    }
    uid.to_string()
}

fn resolve_group(gid: u32) -> String {
    let content = fs::read_to_string("/etc/group").unwrap_or_default();
    for line in content.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() > 2
            && let Ok(id) = parts[2].parse::<u32>()
            && id == gid
        {
            return parts[0].to_string();
        }
    }
    gid.to_string()
}

fn human_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut idx = 0;
    while size >= 1024.0 && idx < UNITS.len() - 1 {
        size /= 1024.0;
        idx += 1;
    }
    if idx == 0 {
        format!("{} {}", bytes, UNITS[idx])
    } else {
        format!("{:.1} {}", size, UNITS[idx])
    }
}

fn format_time(time: SystemTime) -> String {
    let secs = time.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
    if secs < 0 {
        return "unknown".to_string();
    }
    let secs = secs as u64;
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let h = time_secs / 3600;
    let m = (time_secs % 3600) / 60;
    let s = time_secs % 60;
    let (y, mo, d) = civil_from_days(days as i64);
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, mo, d, h, m, s)
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

fn get_fs_info(path: &Path) -> Option<FsInfo> {
    let output = Command::new("stat")
        .args(["-f", "--format=%T %S %b %a"])
        .arg(path)
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return None;
    }
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.len() < 4 {
        return None;
    }
    let fs_type = parts[0].to_string();
    let block_size: u64 = parts[1].parse().ok()?;
    let total_blocks: u64 = parts[2].parse().ok()?;
    let avail_blocks: u64 = parts[3].parse().ok()?;
    let total = total_blocks.saturating_mul(block_size);
    let avail = avail_blocks.saturating_mul(block_size);
    let used = total.saturating_sub(avail);
    Some(FsInfo {
        fs_type,
        total: human_size(total),
        used: human_size(used),
        avail: human_size(avail),
    })
}
