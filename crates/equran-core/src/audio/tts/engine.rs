//! Synthesis engine: cache-first rendering through the worker, with a CLI fallback.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use tokio::fs;
use tokio::process::Command;
use tokio::sync::Mutex;

use crate::audio::cache::{is_valid_pcm_wav, tafsir_tts_chunk_filename, tts_filename};
use crate::domain::Lang;

use super::config::{SupertonicConfig, interpreter_version, status_from_config};
use super::service::SupertonicService;
use super::text::split_text_for_tts;
use super::{CACHE_SUBDIR, TAFSIR_CHUNK_CHARS, TtsVoiceStatus, temporary_path};

#[derive(Clone)]
pub struct TtsEngine {
    cache_root: PathBuf,
    project_roots: Vec<PathBuf>,
    service: Arc<Mutex<Option<Arc<SupertonicService>>>>,
}

impl TtsEngine {
    pub fn new(cache_root: PathBuf) -> Self {
        let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::with_project_root(cache_root, project_root)
    }

    pub fn with_project_root(cache_root: PathBuf, project_root: PathBuf) -> Self {
        Self::with_project_roots(cache_root, vec![project_root])
    }

    pub fn with_project_roots(cache_root: PathBuf, project_roots: Vec<PathBuf>) -> Self {
        Self {
            cache_root,
            project_roots,
            service: Arc::new(Mutex::new(None)),
        }
    }

    /// Backend identifier used in cache filenames and status messages.
    pub fn backend_name(&self) -> &'static str {
        "supertonic"
    }

    pub fn voice_status(&self) -> TtsVoiceStatus {
        status_from_config(&SupertonicConfig::from_env_or_project_roots(
            &self.project_roots,
        ))
    }

    /// Diagnoses why the voice is not ready, in words the reader can act on.
    ///
    /// "Install the supertonic package" is wrong advice when the package is
    /// present but the interpreter is too new: `onnxruntime` publishes no wheel
    /// for some Python versions, so `pip install` fails no matter what. Naming
    /// the interpreter and the real cause is what actually helps.
    pub fn voice_diagnosis(&self) -> String {
        let config = SupertonicConfig::from_env_or_project_roots(&self.project_roots);
        if !config.script.is_file() {
            return "TTS engine files are missing. Reinstall eQuran.".to_owned();
        }

        let python = config.synthesis_python();
        match interpreter_version(python) {
            Some(version) => format!(
                "Python {version} ({}) cannot load supertonic. onnxruntime publishes wheels only up to Python 3.14, so use Python 3.12 or 3.14.",
                python.display()
            ),
            None => format!(
                "The Python interpreter {} could not be run. Install Python 3.12 or 3.14 and the supertonic package.",
                python.display()
            ),
        }
    }

    pub fn ensure_ready(&self) -> Result<()> {
        match self.voice_status() {
            TtsVoiceStatus::Ready => Ok(()),
            TtsVoiceStatus::MissingScript => bail!(
                "TTS engine files are missing. Reinstall eQuran or set EQURAN_TTS_SUPERTONIC to tts_supertonic.py."
            ),
            TtsVoiceStatus::MissingRuntime => bail!(
                "Python runtime with the supertonic package is required. Install it with: pip install supertonic"
            ),
        }
    }

    pub async fn synthesize_cached(
        &self,
        text: &str,
        lang: Lang,
        surah: u8,
        ayah: u16,
    ) -> Result<PathBuf> {
        let target = self
            .cache_path(tts_filename(lang.code(), surah, ayah))
            .await?;
        self.ensure_cached(text, lang, &target).await
    }

    pub async fn synthesize_tafsir_chunk_cached(
        &self,
        text: &str,
        surah: u8,
        ayah: u16,
        chunk: usize,
    ) -> Result<PathBuf> {
        let target = self
            .cache_path(tafsir_tts_chunk_filename(
                Lang::Id.code(),
                surah,
                ayah,
                chunk,
            ))
            .await?;
        self.ensure_cached(text, Lang::Id, &target).await
    }

    async fn cache_path(&self, filename: String) -> Result<PathBuf> {
        let tts_dir = self.cache_root.join(CACHE_SUBDIR);
        fs::create_dir_all(&tts_dir)
            .await
            .with_context(|| format!("failed to create {}", tts_dir.display()))?;
        Ok(tts_dir.join(filename))
    }

    async fn ensure_cached(&self, text: &str, lang: Lang, target: &Path) -> Result<PathBuf> {
        if is_valid_pcm_wav(target).await {
            return Ok(target.to_path_buf());
        }

        self.ensure_ready()?;
        self.synthesize_to(text, lang, target).await?;
        Ok(target.to_path_buf())
    }

    /// Renders speech into a temporary file and only then publishes it as
    /// `target`. A crash or timeout therefore never leaves a partially written
    /// file that would later be treated as a valid cache hit.
    async fn synthesize_to(&self, text: &str, lang: Lang, target: &Path) -> Result<()> {
        let temp = temporary_path(target);
        let result = self.run_synthesis(text, lang.code(), &temp).await;

        if let Err(error) = result {
            let _ = fs::remove_file(&temp).await;
            return Err(error);
        }

        if !is_valid_pcm_wav(&temp).await {
            let _ = fs::remove_file(&temp).await;
            bail!(
                "TTS produced an invalid audio file for {}",
                target.display()
            );
        }

        fs::rename(&temp, target).await.with_context(|| {
            format!(
                "failed to publish generated audio {} to {}",
                temp.display(),
                target.display()
            )
        })?;

        Ok(())
    }

    async fn run_synthesis(&self, text: &str, lang: &str, output: &Path) -> Result<()> {
        let service_error = match self.synthesize_with_service(text, lang, output).await {
            Ok(()) => return Ok(()),
            Err(error) => error,
        };

        match self.synthesize_with_cli(text, lang, output).await {
            Ok(()) => Ok(()),
            Err(cli_error) => {
                bail!("TTS synthesis failed (service: {service_error}; cli: {cli_error})")
            }
        }
    }

    pub(super) async fn synthesize_with_service(
        &self,
        text: &str,
        lang: &str,
        output: &Path,
    ) -> Result<()> {
        let service = self.service().await?;
        service.synthesize(text, output, lang).await
    }

    async fn synthesize_with_cli(&self, text: &str, lang: &str, output: &Path) -> Result<()> {
        let config = SupertonicConfig::from_env_or_project_roots(&self.project_roots);
        if !config.is_usable() {
            bail!("TTS engine is not configured");
        }

        let python = config.synthesis_python();
        let result = Command::new(python)
            .arg(&config.script)
            .arg("--text")
            .arg(text)
            .arg("--output")
            .arg(output)
            .arg("--lang")
            .arg(lang)
            .arg("--voice")
            .arg(&config.voice)
            .arg("--steps")
            .arg(config.total_steps.to_string())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .await
            .with_context(|| format!("failed to start TTS engine via {}", python.display()))?;

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            let stderr = stderr.trim();
            if stderr.is_empty() {
                bail!("TTS engine exited with {}", result.status);
            }
            bail!("TTS engine exited with {}: {stderr}", result.status);
        }

        Ok(())
    }

    async fn service(&self) -> Result<Arc<SupertonicService>> {
        let mut slot = self.service.lock().await;
        if let Some(service) = slot.as_ref() {
            return Ok(Arc::clone(service));
        }

        let config = SupertonicConfig::from_env_or_project_roots(&self.project_roots);
        if !config.is_usable() {
            bail!("TTS engine is not configured");
        }

        let created = Arc::new(SupertonicService::new(config));
        *slot = Some(Arc::clone(&created));
        Ok(created)
    }

    /// Splits a tafsir into chunks the voice service can speak in one request.
    pub fn split_tafsir_text(text: &str) -> Vec<String> {
        split_text_for_tts(text, TAFSIR_CHUNK_CHARS)
    }
}
