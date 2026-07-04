use std::fs;
use std::path::Path;
use std::path::PathBuf;

use color_eyre::Result;

pub fn copy_to(src: &Path, dst_dir: &Path) -> Result<()> {
    let dest = dst_dir.join(
        src.file_name().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid file name")
        })?,
    );
    if dest.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("destination exists: {}", dest.display()),
        )
        .into());
    }
    copy_recursively(src, &dest)
}

pub fn move_to(src: &Path, dst_dir: &Path) -> Result<()> {
    let dest = dst_dir.join(
        src.file_name().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid file name")
        })?,
    );
    if dest.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("destination exists: {}", dest.display()),
        )
        .into());
    }
    if fs::rename(src, &dest).is_err() {
        copy_recursively(src, &dest)?;
        delete_recursively(src)?;
    }
    Ok(())
}

pub fn rename(path: &Path, new_name: &str) -> Result<()> {
    if new_name.is_empty() {
        return Err(
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "name cannot be empty").into(),
        );
    }
    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "cannot rename root")
    })?;
    let dest = parent.join(new_name);
    if dest.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("destination exists: {}", dest.display()),
        )
        .into());
    }
    fs::rename(path, &dest)?;
    Ok(())
}

pub fn delete(paths: &[PathBuf]) -> Result<()> {
    for p in paths {
        if !p.exists() {
            continue;
        }
        delete_recursively(p).map_err(|e| {
                std::io::Error::other(
                    format!("failed to delete {}: {}", p.display(), e),
                )
        })?;
    }
    Ok(())
}

pub fn create_file(path: &Path, name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "name cannot be empty").into(),
        );
    }
    let dest = path.join(name);
    if dest.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("file exists: {}", dest.display()),
        )
        .into());
    }
    fs::write(&dest, "")?;
    Ok(())
}

pub fn create_dir(path: &Path, name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "name cannot be empty").into(),
        );
    }
    let dest = path.join(name);
    if dest.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("directory exists: {}", dest.display()),
        )
        .into());
    }
    fs::create_dir(&dest)?;
    Ok(())
}

fn copy_recursively(src: &Path, dst: &Path) -> Result<()> {
    if src.is_dir() {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let child_src = entry.path();
            let child_dst = dst.join(entry.file_name());
            copy_recursively(&child_src, &child_dst)?;
        }
    } else {
        fs::copy(src, dst)?;
    }
    Ok(())
}

fn delete_recursively(path: &Path) -> Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            delete_recursively(&entry.path())?;
        }
        fs::remove_dir(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
}
