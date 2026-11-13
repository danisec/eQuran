//! Tauri commands backing the local full-text Quran search.
//!
//! The index is built lazily, never during app setup. The first search (or an
//! explicit rebuild) triggers a build that runs while holding the write half of
//! an [`RwLock`], so a build happens at most once concurrently.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use equran_core::domain::Lang;
use equran_core::search::{SearchHit, SearchIndex};
use serde::Serialize;
use tauri::{Emitter, State, Window};
use tokio::sync::RwLock;

use crate::AppState;

/// Managed application state holding the lazily built search index.
pub struct SearchState {
    index: RwLock<Option<Arc<SearchIndex>>>,
    building: AtomicBool,
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchState {
    /// Creates empty state; no index is loaded or built until first use.
    pub fn new() -> Self {
        Self {
            index: RwLock::new(None),
            building: AtomicBool::new(false),
        }
    }

    /// Loads a valid on-disk index into memory without touching the network.
    ///
    /// Called once at startup so the home page can report a real ayah count
    /// and the first query is instant. Without this the count read `0` until
    /// the reader typed something, even though the index was already cached.
    pub async fn hydrate(&self) -> bool {
        if self.index.read().await.is_some() {
            return true;
        }
        match equran_core::search::load_cached().await {
            Ok(Some(index)) => {
                *self.index.write().await = Some(Arc::new(index));
                true
            }
            _ => false,
        }
    }
}

/// Snapshot of the search index availability, returned to the frontend.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexStatus {
    /// True when an index is currently loaded in memory.
    pub ready: bool,
    /// Number of ayahs in the loaded index (0 when not ready).
    pub ayah_count: usize,
    /// Unix build timestamp of the loaded index, when available.
    pub built_at: Option<u64>,
    /// True while a build is in progress.
    pub building: bool,
}

/// Collects the current status without blocking on an in-flight build.
async fn status_of(state: &SearchState) -> IndexStatus {
    let building = state.building.load(Ordering::SeqCst);
    match state.index.try_read() {
        Ok(guard) => match guard.as_ref() {
            Some(index) => IndexStatus {
                ready: true,
                ayah_count: index.len(),
                built_at: Some(index.built_at),
                building,
            },
            None => IndexStatus {
                ready: false,
                ayah_count: 0,
                built_at: None,
                building,
            },
        },
        Err(_) => IndexStatus {
            ready: false,
            ayah_count: 0,
            built_at: None,
            building: true,
        },
    }
}

/// Returns the loaded index, building it if it is not present yet.
///
/// Uses the exclusive write lock to guarantee a build happens at most once
/// concurrently; a competing caller simply waits and then observes the result.
async fn ensure_index(
    api: &equran_core::api::client::ApiClient,
    state: &SearchState,
    rebuild: bool,
    progress: impl Fn(usize, usize) + Send + Sync,
) -> anyhow::Result<Arc<SearchIndex>> {
    if !rebuild && let Some(index) = state.index.read().await.as_ref() {
        return Ok(Arc::clone(index));
    }

    let mut guard = state.index.write().await;
    if !rebuild && let Some(index) = guard.as_ref() {
        return Ok(Arc::clone(index));
    }

    state.building.store(true, Ordering::SeqCst);
    let built = equran_core::search::load_or_build(api, rebuild, progress).await;
    state.building.store(false, Ordering::SeqCst);

    let index = Arc::new(built?);
    *guard = Some(Arc::clone(&index));
    Ok(index)
}

/// Searches the Quran, building the index on first use when necessary.
///
/// `lang` decides which fields are searchable: the English mode never matches
/// or quotes the Indonesian tafsir.
#[tauri::command]
pub async fn search_quran(
    state: State<'_, AppState>,
    search: State<'_, SearchState>,
    query: String,
    limit: Option<usize>,
    lang: Option<String>,
) -> Result<Vec<SearchHit>, String> {
    let limit = limit.unwrap_or(50);
    let lang: Lang = lang.unwrap_or_else(|| "id".to_owned()).parse()?;
    let index = ensure_index(state.api(), &search, false, |_, _| {})
        .await
        .map_err(|error| error.to_string())?;
    Ok(index.search_for(&query, limit, lang))
}

/// Reports whether the search index is ready, how large it is, and whether a
/// build is currently running.
///
/// Loads a cached index on first call so the reader sees a real count before
/// searching. This never triggers a network build.
#[tauri::command]
pub async fn get_index_status(search: State<'_, SearchState>) -> Result<IndexStatus, String> {
    if !status_of(&search).await.ready {
        search.hydrate().await;
    }
    Ok(status_of(&search).await)
}

/// Forces a full rebuild of the search index, emitting `index-progress` events.
#[tauri::command]
pub async fn rebuild_index(
    window: Window,
    state: State<'_, AppState>,
    search: State<'_, SearchState>,
) -> Result<IndexStatus, String> {
    let progress = move |done: usize, total: usize| {
        let _ = window.emit(
            "index-progress",
            serde_json::json!({ "done": done, "total": total }),
        );
    };
    ensure_index(state.api(), &search, true, progress)
        .await
        .map_err(|error| error.to_string())?;
    Ok(status_of(&search).await)
}
