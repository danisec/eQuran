pub mod commands;

use std::path::PathBuf;

use commands::bookmark::BookmarkState;
use commands::playback::PlaybackState;
use commands::search::SearchState;
use commands::settings::SettingsState;
use equran_core::api::client::ApiClient;
use equran_core::audio::cache::AudioCache;
use tauri::{Manager, path::BaseDirectory};

#[derive(Clone)]
pub struct AppState {
    api: ApiClient,
    tts_roots: Vec<PathBuf>,
    writable_tts_root: PathBuf,
}

impl AppState {
    pub fn new(
        resource_root: Option<PathBuf>,
        writable_tts_root: Option<PathBuf>,
    ) -> anyhow::Result<Self> {
        let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let writable_tts_root = writable_tts_root.unwrap_or_else(|| project_root.clone());

        // The worker script is part of the binary's contract, so the bundled
        // resource (or the checkout during development) must win. The app-data
        // directory is searched last: it is writable and useful for the Python
        // venv, but a copy left there by an older install must not shadow the
        // script shipped with this build.
        let mut tts_roots = Vec::new();
        if let Some(resource_root) = resource_root {
            tts_roots.push(resource_root);
        }
        if include_project_tts_root() {
            tts_roots.push(project_root);
        }
        tts_roots.push(writable_tts_root.clone());

        Ok(Self {
            api: ApiClient::new()?,
            tts_roots,
            writable_tts_root,
        })
    }

    pub fn api(&self) -> &ApiClient {
        &self.api
    }

    pub fn tts_roots(&self) -> &[PathBuf] {
        &self.tts_roots
    }

    pub fn writable_tts_root(&self) -> &PathBuf {
        &self.writable_tts_root
    }
}

fn include_project_tts_root() -> bool {
    cfg!(debug_assertions)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            let resource_root = app.path().resolve(".", BaseDirectory::Resource).ok();
            let writable_tts_root = app.path().app_data_dir().ok();
            let state = AppState::new(resource_root, writable_tts_root)
                .map_err(|error| error.to_string())?;
            app.manage(state);
            app.manage(PlaybackState::new());
            app.manage(BookmarkState::new());
            app.manage(SettingsState::new());
            app.manage(SearchState::new());

            // Reclaim disk before the first clip, on a worker so the window opens
            // without waiting on a directory walk over a large cache.
            tauri::async_runtime::spawn(async {
                if let Ok(cache) = AudioCache::new() {
                    let report = cache.prune().await;
                    if report.orphan_parts_removed > 0 || report.files_removed > 0 {
                        eprintln!(
                            "cache pruned: {} leftover renders, {} files, {} bytes reclaimed",
                            report.orphan_parts_removed, report.files_removed, report.bytes_freed
                        );
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::surah::get_surah_list,
            commands::surah::get_surah,
            commands::surah::get_surah_english,
            commands::surah::get_tafsir,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::playback::start_playback,
            commands::playback::stop_playback,
            commands::playback::pause_playback,
            commands::playback::resume_playback,
            commands::playback::get_cache_status,
            commands::playback::get_cache_usage,
            commands::playback::clear_cache,
            commands::playback::set_tts_enabled,
            commands::tafsir_voice::play_tafsir_voice,
            commands::tafsir_voice::is_tafsir_voice_active,
            commands::tafsir_voice::is_tafsir_voice_paused,
            commands::tafsir_voice::pause_tafsir_voice,
            commands::tafsir_voice::resume_tafsir_voice,
            commands::tts::get_tts_voice_status,
            commands::bookmark::get_bookmarks,
            commands::bookmark::add_bookmark,
            commands::bookmark::remove_bookmark,
            commands::search::search_quran,
            commands::search::get_index_status,
            commands::search::rebuild_index,
        ])
        .run(tauri::generate_context!())
        .expect("error while running eQuran desktop application");
}

#[cfg(test)]
mod tests {
    use super::{AppState, include_project_tts_root};

    #[test]
    fn app_state_initializes_api_client() {
        let state = AppState::new(None, None).expect("AppState should initialize ApiClient");

        let _api = state.api();
    }

    #[test]
    fn app_state_prioritizes_bundled_tts_over_app_data() {
        let resource_root = std::path::PathBuf::from("/tmp/equran-resource");
        let writable_root = std::path::PathBuf::from("/tmp/equran-writable");
        let state = AppState::new(Some(resource_root.clone()), Some(writable_root.clone()))
            .expect("AppState should initialize");

        assert_eq!(state.tts_roots().first(), Some(&resource_root));
        assert_eq!(state.tts_roots().last(), Some(&writable_root));
    }

    #[test]
    fn app_state_tracks_writable_tts_root() {
        let writable_root = std::path::PathBuf::from("/tmp/equran-writable");
        let state =
            AppState::new(None, Some(writable_root.clone())).expect("AppState should initialize");

        assert_eq!(state.writable_tts_root(), &writable_root);
    }

    #[test]
    fn project_tts_root_is_debug_only() {
        assert!(include_project_tts_root());
    }
}
