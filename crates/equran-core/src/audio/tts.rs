use std::{
    path::{Path, PathBuf},
    process::Stdio,
    sync::Arc,
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
    audio::cache::{TtsBackend, tafsir_tts_chunk_filename, tafsir_tts_filename, tts_filename},
    domain::Lang,
};

struct WibowoConfig {
    python: PathBuf,
    script: PathBuf,
}

struct WibowoService {
    config: WibowoConfig,
    process: Mutex<Option<WibowoServiceProcess>>,
}

struct WibowoServiceProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: Lines<BufReader<ChildStdout>>,
}

#[derive(Debug, Serialize)]
struct WibowoServiceRequest<'a> {
    text: &'a str,
    output: &'a str,
}

#[derive(Debug, Deserialize)]
struct WibowoServiceResponse {
    ok: bool,
    error: Option<String>,
}

const WIBOWO_SERVICE_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NaturalIndonesianStatus {
    Ready,
    MissingRuntime,
    MissingScript,
}

impl NaturalIndonesianStatus {
    pub fn is_ready(self) -> bool {
        matches!(self, Self::Ready)
    }
}

impl WibowoConfig {
    fn from_env_or_project_roots(project_roots: &[PathBuf]) -> Self {
        let script = std::env::var_os("EQURAN_TTS_WIBOWO")
            .map(PathBuf::from)
            .unwrap_or_else(|| Self::first_project_file(project_roots, "tts/tts_wibowo.py"));
        let python = std::env::var_os("EQURAN_TTS_PYTHON")
            .map(PathBuf::from)
            .unwrap_or_else(|| Self::first_project_file(project_roots, "tts/.venv/bin/python"));
        Self { python, script }
    }

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

    fn is_available(&self) -> bool {
        self.python.is_file() && self.script.is_file()
    }

    fn natural_indonesian_status(&self) -> NaturalIndonesianStatus {
        if !self.script.is_file() {
            return NaturalIndonesianStatus::MissingScript;
        }

        if !self.python.is_file() {
            return NaturalIndonesianStatus::MissingRuntime;
        }

        NaturalIndonesianStatus::Ready
    }
}

#[derive(Clone)]
pub struct TtsEngine {
    cache_root: PathBuf,
    project_roots: Vec<PathBuf>,
    wibowo_service: Arc<Mutex<Option<Arc<WibowoService>>>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_default_wibowo_paths_from_project_root() {
        let paths = WibowoConfig::from_env_or_project_roots(&[PathBuf::from("/tmp/equran-cli")]);
        assert_eq!(
            paths.script,
            PathBuf::from("/tmp/equran-cli/tts/tts_wibowo.py")
        );
        assert_eq!(
            paths.python,
            PathBuf::from("/tmp/equran-cli/tts/.venv/bin/python")
        );
    }

    #[test]
    fn resolves_wibowo_paths_from_first_available_project_root() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let missing_root = temp_dir.path().join("missing");
        let resource_root = temp_dir.path().join("resources");
        let tts_dir = resource_root.join("tts");
        let venv_bin = tts_dir.join(".venv/bin");
        std::fs::create_dir_all(&venv_bin).expect("venv bin should be created");
        std::fs::write(tts_dir.join("tts_wibowo.py"), "").expect("script should be created");
        std::fs::write(venv_bin.join("python"), "").expect("python should be created");

        let paths = WibowoConfig::from_env_or_project_roots(&[missing_root, resource_root.clone()]);

        assert_eq!(paths.script, resource_root.join("tts/tts_wibowo.py"));
        assert_eq!(paths.python, resource_root.join("tts/.venv/bin/python"));
    }

    #[test]
    fn reports_missing_natural_indonesian_script() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let root = temp_dir.path().to_path_buf();
        std::fs::create_dir_all(root.join("tts/.venv/bin")).expect("venv bin should be created");
        std::fs::write(root.join("tts/.venv/bin/python"), "").expect("python should be created");

        let engine = TtsEngine::with_project_root(root.join("cache"), root);

        assert_eq!(
            engine.natural_indonesian_status(),
            NaturalIndonesianStatus::MissingScript
        );
    }

    #[test]
    fn reports_missing_natural_indonesian_runtime() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let root = temp_dir.path().to_path_buf();
        std::fs::create_dir_all(root.join("tts")).expect("tts dir should be created");
        std::fs::write(root.join("tts/tts_wibowo.py"), "").expect("script should be created");

        let engine = TtsEngine::with_project_root(root.join("cache"), root);

        assert_eq!(
            engine.natural_indonesian_status(),
            NaturalIndonesianStatus::MissingRuntime
        );
    }

    #[test]
    fn reports_ready_natural_indonesian_voice() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let root = temp_dir.path().to_path_buf();
        std::fs::create_dir_all(root.join("tts/.venv/bin")).expect("venv bin should be created");
        std::fs::write(root.join("tts/tts_wibowo.py"), "").expect("script should be created");
        std::fs::write(root.join("tts/.venv/bin/python"), "").expect("python should be created");

        let engine = TtsEngine::with_project_root(root.join("cache"), root);

        assert_eq!(
            engine.natural_indonesian_status(),
            NaturalIndonesianStatus::Ready
        );
        assert!(engine.natural_indonesian_status().is_ready());
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

    #[tokio::test]
    async fn uses_persistent_wibowo_service_protocol_when_enabled() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let root = temp_dir.path().to_path_buf();
        let tts_dir = root.join("tts");
        let venv_bin = tts_dir.join(".venv/bin");
        std::fs::create_dir_all(&venv_bin).expect("venv bin should be created");
        std::os::unix::fs::symlink("/usr/bin/python3", venv_bin.join("python"))
            .expect("python symlink should be created");
        let service_log = root.join("service.log");
        let script = tts_dir.join("tts_wibowo.py");
        std::fs::write(
            &script,
            format!(
                r#"#!/usr/bin/env python3
import argparse, json, pathlib, sys
parser = argparse.ArgumentParser()
parser.add_argument('--service', action='store_true')
args = parser.parse_args()
if not args.service:
    sys.exit(17)
pathlib.Path({log:?}).write_text('started')
for line in sys.stdin:
    request = json.loads(line)
    pathlib.Path(request['output']).write_text(request['text'])
    print(json.dumps({{'ok': True}}), flush=True)
"#,
                log = service_log.to_string_lossy()
            ),
        )
        .expect("script should be created");

        let engine = TtsEngine::with_project_roots(root.join("cache"), vec![root.clone()]);
        let output = root.join("out.wav");

        engine
            .synthesize_with_wibowo_service("halo", &output)
            .await
            .expect("service synthesis should succeed");

        assert_eq!(std::fs::read_to_string(&output).expect("output"), "halo");
        assert_eq!(
            std::fs::read_to_string(&service_log).expect("log"),
            "started"
        );
    }

    #[tokio::test]
    async fn wibowo_service_error_does_not_poison_next_request() {
        let temp_dir = tempfile::tempdir().expect("temp dir should be created");
        let root = temp_dir.path().to_path_buf();
        let tts_dir = root.join("tts");
        let venv_bin = tts_dir.join(".venv/bin");
        std::fs::create_dir_all(&venv_bin).expect("venv bin should be created");
        std::os::unix::fs::symlink("/usr/bin/python3", venv_bin.join("python"))
            .expect("python symlink should be created");
        let marker = root.join("marker");
        let script = tts_dir.join("tts_wibowo.py");
        std::fs::write(
            &script,
            format!(
                r#"#!/usr/bin/env python3
import argparse, json, pathlib, sys
parser = argparse.ArgumentParser()
parser.add_argument('--service', action='store_true')
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
                .synthesize_with_wibowo_service("first", &first_output)
                .await
                .is_err()
        );
        engine
            .synthesize_with_wibowo_service("second", &second_output)
            .await
            .expect("service should restart after failure");

        assert_eq!(
            std::fs::read_to_string(&second_output).expect("second output"),
            "second"
        );
    }
}

impl TtsEngine {
    pub fn new(cache_root: PathBuf) -> Self {
        let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            cache_root,
            project_roots: vec![project_root],
            wibowo_service: Arc::new(Mutex::new(None)),
        }
    }

    pub fn with_project_root(cache_root: PathBuf, project_root: PathBuf) -> Self {
        Self {
            cache_root,
            project_roots: vec![project_root],
            wibowo_service: Arc::new(Mutex::new(None)),
        }
    }

    pub fn with_project_roots(cache_root: PathBuf, project_roots: Vec<PathBuf>) -> Self {
        Self {
            cache_root,
            project_roots,
            wibowo_service: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn synthesize_cached(
        &self,
        text: &str,
        lang: Lang,
        surah: u8,
        ayah: u16,
    ) -> Result<PathBuf> {
        let backend = self.active_backend(lang)?;
        let tts_dir = self.cache_root.join("tts");
        fs::create_dir_all(&tts_dir)
            .await
            .with_context(|| format!("failed to create {}", tts_dir.display()))?;
        let target = tts_dir.join(tts_filename(backend, lang.code(), surah, ayah));
        if target.exists() {
            return Ok(target);
        }
        self.synthesize(text, lang, &target).await?;
        Ok(target)
    }

    pub async fn synthesize_tafsir_edge_cached(
        &self,
        text: &str,
        surah: u8,
        ayah: u16,
    ) -> Result<PathBuf> {
        let tts_dir = self.cache_root.join("tts");
        fs::create_dir_all(&tts_dir)
            .await
            .with_context(|| format!("failed to create {}", tts_dir.display()))?;
        let target = tts_dir.join(tafsir_tts_filename(Lang::Id.code(), surah, ayah));
        if target.exists() {
            return Ok(target);
        }
        self.synthesize_with_edge_tts(text, Lang::Id, &target)
            .await?;
        Ok(target)
    }

    pub async fn synthesize_tafsir_chunk_cached(
        &self,
        text: &str,
        surah: u8,
        ayah: u16,
        chunk: usize,
    ) -> Result<PathBuf> {
        let backend = self.active_backend(Lang::Id)?;
        let tts_dir = self.cache_root.join("tts");
        fs::create_dir_all(&tts_dir)
            .await
            .with_context(|| format!("failed to create {}", tts_dir.display()))?;
        let target = tts_dir.join(tafsir_tts_chunk_filename(
            backend,
            Lang::Id.code(),
            surah,
            ayah,
            chunk,
        ));
        if target.exists() {
            return Ok(target);
        }
        self.synthesize(text, Lang::Id, &target).await?;
        Ok(target)
    }

    pub fn split_tafsir_text(text: &str) -> Vec<String> {
        Self::split_text_for_tts(text, 700)
    }

    pub fn active_backend(&self, lang: Lang) -> Result<TtsBackend> {
        if lang == Lang::Id {
            let config = WibowoConfig::from_env_or_project_roots(&self.project_roots);
            if config.is_available() {
                return Ok(TtsBackend::Wibowo);
            }
        }

        if Self::is_edge_tts_available() {
            return Ok(TtsBackend::EdgeTts);
        }

        bail!("No TTS backend available. Configure TTS Wibowo or install edge-tts.")
    }

    pub fn natural_indonesian_status(&self) -> NaturalIndonesianStatus {
        WibowoConfig::from_env_or_project_roots(&self.project_roots).natural_indonesian_status()
    }

    fn is_edge_tts_available() -> bool {
        std::process::Command::new("edge-tts")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok()
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

    async fn synthesize(&self, text: &str, lang: Lang, output_path: &Path) -> Result<()> {
        if lang == Lang::Id {
            if self
                .synthesize_with_wibowo_service(text, output_path)
                .await
                .is_ok()
            {
                return Ok(());
            }

            if self.synthesize_with_wibowo(text, output_path).await.is_ok() {
                return Ok(());
            }
        }

        if self
            .synthesize_with_edge_tts(text, lang, output_path)
            .await
            .is_ok()
        {
            return Ok(());
        }

        bail!("No TTS backend available. Configure TTS Wibowo or install edge-tts.")
    }

    async fn synthesize_with_wibowo(&self, text: &str, output_path: &Path) -> Result<()> {
        let config = WibowoConfig::from_env_or_project_roots(&self.project_roots);
        if !config.is_available() {
            bail!("TTS Wibowo is not configured");
        }

        let output = Command::new(&config.python)
            .arg(&config.script)
            .arg("--text")
            .arg(text)
            .arg("--output")
            .arg(output_path)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .await
            .with_context(|| {
                format!("failed to start TTS Wibowo via {}", config.python.display())
            })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("TTS Wibowo failed: {stderr}");
        }
        Ok(())
    }

    async fn synthesize_with_wibowo_service(&self, text: &str, output_path: &Path) -> Result<()> {
        let service = self.wibowo_service().await?;
        timeout(
            WIBOWO_SERVICE_REQUEST_TIMEOUT,
            service.synthesize(text, output_path),
        )
        .await
        .context("TTS Wibowo service request timed out")?
    }

    async fn synthesize_with_edge_tts(
        &self,
        text: &str,
        lang: Lang,
        output_path: &Path,
    ) -> Result<()> {
        let voice = match lang {
            Lang::Id => "id-ID-ArdiNeural",
            Lang::En => "en-US-ChristopherNeural",
        };
        let output = Command::new("edge-tts")
            .arg("--text")
            .arg(text)
            .arg("--voice")
            .arg(voice)
            .arg("--write-media")
            .arg(output_path)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("failed to start edge-tts. Install with: pip install edge-tts")?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("edge-tts failed: {stderr}");
        }
        Ok(())
    }

    async fn wibowo_service(&self) -> Result<Arc<WibowoService>> {
        let mut service = self.wibowo_service.lock().await;
        if let Some(service) = service.as_ref() {
            return Ok(service.clone());
        }

        let config = WibowoConfig::from_env_or_project_roots(&self.project_roots);
        if !config.is_available() {
            bail!("TTS Wibowo is not configured");
        }
        let created = Arc::new(WibowoService::new(config));
        *service = Some(created.clone());
        Ok(created)
    }
}

impl WibowoService {
    fn new(config: WibowoConfig) -> Self {
        Self {
            config,
            process: Mutex::new(None),
        }
    }

    async fn synthesize(&self, text: &str, output_path: &Path) -> Result<()> {
        let mut process = self.process.lock().await;
        if process.is_none() {
            *process = Some(self.spawn().await?);
        }

        let result = match process.as_mut() {
            Some(process) => process.synthesize(text, output_path).await,
            None => bail!("TTS Wibowo service was not started"),
        };

        if result.is_err() {
            if let Some(mut failed_process) = process.take() {
                let _ = failed_process.child.kill().await;
            }
        }

        result
    }

    async fn spawn(&self) -> Result<WibowoServiceProcess> {
        let mut child = Command::new(&self.config.python)
            .arg(&self.config.script)
            .arg("--service")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| {
                format!(
                    "failed to start persistent TTS Wibowo service via {}",
                    self.config.python.display()
                )
            })?;

        let stdin = child
            .stdin
            .take()
            .context("TTS Wibowo service stdin missing")?;
        let stdout = child
            .stdout
            .take()
            .context("TTS Wibowo service stdout missing")?;

        Ok(WibowoServiceProcess {
            child,
            stdin,
            stdout: BufReader::new(stdout).lines(),
        })
    }
}

impl WibowoServiceProcess {
    async fn synthesize(&mut self, text: &str, output_path: &Path) -> Result<()> {
        let output = output_path.to_str().with_context(|| {
            format!(
                "TTS output path is not valid UTF-8: {}",
                output_path.display()
            )
        })?;
        let request = WibowoServiceRequest { text, output };
        let mut line = serde_json::to_string(&request).context("failed to encode TTS request")?;
        line.push('\n');
        self.stdin
            .write_all(line.as_bytes())
            .await
            .context("failed to write TTS Wibowo service request")?;
        self.stdin
            .flush()
            .await
            .context("failed to flush TTS Wibowo service request")?;

        let response_line = self
            .stdout
            .next_line()
            .await
            .context("failed to read TTS Wibowo service response")?
            .context("TTS Wibowo service exited before responding")?;
        let response: WibowoServiceResponse = serde_json::from_str(&response_line)
            .with_context(|| format!("invalid TTS Wibowo service response: {response_line}"))?;
        if !response.ok {
            bail!(
                "TTS Wibowo service failed: {}",
                response.error.unwrap_or_else(|| "unknown error".to_owned())
            );
        }
        Ok(())
    }
}
