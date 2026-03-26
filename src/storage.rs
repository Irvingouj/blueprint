use std::path::{Path, PathBuf};

use tokio::fs;
use tracing::info;

use crate::error::{BlueprintError, Result};

/// Find the project-level .blueprint/ directory by walking up from CWD
/// looking for a .git directory.
pub fn project_storage_dir() -> Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    let mut dir = cwd.as_path();
    loop {
        if dir.join(".git").exists() {
            let bp_dir = dir.join(".blueprint");
            info!(path = %bp_dir.display(), "found project storage");
            return Ok(bp_dir);
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => {
                // No .git found; fall back to CWD/.blueprint/
                let bp_dir = cwd.join(".blueprint");
                info!(path = %bp_dir.display(), "no .git found, using CWD");
                return Ok(bp_dir);
            }
        }
    }
}

/// Global storage: ~/.blueprint/
pub fn global_storage_dir() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| BlueprintError::Other("cannot determine home directory".into()))?;
    Ok(home.join(".blueprint"))
}

/// Get the storage dir based on the --global flag.
pub fn storage_dir(global: bool) -> Result<PathBuf> {
    if global {
        global_storage_dir()
    } else {
        project_storage_dir()
    }
}

/// Ensure the storage directory exists.
pub async fn ensure_dir(dir: &Path) -> Result<()> {
    if !dir.exists() {
        fs::create_dir_all(dir).await?;
        info!(path = %dir.display(), "created storage directory");
    }
    Ok(())
}

/// Write a blueprint file.
pub async fn write_blueprint(dir: &Path, handle: &str, content: &str) -> Result<()> {
    ensure_dir(dir).await?;
    let path = dir.join(format!("{handle}.md"));
    fs::write(&path, content).await?;
    info!(path = %path.display(), "wrote blueprint");
    Ok(())
}

/// Read a blueprint file by handle.
pub async fn read_blueprint(dir: &Path, handle: &str) -> Result<String> {
    let path = dir.join(format!("{handle}.md"));
    if !path.exists() {
        return Err(BlueprintError::NotFound {
            handle: handle.to_string(),
            search_path: dir.display().to_string(),
        });
    }
    Ok(fs::read_to_string(&path).await?)
}

/// List all blueprint handles in a directory.
pub async fn list_handles(dir: &Path) -> Result<Vec<String>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut handles = Vec::new();
    let mut entries = fs::read_dir(dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("md")
            && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
        {
            handles.push(stem.to_string());
        }
    }
    handles.sort();
    Ok(handles)
}
