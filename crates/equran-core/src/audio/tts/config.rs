//! Runtime discovery for the Supertonic interpreter and worker script.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Mutex as StdMutex, OnceLock};

use super::{
    DEFAULT_DEVICE, DEFAULT_IDLE_TIMEOUT_SECS, DEFAULT_TOTAL_STEPS, DEFAULT_VOICE, TtsVoiceStatus,
};

pub(super) struct SupertonicConfig {
    pub(super) python: PathBuf,
    pub(super) gpu_python: Option<PathBuf>,
    pub(super) script: PathBuf,
    /// True when `python` is a project venv rather than a system interpreter.
    pub(super) uses_project_venv: bool,
    pub(super) voice: String,
    pub(super) total_steps: u32,
    pub(super) device: String,
    pub(super) idle_timeout_secs: u32,
}

impl SupertonicConfig {
    pub(super) fn from_env_or_project_roots(project_roots: &[PathBuf]) -> Self {
        let script = std::env::var_os("EQURAN_TTS_SUPERTONIC")
            .map(PathBuf::from)
            .unwrap_or_else(|| first_project_file(project_roots, "tts/tts_supertonic.py"));

        let venv_python = first_project_file(project_roots, "tts/.venv/bin/python");
        let uses_project_venv = venv_python.is_file();
        let python = std::env::var_os("EQURAN_TTS_PYTHON")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                if uses_project_venv {
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
            uses_project_venv,
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
    pub(super) fn synthesis_python(&self) -> &Path {
        match &self.gpu_python {
            Some(python) if supertonic_module_available(python) => python,
            _ => &self.python,
        }
    }

    pub(super) fn python_is_usable(&self) -> bool {
        supertonic_module_available(self.synthesis_python())
    }

    pub(super) fn is_usable(&self) -> bool {
        self.script.is_file() && self.python_is_usable()
    }
}

pub(super) fn status_from_config(config: &SupertonicConfig) -> TtsVoiceStatus {
    if !config.script.is_file() {
        return TtsVoiceStatus::MissingScript;
    }
    if !config.python_is_usable() {
        return TtsVoiceStatus::MissingRuntime;
    }
    TtsVoiceStatus::Ready
}

/// Resolves the first `relative_path` that exists under any project root.
pub(super) fn first_project_file(project_roots: &[PathBuf], relative_path: &str) -> PathBuf {
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
pub(super) fn interpreter_version(python: &Path) -> Option<String> {
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
/// queried on every playback command, so a definitive answer is memoized for the
/// lifetime of the process. A bare interpreter name such as `python3` is resolved
/// through `PATH` first, so the cache key is the real executable.
///
/// Only a real answer is remembered. Under load a spawn can fail transiently
/// (`ETXTBSY` just after a file is written, `EAGAIN` when the process table is
/// full); caching such a failure would disable the voice for the rest of the
/// session over a momentary hiccup, so those are retried and, if they persist,
/// left uncached.
pub(super) fn supertonic_module_available(python: &Path) -> bool {
    let key = if python.components().count() == 1 {
        which_in_path(python)
    } else {
        None
    }
    .unwrap_or_else(|| python.to_path_buf());

    let cache = probe_cache();
    if let Ok(guard) = cache.lock()
        && let Some(cached) = guard.get(&key)
    {
        return *cached;
    }

    match probe_supertonic(&key) {
        Some(available) => {
            if let Ok(mut guard) = cache.lock() {
                guard.insert(key, available);
            }
            available
        }
        // Never cache "could not even try"; the next call gets a fresh attempt.
        None => false,
    }
}

/// Forgets every memoized probe answer.
///
/// The setup script can install the package into an interpreter that was
/// already found lacking; without this the app would keep reporting "not ready"
/// for the rest of the session and the reader would run setup twice.
pub fn forget_supertonic_probes() {
    if let Ok(mut guard) = probe_cache().lock() {
        guard.clear();
    }
}

fn probe_cache() -> &'static StdMutex<HashMap<PathBuf, bool>> {
    static CACHE: OnceLock<StdMutex<HashMap<PathBuf, bool>>> = OnceLock::new();
    CACHE.get_or_init(|| StdMutex::new(HashMap::new()))
}

/// Runs the import probe, retrying a transient spawn failure a few times.
///
/// Returns `Some(true/false)` for a real answer (the interpreter ran and either
/// did or did not import the package) and `None` when it could not be executed
/// at all.
fn probe_supertonic(python: &Path) -> Option<bool> {
    const ATTEMPTS: u32 = 4;
    for attempt in 0..ATTEMPTS {
        let result = std::process::Command::new(python)
            .arg("-c")
            .arg("import supertonic")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        match result {
            Ok(status) => return Some(status.success()),
            Err(error) if attempt + 1 < ATTEMPTS => {
                // `ETXTBSY` and friends clear on their own within milliseconds.
                let _ = error;
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(_) => return None,
        }
    }
    None
}

/// Resolves a bare executable name through `PATH`.
fn which_in_path(name: &Path) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.is_file())
}
