use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Stdio,
    sync::{
        Arc, Mutex as StdMutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use tokio::{
    fs,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    process::{Child, ChildStdin, ChildStdout, Command},
    sync::Mutex,
    time::{Duration, timeout},
};

use crate::{
    audio::cache::{is_valid_pcm_wav, tafsir_tts_chunk_filename, tts_filename},
    domain::Lang,
};

const DEFAULT_VOICE: &str = "M2";
const CACHE_SUBDIR: &str = "tts";
const SERVICE_REQUEST_TIMEOUT: Duration = Duration::from_secs(300);
/// Characters per synthesis request.
///
/// Supertonic's diffusion loop costs roughly 0.12 s per character per step on
/// CPU, so a long request blocks the single service process for minutes. Short
/// requests keep the first chunk audible quickly and let the prefetch queue
/// overlap synthesis with playback.
const TAFSIR_CHUNK_CHARS: usize = 320;
/// Diffusion steps handed to Supertonic.
///
/// The library default is 8. Measured on a Ryzen 7 5700U, 5 steps cut synthesis
/// time by ~43% with no audible quality loss for narration, and 4 steps only
/// saves another ~25% while starting to sound flat. Override with
/// `EQURAN_TTS_SUPERTONIC_STEPS`.
const DEFAULT_TOTAL_STEPS: u32 = 5;
/// Device preference handed to the Python runtime.
///
/// `auto` uses a GPU execution provider when one is installed and otherwise
/// falls back to CPU; `cpu` keeps the GPU free for games; `gpu` is the same as
/// `auto` but makes a missing GPU explicit in the logs. Override with
/// `EQURAN_TTS_DEVICE`.
const DEFAULT_DEVICE: &str = "auto";
/// Seconds the Python service may sit idle before it exits.
///
/// The loaded model holds ~1 GB of RSS and a thread pool; releasing it while
/// nobody is listening keeps the desktop responsive. The next request respawns
/// the service and pays the model load again. Override with
/// `EQURAN_TTS_IDLE_TIMEOUT` (0 keeps it alive forever).
const DEFAULT_IDLE_TIMEOUT_SECS: u32 = 600;

/// Readiness of the local Supertonic runtime, surfaced to the UI so the
/// translation toggle can be disabled with an actionable message.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TtsVoiceStatus {
    Ready,
    MissingRuntime,
    MissingScript,
}

impl TtsVoiceStatus {
    pub fn is_ready(self) -> bool {
        matches!(self, Self::Ready)
    }
}

struct SupertonicConfig {
    python: PathBuf,
    gpu_python: Option<PathBuf>,
    script: PathBuf,
    voice: String,
    total_steps: u32,
    device: String,
    idle_timeout_secs: u32,
}

struct SupertonicService {
    config: SupertonicConfig,
    process: Mutex<Option<SupertonicServiceProcess>>,
}

struct SupertonicServiceProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: Lines<BufReader<ChildStdout>>,
}

#[derive(Debug, Serialize)]
struct SupertonicServiceRequest<'a> {
    text: &'a str,
    output: &'a str,
    lang: &'a str,
    voice: &'a str,
    steps: u32,
}

#[derive(Debug, Deserialize)]
struct SupertonicServiceResponse {
    ok: bool,
    error: Option<String>,
}

impl SupertonicConfig {
    fn from_env_or_project_roots(project_roots: &[PathBuf]) -> Self {
        let script = std::env::var_os("EQURAN_TTS_SUPERTONIC")
            .map(PathBuf::from)
            .unwrap_or_else(|| first_project_file(project_roots, "tts/tts_supertonic.py"));

        let python = std::env::var_os("EQURAN_TTS_PYTHON")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let venv_python = first_project_file(project_roots, "tts/.venv/bin/python");
                if venv_python.is_file() {
                    venv_python
                } else {
                    PathBuf::from("python3")
                }
            });

        // A GPU-capable ONNX Runtime wheel can be installed into a side venv
        // without touching the system interpreter. Prefer it when present.
        let gpu_python = first_project_file(project_roots, "tts/.venv-gpu/bin/python");
        let gpu_python = gpu_python.is_file().then_some(gpu_python);

        let voice = std::env::var("EQURAN_TTS_SUPERTONIC_VOICE")
            .ok()
            .filter(|voice| !voice.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_VOICE.to_owned());

        let total_steps = std::env::var("EQURAN_TTS_SUPERTONIC_STEPS")
            .ok()
            .and_then(|steps| steps.trim().parse::<u32>().ok())
            .filter(|steps| (1..=100).contains(steps))
            .unwrap_or(DEFAULT_TOTAL_STEPS);

        let device = std::env::var("EQURAN_TTS_DEVICE")
            .ok()
            .map(|device| device.trim().to_ascii_lowercase())
            .filter(|device| matches!(device.as_str(), "auto" | "cpu" | "gpu"))
            .unwrap_or_else(|| DEFAULT_DEVICE.to_owned());

        let idle_timeout_secs = std::env::var("EQURAN_TTS_IDLE_TIMEOUT")
            .ok()
            .and_then(|secs| secs.trim().parse::<u32>().ok())
            .unwrap_or(DEFAULT_IDLE_TIMEOUT_SECS);

        Self {
            python,
            gpu_python,
            script,
            voice,
            total_steps,
            device,
            idle_timeout_secs,
        }
    }

    /// Interpreter used for synthesis.
    ///
    /// The GPU venv is only chosen when it actually has the package, so a
    /// half-built venv degrades to the CPU interpreter instead of disabling TTS.
    fn synthesis_python(&self) -> &Path {
        match &self.gpu_python {
            Some(python) if supertonic_module_available(python) => python,
            _ => &self.python,
        }
    }

    fn python_is_usable(&self) -> bool {
        supertonic_module_available(self.synthesis_python())
    }

    fn is_usable(&self) -> bool {
        self.script.is_file() && self.python_is_usable()
    }
}

fn status_from_config(config: &SupertonicConfig) -> TtsVoiceStatus {
    if !config.script.is_file() {
        return TtsVoiceStatus::MissingScript;
    }
    if !config.python_is_usable() {
        return TtsVoiceStatus::MissingRuntime;
    }
    TtsVoiceStatus::Ready
}

/// Resolves the first `relative_path` that exists under any project root.
fn first_project_file(project_roots: &[PathBuf], relative_path: &str) -> PathBuf {
    project_roots
        .iter()
        .map(|root| root.join(relative_path))
        .find(|path| path.is_file())
        .unwrap_or_else(|| {
            project_roots
                .first()
                .map(|root| root.join(relative_path))
                .unwrap_or_else(|| PathBuf::from(relative_path))
        })
}

/// Reads the interpreter's version as `major.minor.patch`, or `None` when it
/// cannot be executed at all.
fn interpreter_version(python: &Path) -> Option<String> {
    let output = std::process::Command::new(python)
        .arg("-c")
        .arg("import sys; print('%d.%d.%d' % sys.version_info[:3])")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let version = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    (!version.is_empty()).then_some(version)
}

/// Probes `import supertonic` once per interpreter path.
///
/// Spawning a Python process costs hundreds of milliseconds, and readiness is
/// queried on every playback command, so results are memoized for the lifetime
/// of the process. A bare interpreter name such as `python3` is resolved
/// through `PATH` first, so the cache key is the real executable.
fn supertonic_module_available(python: &Path) -> bool {
    static CACHE: OnceLock<StdMutex<HashMap<PathBuf, bool>>> = OnceLock::new();

    let key = if python.components().count() == 1 {
        which_in_path(python)
    } else {
        None
    }
    .unwrap_or_else(|| python.to_path_buf());

    let cache = CACHE.get_or_init(|| StdMutex::new(HashMap::new()));
    if let Ok(guard) = cache.lock()
        && let Some(cached) = guard.get(&key)
    {
        return *cached;
    }

    let available = std::process::Command::new(&key)
        .arg("-c")
        .arg("import supertonic")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false);

    if let Ok(mut guard) = cache.lock() {
        guard.insert(key, available);
    }

    available
}

/// Resolves a bare executable name through `PATH`.
fn which_in_path(name: &Path) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.is_file())
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

    async fn synthesize_with_service(&self, text: &str, lang: &str, output: &Path) -> Result<()> {
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

    pub fn split_tafsir_text(text: &str) -> Vec<String> {
        Self::split_text_for_tts(text, TAFSIR_CHUNK_CHARS)
    }

    fn split_text_for_tts(text: &str, max_chars: usize) -> Vec<String> {
        let mut chunks = Vec::new();
        for paragraph in text.split("\n\n").map(str::trim).filter(|p| !p.is_empty()) {
            if paragraph.chars().count() <= max_chars {
                chunks.push(paragraph.to_owned());
                continue;
            }

            let mut current = String::new();
            for sentence in Self::split_sentences(paragraph) {
                let sentence_len = sentence.chars().count();
                let current_len = current.chars().count();
                let separator_len = usize::from(!current.is_empty());

                if current_len + separator_len + sentence_len > max_chars && !current.is_empty() {
                    chunks.push(current.trim().to_owned());
                    current.clear();
                }

                if !current.is_empty() {
                    current.push(' ');
                }
                current.push_str(&sentence);
            }

            if !current.trim().is_empty() {
                chunks.push(current.trim().to_owned());
            }
        }

        if chunks.is_empty() && !text.trim().is_empty() {
            chunks.push(text.trim().to_owned());
        }
        chunks
    }

    fn split_sentences(text: &str) -> Vec<String> {
        let mut sentences = Vec::new();
        let mut current = String::new();
        for character in text.chars() {
            current.push(character);
            if matches!(character, '.' | '!' | '?' | '।' | '؛') {
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    sentences.push(trimmed.to_owned());
                }
                current.clear();
            }
        }
        let trimmed = current.trim();
        if !trimmed.is_empty() {
            sentences.push(trimmed.to_owned());
        }
        sentences
    }
}

/// Builds a unique sibling path so concurrent synthesis never shares a temp file.
fn temporary_path(target: &Path) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let sequence = COUNTER.fetch_add(1, Ordering::Relaxed);
    let stem = target
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("tts");
    target.with_file_name(format!("{stem}.{}.{sequence}.part", std::process::id()))
}

impl SupertonicService {
    fn new(config: SupertonicConfig) -> Self {
        Self {
            config,
            process: Mutex::new(None),
        }
    }

    async fn synthesize(&self, text: &str, output: &Path, lang: &str) -> Result<()> {
        let mut slot = self.process.lock().await;
        if slot.is_none() {
            *slot = Some(self.spawn().await?);
        }

        let outcome = {
            let process = slot.as_mut().context("TTS service was not started")?;
            timeout(
                SERVICE_REQUEST_TIMEOUT,
                process.synthesize(
                    text,
                    output,
                    lang,
                    &self.config.voice,
                    self.config.total_steps,
                ),
            )
            .await
        };

        match outcome {
            Ok(Ok(())) => Ok(()),
            Ok(Err(error)) => {
                discard(&mut slot).await;
                Err(error)
            }
            Err(_) => {
                discard(&mut slot).await;
                bail!(
                    "TTS synthesis timed out after {}s",
                    SERVICE_REQUEST_TIMEOUT.as_secs()
                )
            }
        }
    }

    async fn spawn(&self) -> Result<SupertonicServiceProcess> {
        let python = self.config.synthesis_python();
        let mut child = Command::new(python)
            .arg(&self.config.script)
            .arg("--service")
            .arg("--device")
            .arg(&self.config.device)
            .arg("--idle-timeout")
            .arg(self.config.idle_timeout_secs.to_string())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("failed to start TTS service via {}", python.display()))?;

        let stdin = child.stdin.take().context("TTS service stdin missing")?;
        let stdout = child.stdout.take().context("TTS service stdout missing")?;

        Ok(SupertonicServiceProcess {
            child,
            stdin,
            stdout: BufReader::new(stdout).lines(),
        })
    }
}

/// Kills and forgets a service whose request stream is no longer trustworthy.
async fn discard(slot: &mut Option<SupertonicServiceProcess>) {
    if let Some(mut process) = slot.take() {
        let _ = process.child.kill().await;
        let _ = process.child.wait().await;
    }
}

impl SupertonicServiceProcess {
    async fn synthesize(
        &mut self,
        text: &str,
        output: &Path,
        lang: &str,
        voice: &str,
        steps: u32,
    ) -> Result<()> {
        let output = output
            .to_str()
            .with_context(|| format!("TTS output path is not valid UTF-8: {}", output.display()))?;

        let request = SupertonicServiceRequest {
            text,
            output,
            lang,
            voice,
            steps,
        };
        let mut line = serde_json::to_string(&request).context("failed to encode TTS request")?;
        line.push('\n');

        self.stdin
            .write_all(line.as_bytes())
            .await
            .context("failed to write TTS request")?;
        self.stdin
            .flush()
            .await
            .context("failed to flush TTS request")?;

        let response_line = self
            .stdout
            .next_line()
            .await
            .context("failed to read TTS response")?
            .context("TTS service exited before responding")?;

        let response: SupertonicServiceResponse = serde_json::from_str(&response_line)
            .with_context(|| format!("invalid TTS response: {response_line}"))?;

        if !response.ok {
            bail!(
                "TTS service failed: {}",
                response.error.unwrap_or_else(|| "unknown error".to_owned())
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_supertonic_script_and_falls_back_to_system_python() {
        let config =
            SupertonicConfig::from_env_or_project_roots(&[PathBuf::from("/tmp/equran-cli")]);

        assert_eq!(
            config.script,
            PathBuf::from("/tmp/equran-cli/tts/tts_supertonic.py")
        );
        assert_eq!(config.python, PathBuf::from("python3"));
    }

    #[test]
    fn prefers_project_venv_python_when_present() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let missing_root = temp_dir.path().join("missing");
        let resource_root = temp_dir.path().join("resources");
        let tts_dir = resource_root.join("tts");
        let venv_bin = tts_dir.join(".venv/bin");
        std::fs::create_dir_all(&venv_bin).expect("venv bin should be created");
        std::fs::write(tts_dir.join("tts_supertonic.py"), "").expect("script should be created");
        std::fs::write(venv_bin.join("python"), "").expect("python should be created");

        let config =
            SupertonicConfig::from_env_or_project_roots(&[missing_root, resource_root.clone()]);

        assert_eq!(config.script, resource_root.join("tts/tts_supertonic.py"));
        assert_eq!(config.python, resource_root.join("tts/.venv/bin/python"));
    }

    #[test]
    fn ignores_gpu_venv_that_cannot_import_supertonic() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let root = temp_dir.path().to_path_buf();
        let tts_dir = root.join("tts");
        let gpu_bin = tts_dir.join(".venv-gpu/bin");
        std::fs::create_dir_all(&gpu_bin).expect("gpu venv bin should be created");
        std::fs::write(tts_dir.join("tts_supertonic.py"), "").expect("script should be created");
        // An interpreter that cannot import the package must not be selected.
        std::fs::write(gpu_bin.join("python"), "#!/bin/sh\nexit 1\n")
            .expect("python stub should be created");
        std::fs::set_permissions(
            gpu_bin.join("python"),
            std::os::unix::fs::PermissionsExt::from_mode(0o755),
        )
        .expect("python stub should be executable");

        let config = SupertonicConfig::from_env_or_project_roots(&[root]);

        assert_eq!(
            config.gpu_python,
            Some(tts_dir.join(".venv-gpu/bin/python"))
        );
        assert_eq!(config.synthesis_python(), Path::new("python3"));
    }

    #[test]
    fn reports_missing_script() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let engine = TtsEngine::with_project_root(
            temp_dir.path().join("cache"),
            temp_dir.path().to_path_buf(),
        );

        assert_eq!(engine.voice_status(), TtsVoiceStatus::MissingScript);
    }

    #[test]
    fn reports_missing_runtime_when_python_cannot_import_supertonic() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let root = temp_dir.path();
        let script = root.join("tts_supertonic.py");
        std::fs::write(&script, "").expect("script should be created");

        let failing_python = root.join("python-failing");
        std::fs::write(&failing_python, "#!/bin/sh\nexit 1\n").expect("python should be created");
        make_executable(&failing_python);

        let config = SupertonicConfig {
            python: failing_python,
            gpu_python: None,
            script,
            voice: DEFAULT_VOICE.to_owned(),
            total_steps: DEFAULT_TOTAL_STEPS,
            device: DEFAULT_DEVICE.to_owned(),
            idle_timeout_secs: DEFAULT_IDLE_TIMEOUT_SECS,
        };

        assert_eq!(status_from_config(&config), TtsVoiceStatus::MissingRuntime);
    }

    #[test]
    fn reports_ready_when_python_imports_supertonic() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let root = temp_dir.path();
        let script = root.join("tts_supertonic.py");
        std::fs::write(&script, "").expect("script should be created");

        let working_python = root.join("python-working");
        std::fs::write(&working_python, "#!/bin/sh\nexit 0\n").expect("python should be created");
        make_executable(&working_python);

        let config = SupertonicConfig {
            python: working_python,
            gpu_python: None,
            script,
            voice: DEFAULT_VOICE.to_owned(),
            total_steps: DEFAULT_TOTAL_STEPS,
            device: DEFAULT_DEVICE.to_owned(),
            idle_timeout_secs: DEFAULT_IDLE_TIMEOUT_SECS,
        };

        assert_eq!(status_from_config(&config), TtsVoiceStatus::Ready);
    }

    fn make_executable(path: &Path) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mut permissions = std::fs::metadata(path)
                .expect("metadata should be readable")
                .permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(path, permissions).expect("permissions should be set");
        }

        #[cfg(not(unix))]
        let _ = path;
    }

    #[test]
    fn splits_tafsir_text_by_paragraphs_and_sentences() {
        let text = "Paragraf pertama cukup pendek. Paragraf pertama kalimat kedua.\n\nParagraf kedua yang juga pendek.";
        let chunks = TtsEngine::split_tafsir_text(text);

        assert_eq!(chunks.len(), 2);
        assert!(chunks[0].contains("kalimat kedua"));
        assert_eq!(chunks[1], "Paragraf kedua yang juga pendek.");
    }

    #[test]
    fn splits_long_tafsir_paragraph_by_sentence_limit() {
        let text = "Kalimat pertama cukup panjang untuk contoh. Kalimat kedua cukup panjang untuk contoh. Kalimat ketiga cukup panjang untuk contoh.";
        let chunks = TtsEngine::split_text_for_tts(text, 78);

        assert!(chunks.len() > 1);
        assert!(chunks.iter().all(|chunk| !chunk.trim().is_empty()));
    }

    #[test]
    fn keeps_whole_text_when_no_boundary_exists() {
        let chunks = TtsEngine::split_text_for_tts("tanpa titik sama sekali", 700);

        assert_eq!(chunks, vec!["tanpa titik sama sekali".to_owned()]);
    }

    /// Writes a stand-in interpreter for tests.
    ///
    /// It satisfies the `import supertonic` readiness probe and otherwise runs
    /// the host Python, so the tests exercise the service protocol instead of
    /// whatever packages the machine happens to have installed.
    fn write_stub_python(path: &Path) {
        use std::os::unix::fs::PermissionsExt;

        std::fs::write(
            path,
            "#!/bin/sh\nif [ \"$1\" = \"-c\" ]; then exit 0; fi\nexec python3 \"$@\"\n",
        )
        .expect("stub interpreter should be written");

        let mut permissions = std::fs::metadata(path)
            .expect("stub should exist")
            .permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions).expect("stub should be executable");
    }

    #[test]
    fn builds_unique_temporary_paths() {
        let target = Path::new("/tmp/supertonic_id_001_001.wav");
        let first = temporary_path(target);
        let second = temporary_path(target);

        assert_ne!(first, second);
        assert_eq!(first.parent(), target.parent());
        assert!(first.to_string_lossy().ends_with(".part"));
    }

    #[tokio::test]
    async fn uses_persistent_service_protocol() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let root = temp_dir.path().to_path_buf();
        let tts_dir = root.join("tts");
        let venv_bin = tts_dir.join(".venv/bin");
        std::fs::create_dir_all(&venv_bin).expect("venv bin should be created");
        // A stub interpreter stands in for the real one: these tests exercise
        // the service protocol, not the machine's Python. Depending on a system
        // interpreter made them fail whenever the host had no supertonic wheel.
        write_stub_python(&venv_bin.join("python"));

        let service_log = root.join("service.log");
        let script = tts_dir.join("tts_supertonic.py");
        std::fs::write(
            &script,
            format!(
                r#"#!/usr/bin/env python3
import argparse, json, pathlib, sys
parser = argparse.ArgumentParser()
parser.add_argument('--service', action='store_true')
parser.add_argument('--text')
parser.add_argument('--output')
parser.add_argument('--lang')
parser.add_argument('--voice')
parser.add_argument('--steps')
parser.add_argument('--device')
parser.add_argument('--idle-timeout')
args = parser.parse_args()
if not args.service:
    sys.exit(17)
pathlib.Path({log:?}).write_text('started')
for line in sys.stdin:
    request = json.loads(line)
    pathlib.Path(request['output']).write_text(request['text'] + '|' + request['lang'] + '|' + request['voice'] + '|' + str(request['steps']))
    print(json.dumps({{'ok': True}}), flush=True)
"#,
                log = service_log.to_string_lossy()
            ),
        )
        .expect("script should be created");

        let engine = TtsEngine::with_project_roots(root.join("cache"), vec![root.clone()]);
        let output = root.join("out.wav");

        engine
            .synthesize_with_service("halo", "id", &output)
            .await
            .expect("service synthesis should succeed");

        assert_eq!(
            std::fs::read_to_string(&output).expect("output"),
            format!("halo|id|M2|{DEFAULT_TOTAL_STEPS}")
        );
        assert_eq!(
            std::fs::read_to_string(&service_log).expect("log"),
            "started"
        );
    }

    #[tokio::test]
    async fn service_error_does_not_poison_next_request() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let root = temp_dir.path().to_path_buf();
        let tts_dir = root.join("tts");
        let venv_bin = tts_dir.join(".venv/bin");
        std::fs::create_dir_all(&venv_bin).expect("venv bin should be created");
        // A stub interpreter stands in for the real one: these tests exercise
        // the service protocol, not the machine's Python. Depending on a system
        // interpreter made them fail whenever the host had no supertonic wheel.
        write_stub_python(&venv_bin.join("python"));

        let marker = root.join("marker");
        let script = tts_dir.join("tts_supertonic.py");
        std::fs::write(
            &script,
            format!(
                r#"#!/usr/bin/env python3
import argparse, json, pathlib, sys
parser = argparse.ArgumentParser()
parser.add_argument('--service', action='store_true')
parser.add_argument('--text')
parser.add_argument('--output')
parser.add_argument('--lang')
parser.add_argument('--voice')
parser.add_argument('--device')
parser.add_argument('--idle-timeout')
args = parser.parse_args()
marker = pathlib.Path({marker:?})
if not marker.exists():
    marker.write_text('failed-once')
    print(json.dumps({{'ok': False, 'error': 'temporary failure'}}), flush=True)
    sys.exit(0)
for line in sys.stdin:
    request = json.loads(line)
    pathlib.Path(request['output']).write_text(request['text'])
    print(json.dumps({{'ok': True}}), flush=True)
"#,
                marker = marker.to_string_lossy()
            ),
        )
        .expect("script should be created");

        let engine = TtsEngine::with_project_roots(root.join("cache"), vec![root.clone()]);
        let first_output = root.join("first.wav");
        let second_output = root.join("second.wav");

        assert!(
            engine
                .synthesize_with_service("first", "id", &first_output)
                .await
                .is_err()
        );
        engine
            .synthesize_with_service("second", "id", &second_output)
            .await
            .expect("service should restart after failure");

        assert_eq!(
            std::fs::read_to_string(&second_output).expect("second output"),
            "second"
        );
    }
}
