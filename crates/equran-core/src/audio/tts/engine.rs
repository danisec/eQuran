//! Synthesis engine: cache-first rendering through the worker, with a CLI fallback.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use tokio::fs;
use tokio::process::Command;
use tokio::sync::Mutex;

use crate::audio::cache::{
    is_valid_pcm_wav, tafsir_tts_chunk_filename, translation_tts_chunk_filename, tts_filename,
};
use crate::domain::Lang;

use super::config::{SupertonicConfig, interpreter_version, status_from_config};
use super::service::SupertonicService;
use super::text::{sanitize_for_speech, split_text_for_tts};
use super::{CACHE_SUBDIR, TAFSIR_CHUNK_CHARS, TtsVoiceStatus, temporary_path};

/// Last non-empty line of engine output, which is where a Python traceback
/// states the real error.
///
/// Pasting the whole traceback into a UI banner buries the message ("found 2
/// unsupported characters") under absolute file paths.
fn last_error_line(stderr: &str) -> Option<String> {
    stderr
        .lines()
        .map(str::trim)
        .rfind(|line| !line.is_empty())
        .map(str::to_owned)
}

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
    /// "Install the supertonic package" is wrong advice when the interpreter
    /// itself is the problem: `onnxruntime` publishes no wheel past Python 3.14,
    /// so `pip install` fails no matter what. Naming the interpreter, its
    /// version, and whether the app fell back to the system Python is what
    /// actually tells the reader which fix applies.
    pub fn voice_diagnosis(&self) -> String {
        let config = SupertonicConfig::from_env_or_project_roots(&self.project_roots);
        if !config.script.is_file() {
            return "TTS engine files are missing. Reinstall eQuran.".to_owned();
        }

        let python = config.synthesis_python();
        let name = python.display();
        match interpreter_version(python) {
            // A fallback interpreter means no runtime was ever set up; the app
            // borrowed the system Python and found no `supertonic` there.
            Some(version) if !config.uses_project_venv => format!(
                "The speech runtime is not set up yet. eQuran fell back to the system interpreter {name} (Python {version}), which does not have supertonic. onnxruntime ships wheels for Python 3.12–3.14."
            ),
            Some(version) => format!(
                "The speech runtime at {name} (Python {version}) could not load supertonic. onnxruntime ships wheels for Python 3.12–3.14, so this runtime may need rebuilding."
            ),
            None => format!(
                "The Python interpreter {name} could not be run. Python 3.12–3.14 is required to build the speech runtime."
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
        lang: Lang,
        surah: u8,
        ayah: u16,
        chunk: usize,
    ) -> Result<PathBuf> {
        let filename = {
            let speakable = sanitize_for_speech(text);
            tafsir_tts_chunk_filename(lang.code(), surah, ayah, chunk, &speakable)
        };
        self.synthesize_chunk_cached(text, lang, filename).await
    }

    /// Synthesises one chunk of the ayah translation.
    ///
    /// English reads the tafsir as its translation, which can run to thousands
    /// of characters. Sending that in one request outlives the worker timeout
    /// and stalls every later request behind it, so the translation is spoken
    /// chunk by chunk exactly like the tafsir narration.
    pub async fn synthesize_translation_chunk_cached(
        &self,
        text: &str,
        lang: Lang,
        surah: u8,
        ayah: u16,
        chunk: usize,
    ) -> Result<PathBuf> {
        let filename = {
            let speakable = sanitize_for_speech(text);
            translation_tts_chunk_filename(lang.code(), surah, ayah, chunk, &speakable)
        };
        self.synthesize_chunk_cached(text, lang, filename).await
    }

    async fn synthesize_chunk_cached(
        &self,
        text: &str,
        lang: Lang,
        filename: String,
    ) -> Result<PathBuf> {
        // Clean before hashing so the cache key names exactly the text that is
        // spoken. Dropping a chunk shifts the indexes of the rest, and a stale
        // file must never be replayed for different words.
        let speakable = sanitize_for_speech(text);
        let target = self.cache_path(filename).await?;
        self.ensure_cached(&speakable, lang, &target).await
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

        let speakable = sanitize_for_speech(text);
        if speakable.is_empty() {
            bail!("TTS text contains nothing the narrator can read");
        }

        self.ensure_ready()?;
        self.synthesize_to(&speakable, lang, target).await?;
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
            match last_error_line(&String::from_utf8_lossy(&result.stderr)) {
                Some(line) => bail!("TTS engine failed: {line}"),
                None => bail!("TTS engine exited with {}", result.status),
            }
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

    /// Splits narration (translation or tafsir) into chunks the voice service
    /// can speak in one request.
    ///
    /// The text is cleaned first so a script-only paragraph cannot become a
    /// chunk that fails to synthesize and stops the narration.
    pub fn split_narration_text(text: &str) -> Vec<String> {
        split_text_for_tts(&sanitize_for_speech(text), TAFSIR_CHUNK_CHARS)
    }
}
