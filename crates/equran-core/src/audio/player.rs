use std::{fs::File, path::Path};

#[cfg(feature = "audio")]
use std::{
    io::BufReader,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use anyhow::{Context, Result, bail};

#[cfg(feature = "audio")]
use rodio::{Decoder, OutputStream, Sink};
#[cfg(feature = "audio")]
use tokio_util::sync::CancellationToken;

/// Playback controls shared between the audio thread and the caller.
///
/// The sink lives on the blocking audio thread, so pause/resume is expressed as
/// a flag the thread polls rather than a direct call. This keeps the control
/// surface usable from async code without moving the `Sink` across threads.
#[cfg(feature = "audio")]
#[derive(Clone, Debug, Default)]
pub struct PlaybackControl {
    paused: Arc<AtomicBool>,
}

#[cfg(feature = "audio")]
impl PlaybackControl {
    pub fn new() -> Self {
        Self::default()
    }

    /// Pauses playback. No effect if already paused or finished.
    pub fn pause(&self) {
        self.paused.store(true, Ordering::SeqCst);
    }

    /// Resumes playback. No effect if not paused.
    pub fn resume(&self) {
        self.paused.store(false, Ordering::SeqCst);
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }
}

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

/// Plays `path` while honouring `cancel` (stop) and `control` (pause/resume).
#[cfg(feature = "audio")]
pub async fn play_audio_controlled(
    path: &Path,
    cancel: &CancellationToken,
    control: &PlaybackControl,
) -> Result<()> {
    ensure_readable(path)?;
    play_audio_async_with(path, cancel, control).await
}

#[cfg(feature = "audio")]
async fn play_audio_async(path: &Path, cancel: &CancellationToken) -> Result<()> {
    play_audio_async_with(path, cancel, &PlaybackControl::new()).await
}

#[cfg(feature = "audio")]
async fn play_audio_async_with(
    path: &Path,
    cancel: &CancellationToken,
    control: &PlaybackControl,
) -> Result<()> {
    if !is_likely_supported_audio_path(path) {
        bail!(
            "unsupported audio file extension for {}. Native playback supports MP3 and WAV files",
            path.display()
        );
    }

    let path = path.to_path_buf();
    let cancel = cancel.clone();
    let control = control.clone();
    tokio::task::spawn_blocking(move || play_audio_blocking(&path, &cancel, &control))
        .await
        .context("native audio playback task failed")?
}

#[cfg(feature = "audio")]
fn play_audio_blocking(
    path: &Path,
    cancel: &CancellationToken,
    control: &PlaybackControl,
) -> Result<()> {
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

        // Mirror the shared flag onto the sink. `pause()`/`play()` are cheap
        // and idempotent, so calling them every tick keeps the sink in sync
        // even if the caller toggles faster than the poll interval.
        if control.is_paused() {
            if !sink.is_paused() {
                sink.pause();
            }
        } else if sink.is_paused() {
            sink.play();
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

/// Pause/resume handle. A no-op stub when the `audio` feature is off.
#[cfg(not(feature = "audio"))]
#[derive(Clone, Debug, Default)]
pub struct PlaybackControl;

#[cfg(not(feature = "audio"))]
impl PlaybackControl {
    pub fn new() -> Self {
        Self
    }

    pub fn pause(&self) {}

    pub fn resume(&self) {}

    pub fn is_paused(&self) -> bool {
        false
    }
}

#[cfg(not(feature = "audio"))]
pub async fn play_audio_cancellable(
    path: &Path,
    _cancel: &tokio_util::sync::CancellationToken,
) -> Result<()> {
    play_audio(path)
}

#[cfg(not(feature = "audio"))]
pub async fn play_audio_controlled(
    path: &Path,
    _cancel: &tokio_util::sync::CancellationToken,
    _control: &PlaybackControl,
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
