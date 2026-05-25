pub mod commands;

use std::path::PathBuf;

use commands::bookmark::BookmarkState;
use commands::playback::PlaybackState;
use commands::settings::SettingsState;
use equran_core::api::client::ApiClient;
use tauri::{Manager, path::BaseDirectory};

#[derive(Clone)]
pub struct AppState {
    api: ApiClient,
    tts_roots: Vec<PathBuf>,
    writable_tts_root: PathBuf,
}

impl AppState {
    pub fn new(resource_root: Option<PathBuf>, writable_tts_root: Option<PathBuf>) -> anyhow::Result<Self> {
        let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut tts_roots = Vec::new();
        let writable_tts_root = writable_tts_root.unwrap_or_else(|| project_root.clone());
        tts_roots.push(writable_tts_root.clone());
        if let Some(resource_root) = resource_root {
            tts_roots.push(resource_root);
        }
        tts_roots.push(project_root);

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            let resource_root = app.path().resolve(".", BaseDirectory::Resource).ok();
            let writable_tts_root = app.path().app_data_dir().ok();
            let state = AppState::new(resource_root, writable_tts_root).map_err(|error| error.to_string())?;
            app.manage(state);
            app.manage(PlaybackState::new());
            app.manage(BookmarkState::new());
            app.manage(SettingsState::new());
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
            commands::playback::get_cache_status,
            commands::playback::set_tts_enabled,
            commands::playback::play_tafsir_voice,
            commands::tts::get_natural_indonesian_voice_status,
            commands::tts::install_natural_indonesian_voice,
            commands::bookmark::get_bookmarks,
            commands::bookmark::add_bookmark,
            commands::bookmark::remove_bookmark,
        ])
        .run(tauri::generate_context!())
        .expect("error while running eQuran desktop application");
}

#[cfg(test)]
mod tests {
    use super::AppState;

    #[test]
    fn app_state_initializes_api_client() {
        let state = AppState::new(None, None).expect("AppState should initialize ApiClient");

        let _api = state.api();
    }

    #[test]
    fn app_state_prioritizes_resource_root_for_tts() {
        let resource_root = std::path::PathBuf::from("/tmp/equran-resource");
        let writable_root = std::path::PathBuf::from("/tmp/equran-writable");
        let state = AppState::new(Some(resource_root.clone()), Some(writable_root.clone())).expect("AppState should initialize");

        assert_eq!(state.tts_roots().first(), Some(&writable_root));
        assert_eq!(state.tts_roots().get(1), Some(&resource_root));
    }

    #[test]
    fn app_state_tracks_writable_tts_root() {
        let writable_root = std::path::PathBuf::from("/tmp/equran-writable");
        let state = AppState::new(None, Some(writable_root.clone())).expect("AppState should initialize");

        assert_eq!(state.writable_tts_root(), &writable_root);
    }
}
