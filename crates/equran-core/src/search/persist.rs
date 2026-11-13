//! Atomic on-disk persistence for the index.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context, Result};
use tokio::fs;

use crate::api::client::ApiClient;

use super::build::build;
use super::{INDEX_VERSION, SearchIndex};

/// Counter used to make atomic-save temporary files unique within a process.
static PART_COUNTER: AtomicU64 = AtomicU64::new(0);

// ---------------------------------------------------------------------------
// Persistence
// ---------------------------------------------------------------------------

/// Returns the path of the on-disk index:
/// `<cache_dir>/equran-cli/index/quran-index.json`.
pub fn index_path() -> Result<PathBuf> {
    let cache_root = dirs::cache_dir()
        .context("failed to resolve user cache directory")?
        .join("equran-cli")
        .join("index");
    Ok(cache_root.join("quran-index.json"))
}

/// Atomically writes the index to `path` via a unique `.part` file and rename.
pub(super) async fn save_to(path: &Path, index: &SearchIndex) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let json = serde_json::to_string(index).context("failed to serialise the search index")?;

    let counter = PART_COUNTER.fetch_add(1, Ordering::Relaxed);
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("quran-index.json");
    let part_name = format!("{file_name}.part.{}.{}", std::process::id(), counter);
    let mut part_path = path.to_path_buf();
    part_path.set_file_name(part_name);

    fs::write(&part_path, json.as_bytes())
        .await
        .with_context(|| format!("failed to write {}", part_path.display()))?;
    fs::rename(&part_path, path)
        .await
        .with_context(|| format!("failed to move {} into place", part_path.display()))?;
    Ok(())
}

/// Loads an index from `path`.
///
/// A missing, unreadable or corrupt file yields `Ok(None)` so the caller can
/// rebuild instead of failing outright.
pub(super) async fn load_from(path: &Path) -> Result<Option<SearchIndex>> {
    let bytes = match fs::read(path).await {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(error).with_context(|| format!("failed to read {}", path.display()));
        }
    };
    match serde_json::from_slice::<SearchIndex>(&bytes) {
        Ok(index) => Ok(Some(index)),
        Err(_) => Ok(None),
    }
}

/// Loads the persisted index from disk when it exists and matches
/// [`INDEX_VERSION`].
///
/// Returns `Ok(None)` when no valid cached index is available; callers can then
/// fall back to [`load_or_build`] or [`build`].
pub async fn load_cached() -> Result<Option<SearchIndex>> {
    let path = index_path()?;
    match load_from(&path).await? {
        Some(index) if index.version == INDEX_VERSION => Ok(Some(index)),
        _ => Ok(None),
    }
}

/// Loads the cached index when it is valid, otherwise builds and saves it.
///
/// `rebuild` forces a fresh build even when a valid file is present.
pub async fn load_or_build(
    api: &ApiClient,
    rebuild: bool,
    progress: impl Fn(usize, usize) + Send + Sync,
) -> Result<SearchIndex> {
    let path = index_path()?;
    if !rebuild
        && let Some(index) = load_from(&path).await?
        && index.version == INDEX_VERSION
    {
        return Ok(index);
    }
    let index = build(api, progress).await?;
    save_to(&path, &index).await?;
    Ok(index)
}
