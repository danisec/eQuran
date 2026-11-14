use std::process::Stdio;

use equran_core::audio::{
    cache::AudioCache,
    tts::{RuntimeSetup, TtsEngine, TtsVoiceStatus, forget_supertonic_probes},
};
use serde::Serialize;
use tauri::{Emitter, State, Window};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Mutex;

use crate::AppState;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsVoiceStatusPayload {
    pub status: String,
    pub ready: bool,
    /// The specific cause when the voice is not ready (interpreter version,
    /// missing files). Empty when there is nothing more to say than the status.
    pub detail: String,
    /// True when a bundled setup script exists, so the app can offer to run it.
    pub can_setup: bool,
    /// A command a reader can run by hand if the in-app setup cannot proceed.
    pub setup_command: String,
}

/// Serializes setup runs so two clicks cannot build the same venv at once.
#[derive(Default)]
pub struct TtsSetupState {
    lock: Mutex<()>,
}

#[derive(Clone, Debug, Serialize)]
struct TtsSetupProgress<'a> {
    line: &'a str,
}

#[tauri::command]
pub async fn get_tts_voice_status(
    state: State<'_, AppState>,
) -> Result<TtsVoiceStatusPayload, String> {
    tts_voice_status(state.inner())
}

pub fn tts_voice_status(state: &AppState) -> Result<TtsVoiceStatusPayload, String> {
    let engine = engine_for(state)?;
    let status = engine.voice_status();
    // Skip the diagnosis (which spawns Python) when there is nothing to report.
    let detail = (!status.is_ready()).then(|| engine.voice_diagnosis());
    Ok(payload(state, status, detail))
}

/// Fails with an actionable message when the local voice runtime is unusable.
///
/// Qari recitation never depends on this: callers downgrade the session to
/// qari-only and surface the message instead of refusing to play.
pub(crate) fn ensure_tts_ready(state: &AppState) -> Result<(), String> {
    let engine = engine_for(state)?;
    match engine.voice_status() {
        TtsVoiceStatus::Ready => Ok(()),
        TtsVoiceStatus::MissingScript => Err(
            "TTS engine files are missing. Reinstall eQuran before enabling TTS Translation."
                .to_owned(),
        ),
        TtsVoiceStatus::MissingRuntime => {
            Err("TTS runtime is not ready. Set it up from Settings, then try again.".to_owned())
        }
    }
}

/// Runs the bundled setup script, streaming its output to the window.
///
/// The venv is written under the app's own data directory, so this needs no
/// elevator: the reader turns the voice on from Settings and nothing outside
/// their home is touched. On success the caller re-reads the status and finds
/// the freshly built interpreter.
#[tauri::command]
pub async fn setup_tts_runtime(
    window: Window,
    state: State<'_, AppState>,
    setup_state: State<'_, TtsSetupState>,
) -> Result<TtsVoiceStatusPayload, String> {
    let _guard = setup_state.lock.lock().await;

    let plan =
        RuntimeSetup::discover(state.tts_roots(), state.writable_tts_root()).ok_or_else(|| {
            "The setup script is missing from this build. Reinstall eQuran.".to_owned()
        })?;

    let mut command = plan.command();
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|error| format!("Could not start the setup script: {error}"))?;

    // pip and the wheel check print to both streams; forward each line so the
    // reader sees the download actually progressing instead of a frozen button.
    let mut tasks = Vec::new();
    if let Some(stdout) = child.stdout.take() {
        tasks.push(tokio::spawn(forward_lines(
            BufReader::new(stdout).lines(),
            window.clone(),
        )));
    }
    if let Some(stderr) = child.stderr.take() {
        tasks.push(tokio::spawn(forward_lines(
            BufReader::new(stderr).lines(),
            window.clone(),
        )));
    }

    let status = child
        .wait()
        .await
        .map_err(|error| format!("The setup script could not be run: {error}"))?;
    for task in tasks {
        let _ = task.await;
    }

    if !status.success() {
        return Err("The setup script stopped with an error. Its last lines are below.".to_owned());
    }

    // The status probe memoizes "no supertonic" for the session; the script may
    // have just installed it into an interpreter that was already probed.
    forget_supertonic_probes();
    let engine = engine_for(state.inner())?;
    let status = engine.voice_status();
    let detail = (!status.is_ready()).then(|| engine.voice_diagnosis());
    Ok(payload(state.inner(), status, detail))
}

/// Streams a child's output to the window line by line.
async fn forward_lines<R>(mut lines: tokio::io::Lines<BufReader<R>>, window: Window)
where
    R: tokio::io::AsyncRead + Unpin,
{
    while let Ok(Some(line)) = lines.next_line().await {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            continue;
        }
        let _ = window.emit("tts-setup-progress", TtsSetupProgress { line: trimmed });
    }
}

fn engine_for(state: &AppState) -> Result<TtsEngine, String> {
    let cache = AudioCache::new().map_err(|e| e.to_string())?;
    Ok(TtsEngine::with_project_roots(
        cache.root().clone(),
        state.tts_roots().to_vec(),
    ))
}

fn payload(
    state: &AppState,
    status: TtsVoiceStatus,
    detail: Option<String>,
) -> TtsVoiceStatusPayload {
    let setup = RuntimeSetup::discover(state.tts_roots(), state.writable_tts_root());
    let (status_name, ready) = match status {
        TtsVoiceStatus::Ready => ("ready", true),
        TtsVoiceStatus::MissingRuntime => ("missingRuntime", false),
        TtsVoiceStatus::MissingScript => ("missingScript", false),
    };
    let setup_command = setup.as_ref().map(manual_command).unwrap_or_default();
    TtsVoiceStatusPayload {
        status: status_name.to_owned(),
        ready,
        detail: detail.unwrap_or_default(),
        can_setup: setup.is_some(),
        setup_command,
    }
}

/// The copy-pasteable equivalent of the in-app action.
fn manual_command(plan: &RuntimeSetup) -> String {
    format!(
        "VENV_DIR={} bash {}",
        shell_quote(&plan.venv_dir().display().to_string()),
        shell_quote(&plan.script().display().to_string()),
    )
}

/// Quotes a path for the manual-command hint. Paths with spaces are common in
/// home directories, and an unquoted one silently splits into two arguments.
fn shell_quote(value: &str) -> String {
    if value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "-_./=:".contains(c))
    {
        value.to_owned()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn state(resource: &str, writable: &str) -> AppState {
        AppState::new(Some(PathBuf::from(resource)), Some(PathBuf::from(writable)))
            .expect("AppState should initialize")
    }

    #[test]
    fn ready_payload_carries_no_detail() {
        let payload = payload(
            &state("/tmp/equran-resource", "/tmp/equran-writable"),
            TtsVoiceStatus::Ready,
            None,
        );

        assert!(payload.ready);
        assert_eq!(payload.status, "ready");
        assert!(payload.detail.is_empty());
    }

    #[test]
    fn missing_runtime_keeps_the_actionable_detail() {
        let payload = payload(
            &state("/tmp/equran-resource", "/tmp/equran-writable"),
            TtsVoiceStatus::MissingRuntime,
            Some("Python 3.15 cannot load supertonic.".to_owned()),
        );

        assert!(!payload.ready);
        assert_eq!(payload.status, "missingRuntime");
        assert!(payload.detail.contains("supertonic"));
    }

    #[test]
    fn payload_offers_the_setup_action_when_a_script_is_bundled() {
        let base = tempfile::tempdir().expect("tempdir");
        let resource = base.path().join("resource");
        std::fs::create_dir_all(resource.join("tts")).expect("dirs");
        std::fs::write(resource.join("tts/setup.sh"), "").expect("script");
        let state = AppState::new(Some(resource), Some(base.path().join("app-data")))
            .expect("AppState should initialize");

        let payload = payload(&state, TtsVoiceStatus::MissingRuntime, None);

        assert!(payload.can_setup);
        assert!(payload.setup_command.contains("setup.sh"));
        assert!(payload.setup_command.starts_with("VENV_DIR="));
    }

    #[test]
    fn setup_command_redirects_the_venv_to_a_writable_path() {
        let plan = RuntimeSetup::new(
            PathBuf::from("/usr/lib/eQuran/tts/setup.sh"),
            PathBuf::from("/home/user/.local/share/id.equran.desktop/tts/.venv"),
        );

        assert_eq!(
            manual_command(&plan),
            "VENV_DIR=/home/user/.local/share/id.equran.desktop/tts/.venv bash /usr/lib/eQuran/tts/setup.sh"
        );
    }

    #[test]
    fn shell_quote_protects_paths_with_spaces() {
        assert_eq!(
            shell_quote("/home/user/My Files/.venv"),
            "'/home/user/My Files/.venv'"
        );
        assert_eq!(
            shell_quote("/usr/lib/eQuran/tts/setup.sh"),
            "/usr/lib/eQuran/tts/setup.sh"
        );
    }
}
