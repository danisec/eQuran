use std::{fs::File, path::Path};

use anyhow::{Context, Result, bail};

#[cfg(feature = "audio")]
use tokio::process::Command;
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
    let candidates = [
        ("mpv", vec!["--no-video", "--really-quiet"]),
        ("ffplay", vec!["-nodisp", "-autoexit", "-loglevel", "quiet"]),
        ("paplay", vec![]),
        ("aplay", vec![]),
    ];

    for (program, args) in &candidates {
        let child = Command::new(program)
            .args(args)
            .arg(path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn();

        let mut child = match child {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => bail!("{program} failed to start: {e}"),
        };

        tokio::select! {
            _ = cancel.cancelled() => {
                let _ = child.kill().await;
                return Ok(());
            }
            status = child.wait() => {
                match status {
                    Ok(s) if s.success() => return Ok(()),
                    Ok(s) => bail!("{program} failed with exit status {s}"),
                    Err(e) => bail!("{program} failed: {e}"),
                }
            }
        }
    }

    bail!(
        "no supported audio player found. Install mpv, ffmpeg/ffplay, pulseaudio-utils, or alsa-utils"
    )
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
