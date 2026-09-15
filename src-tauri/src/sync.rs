use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Replace the private Git working tree contents with the selected folder,
/// preserving only .git. This makes `git add -A` correctly represent deletions.
pub fn import_folder(src: &Path, worktree: &Path) -> Result<()> {
    if !src.exists() {
        anyhow::bail!("source folder does not exist");
    }
    fs::create_dir_all(worktree)?;
    clear_dir(worktree, true)?;
    copy_dir(src, worktree, true)?;
    Ok(())
}

/// Replace the selected folder with the Git working tree, excluding .git.
/// This intentionally deletes stale files so Pull is a true mirror operation.
pub fn export_folder(worktree: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    clear_dir(dst, false)?;
    copy_dir(worktree, dst, true)?;
    Ok(())
}

fn clear_dir(dir: &Path, keep_git: bool) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if keep_git && entry.file_name() == ".git" {
            continue;
        }
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}

fn copy_dir(src: &Path, dst: &Path, skip_git: bool) -> Result<()> {
    for entry in fs::read_dir(src).with_context(|| format!("read {}", src.display()))? {
        let entry = entry?;
        let p = entry.path();
        let name = entry.file_name();
        if skip_git && name == ".git" {
            continue;
        }
        let q = dst.join(&name);
        if p.is_dir() {
            fs::create_dir_all(&q)?;
            copy_dir(&p, &q, skip_git)?;
        } else if p.is_file() {
            if let Some(parent) = q.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&p, &q)?;
        }
    }
    Ok(())
}
