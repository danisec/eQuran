use std::{fs::File, path::Path};

#[cfg(feature = "audio")]
use std::{io::BufReader, time::Duration};

use anyhow::{Context, Result, bail};

#[cfg(feature = "audio")]
use rodio::{Decoder, OutputStream, Sink};
#[cfg(feature = "audio")]
use tokio_util::sync::CancellationToken;

#[cfg(feature = "audio")]
pub fn play_audio(path: &Path) -> Result<()> {
    ensure_readable(path)?;
    let rt = tokio::runtime::Handle::current();
    rt.block_on(play_audio_async(path, &CancellationToken::new()))
}

#[cfg(feature = "audio")]
pub async fn play_audio_cancellable(path: &Path, cancel: &CancellationToken) -> Result<()> {
    ensure_readable(path)?;
    play_audio_async(path, cancel).await
}

#[cfg(feature = "audio")]
async fn play_audio_async(path: &Path, cancel: &CancellationToken) -> Result<()> {
    if !is_likely_supported_audio_path(path) {
        bail!(
            "unsupported audio file extension for {}. Native playback supports MP3 and WAV files",
            path.display()
        );
    }

    let path = path.to_path_buf();
    let cancel = cancel.clone();
    tokio::task::spawn_blocking(move || play_audio_blocking(&path, &cancel))
        .await
        .context("native audio playback task failed")?
}

#[cfg(feature = "audio")]
fn play_audio_blocking(path: &Path, cancel: &CancellationToken) -> Result<()> {
    let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let source = Decoder::new(BufReader::new(file))
        .with_context(|| format!("failed to decode {} as MP3 or WAV audio", path.display()))?;
    let (_stream, stream_handle) =
        OutputStream::try_default().context("failed to open the default audio output device")?;
    let sink = Sink::try_new(&stream_handle).context("failed to create native audio sink")?;

    sink.append(source);
    while !sink.empty() {
        if cancel.is_cancelled() {
            sink.stop();
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(25));
    }

    Ok(())
}

#[cfg(not(feature = "audio"))]
pub fn play_audio(path: &Path) -> Result<()> {
    ensure_readable(path)?;
    bail!("audio playback is disabled in this build. Rebuild with `--features audio`")
}

#[cfg(not(feature = "audio"))]
pub async fn play_audio_cancellable(
    path: &Path,
    _cancel: &tokio_util::sync::CancellationToken,
) -> Result<()> {
    play_audio(path)
}

fn ensure_readable(path: &Path) -> Result<()> {
    File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    Ok(())
}

#[cfg(any(feature = "audio", test))]
fn is_likely_supported_audio_path(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some("mp3" | "wav")
    )
}

#[cfg(test)]
mod tests {
    use super::is_likely_supported_audio_path;
    use std::path::Path;

    #[test]
    fn recognizes_mp3_and_wav_paths_as_supported_audio() {
        assert!(is_likely_supported_audio_path(Path::new("recitation.MP3")));
        assert!(is_likely_supported_audio_path(Path::new("translation.wav")));
    }

    #[test]
    fn rejects_paths_without_supported_audio_extensions() {
        assert!(!is_likely_supported_audio_path(Path::new("notes.txt")));
        assert!(!is_likely_supported_audio_path(Path::new("audio")));
    }
}
