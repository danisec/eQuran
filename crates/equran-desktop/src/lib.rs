pub mod commands;

use commands::bookmark::BookmarkState;
use commands::playback::PlaybackState;
use commands::settings::SettingsState;
use equran_core::api::client::ApiClient;
use tauri::Manager;

#[derive(Clone)]
pub struct AppState {
    api: ApiClient,
}

impl AppState {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            api: ApiClient::new()?,
        })
    }

    pub fn api(&self) -> &ApiClient {
        &self.api
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            let state = AppState::new().map_err(|error| error.to_string())?;
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
        let state = AppState::new().expect("AppState should initialize ApiClient");

        let _api = state.api();
    }
}
