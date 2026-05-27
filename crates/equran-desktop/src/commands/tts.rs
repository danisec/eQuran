use std::{
    path::{Path, PathBuf},
    process::Stdio,
};

use equran_core::audio::{
    cache::AudioCache,
    tts::{NaturalIndonesianStatus, TtsEngine},
};
use serde::Serialize;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tauri::{Emitter, State, Window};
use tokio::{fs, process::Command};

use crate::AppState;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NaturalIndonesianVoiceStatusPayload {
    pub status: String,
    pub ready: bool,
    pub can_download: bool,
    pub message: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NaturalIndonesianInstallProgressPayload {
    pub step: String,
    pub message: String,
    pub percent: Option<u8>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NaturalIndonesianVoiceManifest {
    version: String,
    platform: String,
    url: Option<String>,
    #[serde(default)]
    parts: Vec<NaturalIndonesianVoiceManifestPart>,
    sha256: Option<String>,
    size: Option<u64>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NaturalIndonesianVoiceManifestPart {
    url: String,
    sha256: Option<String>,
    size: Option<u64>,
}

#[tauri::command]
pub async fn get_natural_indonesian_voice_status(
    state: State<'_, AppState>,
) -> Result<NaturalIndonesianVoiceStatusPayload, String> {
    natural_indonesian_voice_status(state.inner())
}

#[tauri::command]
pub async fn install_natural_indonesian_voice(
    window: Window,
    state: State<'_, AppState>,
) -> Result<(), String> {
    emit_install_progress(
        &window,
        "checking",
        "Checking Natural Indonesian Voice files.",
        Some(10),
    );

    let current_status = natural_indonesian_voice_status(state.inner())?;
    if current_status.ready {
        emit_install_progress(
            &window,
            "ready",
            "Natural Indonesian Voice is ready.",
            Some(100),
        );
        return Ok(());
    }

    emit_install_progress(
        &window,
        "preparingRuntime",
        "Preparing Natural Indonesian Voice runtime.",
        Some(35),
    );

    if let Some(manifest) = load_voice_manifest(state.inner()).await? {
        install_voice_pack_from_manifest(&window, state.inner(), &manifest).await?;

        emit_install_progress(
            &window,
            "verifyingVoice",
            "Verifying Natural Indonesian Voice files.",
            Some(90),
        );

        let refreshed_status = natural_indonesian_voice_status(state.inner())?;
        if refreshed_status.ready {
            emit_install_progress(
                &window,
                "ready",
                "Natural Indonesian Voice is ready.",
                Some(100),
            );
            return Ok(());
        }

        emit_install_progress(&window, "failed", &refreshed_status.message, None);
        return Err(refreshed_status.message);
    }

    let source_setup_script = find_setup_script(state.inner()).ok_or_else(|| {
        "Natural Indonesian Voice download manifest is not configured and setup files are missing. Reinstall eQuran or configure a voice-pack manifest before enabling TTS Translation.".to_owned()
    })?;

    let setup_dir = prepare_writable_setup_dir(state.inner(), &source_setup_script).await?;
    let setup_script = setup_dir.join("setup.sh");

    emit_install_progress(
        &window,
        "downloadingModel",
        "Downloading voice model and checking the runtime.",
        Some(60),
    );

    let output = Command::new("bash")
        .arg(&setup_script)
        .current_dir(setup_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|error| format!("Failed to start Natural Indonesian Voice setup: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        let message = if stderr.is_empty() {
            "Natural Indonesian Voice setup failed. Check your connection and try again.".to_owned()
        } else {
            format!("Natural Indonesian Voice setup failed: {stderr}")
        };
        emit_install_progress(&window, "failed", &message, None);
        return Err(message);
    }

    emit_install_progress(
        &window,
        "verifyingVoice",
        "Verifying Natural Indonesian Voice files.",
        Some(85),
    );

    let refreshed_status = natural_indonesian_voice_status(state.inner())?;
    if refreshed_status.ready {
        emit_install_progress(
            &window,
            "ready",
            "Natural Indonesian Voice is ready.",
            Some(100),
        );
        return Ok(());
    }

    emit_install_progress(
        &window,
        "failed",
        &refreshed_status.message,
        None,
    );

    Err(refreshed_status.message)
}

pub fn natural_indonesian_voice_status(
    state: &AppState,
) -> Result<NaturalIndonesianVoiceStatusPayload, String> {
    let cache = AudioCache::new().map_err(|e| e.to_string())?;
    let tts = TtsEngine::with_project_roots(cache.root().clone(), state.tts_roots().to_vec());
    let status = tts.natural_indonesian_status();

    Ok(payload_from_status(status))
}

fn payload_from_status(status: NaturalIndonesianStatus) -> NaturalIndonesianVoiceStatusPayload {
    match status {
        NaturalIndonesianStatus::Ready => NaturalIndonesianVoiceStatusPayload {
            status: "ready".to_owned(),
            ready: true,
            can_download: false,
            message: "Natural Indonesian Voice is ready.".to_owned(),
        },
        NaturalIndonesianStatus::MissingRuntime => NaturalIndonesianVoiceStatusPayload {
            status: "missingRuntime".to_owned(),
            ready: false,
            can_download: true,
            message: "Download and prepare its voice runtime before TTS Translation can be enabled.".to_owned(),
        },
        NaturalIndonesianStatus::MissingScript => NaturalIndonesianVoiceStatusPayload {
            status: "missingScript".to_owned(),
            ready: false,
            can_download: true,
            message: "Natural Indonesian Voice setup files are missing. Reinstall eQuran or use the source setup script before enabling TTS Translation.".to_owned(),
        },
    }
}

fn find_setup_script(state: &AppState) -> Option<PathBuf> {
    state
        .tts_roots()
        .iter()
        .map(|root| root.join("tts/setup.sh"))
        .find(|path| path.is_file())
}

async fn load_voice_manifest(
    state: &AppState,
) -> Result<Option<NaturalIndonesianVoiceManifest>, String> {
    if let Ok(manifest_url) = std::env::var("EQURAN_NATURAL_INDONESIAN_MANIFEST_URL") {
        if !manifest_url.trim().is_empty() {
            let manifest = reqwest::get(manifest_url.trim())
                .await
                .map_err(|error| format!("Failed to download Natural Indonesian Voice manifest: {error}"))?
                .error_for_status()
                .map_err(|error| format!("Natural Indonesian Voice manifest request failed: {error}"))?
                .json::<NaturalIndonesianVoiceManifest>()
                .await
                .map_err(|error| format!("Failed to read Natural Indonesian Voice manifest: {error}"))?;
            return Ok(Some(manifest));
        }
    }

    for root in state.tts_roots() {
        let manifest_path = root.join("tts/voice-pack-manifest.json");
        if manifest_path.is_file() {
            let manifest_text = fs::read_to_string(&manifest_path).await.map_err(|error| {
                format!("Failed to read Natural Indonesian Voice manifest: {error}")
            })?;
            let manifest = serde_json::from_str::<NaturalIndonesianVoiceManifest>(&manifest_text)
                .map_err(|error| format!("Failed to parse Natural Indonesian Voice manifest: {error}"))?;
            return Ok(Some(manifest));
        }
    }

    Ok(None)
}

async fn install_voice_pack_from_manifest(
    window: &Window,
    state: &AppState,
    manifest: &NaturalIndonesianVoiceManifest,
) -> Result<(), String> {
    validate_voice_manifest(manifest)?;

    emit_install_progress(
        window,
        "downloadingModel",
        &format!("Downloading Voice {}", manifest.version),
        Some(40),
    );

    let archive_bytes = download_voice_archive(window, manifest).await?;

    if let Some(expected_size) = manifest.size {
        if archive_bytes.len() as u64 != expected_size {
            return Err(format!(
                "Natural Indonesian Voice download size mismatch. Expected {expected_size} bytes, got {} bytes.",
                archive_bytes.len()
            ));
        }
    }

    if let Some(expected_sha256) = &manifest.sha256 {
        verify_sha256(&archive_bytes, expected_sha256)?;
    }

    emit_install_progress(
        window,
        "verifyingVoice",
        "Extracting Natural Indonesian Voice files.",
        Some(75),
    );

    fs::create_dir_all(state.writable_tts_root())
        .await
        .map_err(|error| format!("Failed to create Natural Indonesian Voice directory: {error}"))?;
    let archive_path = state
        .writable_tts_root()
        .join(format!("natural-indonesian-voice-{}.tar.zst", manifest.version));
    fs::write(&archive_path, &archive_bytes)
        .await
        .map_err(|error| format!("Failed to save Natural Indonesian Voice archive: {error}"))?;

    extract_tar_zst(&archive_path, state.writable_tts_root()).await
}

fn validate_voice_manifest(manifest: &NaturalIndonesianVoiceManifest) -> Result<(), String> {
    let expected_platform = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);
    if manifest.platform != expected_platform {
        return Err(format!(
            "Natural Indonesian Voice package is for {}, but this device needs {}.",
            manifest.platform, expected_platform
        ));
    }

    if manifest
        .url
        .as_deref()
        .is_some_and(|url| !url.trim().is_empty())
    {
        return Ok(());
    }

    if manifest.parts.is_empty() {
        return Err("Natural Indonesian Voice manifest must include a URL or release asset parts.".to_owned());
    }

    if manifest.parts.iter().any(|part| part.url.trim().is_empty()) {
        return Err("Natural Indonesian Voice manifest contains an empty part URL.".to_owned());
    }

    Ok(())
}

async fn download_voice_archive(
    window: &Window,
    manifest: &NaturalIndonesianVoiceManifest,
) -> Result<Vec<u8>, String> {
    if let Some(url) = manifest.url.as_deref().filter(|url| !url.trim().is_empty()) {
        let mut downloaded = 0;
        return download_bytes_with_progress(
            url,
            manifest.size,
            &mut downloaded,
            manifest.size,
            window,
            &manifest.version,
        )
        .await;
    }

    let mut archive = Vec::new();
    let total_size = manifest
        .size
        .or_else(|| manifest.parts.iter().map(|part| part.size).sum());
    let mut downloaded_total = 0;
    for (index, part) in manifest.parts.iter().enumerate() {
        let part_bytes = download_bytes_with_progress(
            &part.url,
            part.size,
            &mut downloaded_total,
            total_size,
            window,
            &manifest.version,
        )
        .await
        .map_err(|error| {
            format!("Failed to download Natural Indonesian Voice part {}: {error}", index + 1)
        })?;
        if let Some(expected_size) = part.size {
            if part_bytes.len() as u64 != expected_size {
                return Err(format!(
                    "Natural Indonesian Voice part {} size mismatch. Expected {expected_size} bytes, got {} bytes.",
                    index + 1,
                    part_bytes.len()
                ));
            }
        }
        if let Some(expected_sha256) = &part.sha256 {
            verify_sha256(&part_bytes, expected_sha256)?;
        }
        archive.extend_from_slice(&part_bytes);
    }

    Ok(archive)
}

async fn download_bytes_with_progress(
    url: &str,
    expected_size: Option<u64>,
    downloaded_total: &mut u64,
    total_size: Option<u64>,
    window: &Window,
    version: &str,
) -> Result<Vec<u8>, String> {
    let response = reqwest::get(url)
        .await
        .map_err(|error| format!("Failed to download Natural Indonesian Voice: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Natural Indonesian Voice download failed: {error}"))?;
    let mut response = response;
    let mut bytes = Vec::with_capacity(expected_size.unwrap_or_default() as usize);

    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| format!("Failed to read Natural Indonesian Voice download: {error}"))?
    {
        *downloaded_total += chunk.len() as u64;
        bytes.extend_from_slice(&chunk);
        if let Some(total_size) = total_size.filter(|size| *size > 0) {
            let raw_percent = ((*downloaded_total as f64 / total_size as f64) * 100.0).round() as u8;
            let percent = raw_percent.min(99);
            emit_install_progress(
                window,
                "downloadingModel",
                &format!("Downloading Voice {version}"),
                Some(percent),
            );
        }
    }

    Ok(bytes)
}

fn verify_sha256(bytes: &[u8], expected_sha256: &str) -> Result<(), String> {
    let actual = format!("{:x}", Sha256::digest(bytes));
    if actual.eq_ignore_ascii_case(expected_sha256.trim()) {
        return Ok(());
    }

    Err("Natural Indonesian Voice checksum verification failed.".to_owned())
}

async fn extract_tar_zst(archive_path: &Path, destination: &Path) -> Result<(), String> {
    let output = Command::new("tar")
        .arg("--zstd")
        .arg("-xf")
        .arg(archive_path)
        .arg("-C")
        .arg(destination)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|error| format!("Failed to extract Natural Indonesian Voice archive: {error}"))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    if stderr.is_empty() {
        Err("Natural Indonesian Voice archive extraction failed.".to_owned())
    } else {
        Err(format!("Natural Indonesian Voice archive extraction failed: {stderr}"))
    }
}

async fn prepare_writable_setup_dir(
    state: &AppState,
    source_setup_script: &PathBuf,
) -> Result<PathBuf, String> {
    let source_dir = source_setup_script
        .parent()
        .ok_or_else(|| "Natural Indonesian Voice setup path is invalid.".to_owned())?;
    let target_dir = state.writable_tts_root().join("tts");
    fs::create_dir_all(&target_dir)
        .await
        .map_err(|error| format!("Failed to create Natural Indonesian Voice directory: {error}"))?;

    for filename in ["setup.sh", "requirements.txt", "tts_wibowo.py"] {
        let source = source_dir.join(filename);
        let target = target_dir.join(filename);
        if source.is_file() {
            fs::copy(&source, &target).await.map_err(|error| {
                format!("Failed to prepare Natural Indonesian Voice setup file {filename}: {error}")
            })?;
        }
    }

    Ok(target_dir)
}

fn emit_install_progress(
    window: &Window,
    step: &str,
    message: &str,
    percent: Option<u8>,
) {
    let payload = NaturalIndonesianInstallProgressPayload {
        step: step.to_owned(),
        message: message.to_owned(),
        percent,
    };
    let _ = window.emit("natural-indonesian-install-progress", payload);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ready_status_payload_uses_user_facing_copy() {
        let payload = payload_from_status(NaturalIndonesianStatus::Ready);

        assert!(payload.ready);
        assert_eq!(payload.status, "ready");
        assert!(payload.message.contains("Natural Indonesian Voice"));
        assert!(!payload.message.contains("Wibowo"));
    }

    #[test]
    fn missing_runtime_payload_allows_download_without_internal_name() {
        let payload = payload_from_status(NaturalIndonesianStatus::MissingRuntime);

        assert!(!payload.ready);
        assert!(payload.can_download);
        assert_eq!(payload.status, "missingRuntime");
        assert!(!payload.message.contains("Wibowo"));
    }

    #[test]
    fn voice_manifest_accepts_current_platform() {
        let manifest = NaturalIndonesianVoiceManifest {
            version: "1.0.0".to_owned(),
            platform: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
            url: Some("https://example.com/voice.tar.zst".to_owned()),
            parts: Vec::new(),
            sha256: None,
            size: None,
        };

        assert!(validate_voice_manifest(&manifest).is_ok());
    }

    #[test]
    fn voice_manifest_rejects_wrong_platform() {
        let manifest = NaturalIndonesianVoiceManifest {
            version: "1.0.0".to_owned(),
            platform: "windows-x86_64".to_owned(),
            url: Some("https://example.com/voice.tar.zst".to_owned()),
            parts: Vec::new(),
            sha256: None,
            size: None,
        };

        let error = validate_voice_manifest(&manifest).expect_err("wrong platform should fail");
        assert!(error.contains("this device needs"));
    }

    #[test]
    fn verifies_matching_sha256() {
        assert!(verify_sha256(b"equran", "f8f7ca4a850aa1ffaf1c0fb1965a7a3c21f08ba707745f182aab353ed2c624cd").is_ok());
    }

    #[test]
    fn rejects_mismatched_sha256() {
        let error = verify_sha256(b"equran", "0000000000000000000000000000000000000000000000000000000000000000")
            .expect_err("wrong checksum should fail");

        assert!(error.contains("checksum"));
    }

    #[test]
    fn voice_manifest_accepts_release_asset_parts() {
        let manifest = NaturalIndonesianVoiceManifest {
            version: "1.0.0".to_owned(),
            platform: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
            url: None,
            parts: vec![NaturalIndonesianVoiceManifestPart {
                url: "https://example.com/voice.tar.zst.part-aa".to_owned(),
                sha256: None,
                size: None,
            }],
            sha256: None,
            size: None,
        };

        assert!(validate_voice_manifest(&manifest).is_ok());
    }
}
