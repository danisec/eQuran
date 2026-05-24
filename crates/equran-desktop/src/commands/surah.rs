use equran_core::api::models::{EnglishSurah, Surah, SurahListItem, TafsirSurah};
use tauri::State;

use crate::AppState;

#[tauri::command]
pub async fn get_surah_list(state: State<'_, AppState>) -> Result<Vec<SurahListItem>, String> {
    state
        .api()
        .fetch_surah_list()
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn get_surah(number: u8, state: State<'_, AppState>) -> Result<Surah, String> {
    state
        .api()
        .fetch_surah(number)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn get_surah_english(
    number: u8,
    state: State<'_, AppState>,
) -> Result<EnglishSurah, String> {
    state
        .api()
        .fetch_surah_english(number)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn get_tafsir(number: u8, state: State<'_, AppState>) -> Result<TafsirSurah, String> {
    state
        .api()
        .fetch_tafsir(number)
        .await
        .map_err(|error| error.to_string())
}
