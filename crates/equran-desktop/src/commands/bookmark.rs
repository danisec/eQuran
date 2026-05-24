use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::Mutex;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bookmark {
    pub surah: u8,
    pub ayah: u16,
    pub surah_name: String,
    pub ayah_text: String,
}

pub struct BookmarkState {
    bookmarks: Mutex<Vec<Bookmark>>,
    file_path: PathBuf,
}

impl BookmarkState {
    pub fn new() -> Self {
        let file_path = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("equran-cli")
            .join("bookmarks.json");

        let bookmarks = if file_path.exists() {
            std::fs::read_to_string(&file_path)
                .ok()
                .and_then(|content| serde_json::from_str(&content).ok())
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        Self {
            bookmarks: Mutex::new(bookmarks),
            file_path,
        }
    }

    async fn persist(&self) -> Result<(), String> {
        let bookmarks = self.bookmarks.lock().await;
        if let Some(parent) = self.file_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(&*bookmarks).map_err(|e| e.to_string())?;
        tokio::fs::write(&self.file_path, json)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[tauri::command]
pub async fn get_bookmarks(state: State<'_, BookmarkState>) -> Result<Vec<Bookmark>, String> {
    let bookmarks = state.bookmarks.lock().await;
    Ok(bookmarks.clone())
}

#[tauri::command]
pub async fn add_bookmark(
    state: State<'_, BookmarkState>,
    surah: u8,
    ayah: u16,
    surah_name: String,
    ayah_text: String,
) -> Result<Vec<Bookmark>, String> {
    let mut bookmarks = state.bookmarks.lock().await;
    let already_exists = bookmarks.iter().any(|b| b.surah == surah && b.ayah == ayah);
    if !already_exists {
        bookmarks.push(Bookmark {
            surah,
            ayah,
            surah_name,
            ayah_text,
        });
    }
    drop(bookmarks);
    state.persist().await?;
    let bookmarks = state.bookmarks.lock().await;
    Ok(bookmarks.clone())
}

#[tauri::command]
pub async fn remove_bookmark(
    state: State<'_, BookmarkState>,
    surah: u8,
    ayah: u16,
) -> Result<Vec<Bookmark>, String> {
    let mut bookmarks = state.bookmarks.lock().await;
    bookmarks.retain(|b| !(b.surah == surah && b.ayah == ayah));
    drop(bookmarks);
    state.persist().await?;
    let bookmarks = state.bookmarks.lock().await;
    Ok(bookmarks.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn add_and_remove_bookmark() {
        let state = BookmarkState {
            bookmarks: Mutex::new(Vec::new()),
            file_path: PathBuf::from("/tmp/equran-test-bookmarks.json"),
        };

        let mut bookmarks = state.bookmarks.lock().await;
        bookmarks.push(Bookmark {
            surah: 2,
            ayah: 255,
            surah_name: "Al-Baqarah".to_owned(),
            ayah_text: "Ayatul Kursi".to_owned(),
        });
        assert_eq!(bookmarks.len(), 1);

        bookmarks.retain(|b| !(b.surah == 2 && b.ayah == 255));
        assert_eq!(bookmarks.len(), 0);
    }
}
