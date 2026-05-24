use std::{
    path::{Path, PathBuf},
    process::Stdio,
};

use anyhow::{Context, Result, bail};
use tokio::{fs, process::Command};

use crate::{
    audio::cache::{TtsBackend, tafsir_tts_chunk_filename, tafsir_tts_filename, tts_filename},
    domain::Lang,
};

struct WibowoConfig {
    python: PathBuf,
    script: PathBuf,
}

impl WibowoConfig {
    fn from_env_or_project_root(project_root: &Path) -> Self {
        let script = std::env::var_os("EQURAN_TTS_WIBOWO")
            .map(PathBuf::from)
            .unwrap_or_else(|| project_root.join("tts/tts_wibowo.py"));
        let python = std::env::var_os("EQURAN_TTS_PYTHON")
            .map(PathBuf::from)
            .unwrap_or_else(|| project_root.join("tts/.venv/bin/python"));
        Self { python, script }
    }

    fn is_available(&self) -> bool {
        self.python.is_file() && self.script.is_file()
    }
}

#[derive(Clone)]
pub struct TtsEngine {
    cache_root: PathBuf,
    project_root: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_default_wibowo_paths_from_project_root() {
        let paths = WibowoConfig::from_env_or_project_root(Path::new("/tmp/equran-cli"));
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
}

impl TtsEngine {
    pub fn new(cache_root: PathBuf) -> Self {
        let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            cache_root,
            project_root,
        }
    }

    pub fn with_project_root(cache_root: PathBuf, project_root: PathBuf) -> Self {
        Self {
            cache_root,
            project_root,
        }
    }

    pub async fn synthesize_cached(
        &self,
        text: &str,
        lang: Lang,
        surah: u8,
        ayah: u16,
    ) -> Result<PathBuf> {
        let backend = self.active_backend(lang);
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
        let backend = self.active_backend(Lang::Id);
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

    pub fn active_backend(&self, lang: Lang) -> TtsBackend {
        if lang == Lang::Id {
            let config = WibowoConfig::from_env_or_project_root(&self.project_root);
            if config.is_available() {
                return TtsBackend::Wibowo;
            }
        }

        if Self::is_edge_tts_available() {
            return TtsBackend::EdgeTts;
        }

        TtsBackend::Espeak
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
        if lang == Lang::Id && self.synthesize_with_wibowo(text, output_path).await.is_ok() {
            return Ok(());
        }

        if self
            .synthesize_with_edge_tts(text, lang, output_path)
            .await
            .is_ok()
        {
            return Ok(());
        }

        self.synthesize_with_espeak(text, lang, output_path).await
    }

    async fn synthesize_with_wibowo(&self, text: &str, output_path: &Path) -> Result<()> {
        let config = WibowoConfig::from_env_or_project_root(&self.project_root);
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

    async fn synthesize_with_espeak(
        &self,
        text: &str,
        lang: Lang,
        output_path: &Path,
    ) -> Result<()> {
        let voice = match lang {
            Lang::Id => "id",
            Lang::En => "en",
        };
        let output = Command::new("espeak-ng")
            .arg("-v")
            .arg(voice)
            .arg("-w")
            .arg(output_path)
            .arg(text)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("failed to start espeak-ng fallback TTS. Install edge-tts or espeak-ng")?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("espeak-ng failed: {stderr}");
        }
        Ok(())
    }
}
