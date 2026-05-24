use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopSettings {
    pub qari: String,
    pub lang: String,
    pub prefetch_tts: bool,
    pub volume: u8,
    pub last_surah: u8,
}

impl Default for DesktopSettings {
    fn default() -> Self {
        Self {
            qari: "misyari".to_owned(),
            lang: "id".to_owned(),
            prefetch_tts: true,
            volume: 70,
            last_surah: 1,
        }
    }
}

pub struct SettingsState {
    settings: Mutex<DesktopSettings>,
    file_path: PathBuf,
}

impl SettingsState {
    pub fn new() -> Self {
        let file_path = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("equran-cli")
            .join("settings.json");

        let settings = if file_path.exists() {
            std::fs::read_to_string(&file_path)
                .ok()
                .and_then(|content| serde_json::from_str(&content).ok())
                .unwrap_or_default()
        } else {
            DesktopSettings::default()
        };

        Self {
            settings: Mutex::new(settings),
            file_path,
        }
    }

    async fn persist(&self) -> Result<(), String> {
        let settings = self.settings.lock().await;
        if let Some(parent) = self.file_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(&*settings).map_err(|e| e.to_string())?;
        tokio::fs::write(&self.file_path, json)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[tauri::command]
pub async fn get_settings(
    state: tauri::State<'_, SettingsState>,
) -> Result<DesktopSettings, String> {
    let settings = state.settings.lock().await;
    Ok(settings.clone())
}

#[tauri::command]
pub async fn save_settings(
    state: tauri::State<'_, SettingsState>,
    settings: DesktopSettings,
) -> Result<DesktopSettings, String> {
    let mut current = state.settings.lock().await;
    *current = settings.clone();
    drop(current);
    state.persist().await?;
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::DesktopSettings;

    #[test]
    fn default_settings_match_expected() {
        assert_eq!(
            DesktopSettings::default(),
            DesktopSettings {
                qari: "misyari".to_owned(),
                lang: "id".to_owned(),
                prefetch_tts: true,
                volume: 70,
                last_surah: 1,
            }
        );
    }
}
