use equran_core::audio::{
    cache::AudioCache,
    tts::{TtsEngine, TtsVoiceStatus},
};
use serde::Serialize;
use tauri::State;

use crate::AppState;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsVoiceStatusPayload {
    pub status: String,
    pub ready: bool,
    pub message: String,
}

#[tauri::command]
pub async fn get_tts_voice_status(
    state: State<'_, AppState>,
) -> Result<TtsVoiceStatusPayload, String> {
    tts_voice_status(state.inner())
}

pub fn tts_voice_status(state: &AppState) -> Result<TtsVoiceStatusPayload, String> {
    let cache = AudioCache::new().map_err(|e| e.to_string())?;
    let tts = TtsEngine::with_project_roots(cache.root().clone(), state.tts_roots().to_vec());

    let status = tts.voice_status();
    // The generic hint misleads when the interpreter itself is the problem, so
    // the specific diagnosis wins whenever the runtime is the missing piece.
    let diagnosis = matches!(status, TtsVoiceStatus::MissingRuntime).then(|| tts.voice_diagnosis());
    Ok(payload_from_status(status, diagnosis))
}

fn payload_from_status(status: TtsVoiceStatus, diagnosis: Option<String>) -> TtsVoiceStatusPayload {
    match status {
        TtsVoiceStatus::Ready => TtsVoiceStatusPayload {
            status: "ready".to_owned(),
            ready: true,
            message: "Supertonic 3 voice is ready.".to_owned(),
        },
        TtsVoiceStatus::MissingRuntime => TtsVoiceStatusPayload {
            status: "missingRuntime".to_owned(),
            ready: false,
            message: diagnosis.unwrap_or_else(|| {
                "Install the supertonic Python package to enable TTS translation.".to_owned()
            }),
        },
        TtsVoiceStatus::MissingScript => TtsVoiceStatusPayload {
            status: "missingScript".to_owned(),
            ready: false,
            message: "TTS engine files are missing. Reinstall eQuran.".to_owned(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ready_payload_is_user_facing() {
        let payload = payload_from_status(TtsVoiceStatus::Ready, None);

        assert!(payload.ready);
        assert_eq!(payload.status, "ready");
        assert!(payload.message.contains("Supertonic"));
    }

    #[test]
    fn missing_runtime_payload_is_actionable() {
        let payload = payload_from_status(TtsVoiceStatus::MissingRuntime, None);

        assert!(!payload.ready);
        assert_eq!(payload.status, "missingRuntime");
        assert!(payload.message.contains("supertonic"));
    }

    #[test]
    fn missing_script_payload_points_to_reinstall() {
        let payload = payload_from_status(TtsVoiceStatus::MissingScript, None);

        assert!(!payload.ready);
        assert_eq!(payload.status, "missingScript");
        assert!(payload.message.contains("Reinstall"));
    }
}
