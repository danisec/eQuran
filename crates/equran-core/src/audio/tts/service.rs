//! Persistent JSON-lines worker process used for synthesis.

use std::path::Path;
use std::process::Stdio;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;
use tokio::time::timeout;

use super::SERVICE_REQUEST_TIMEOUT;
use super::config::SupertonicConfig;

pub(super) struct SupertonicService {
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

impl SupertonicService {
    pub(super) fn new(config: SupertonicConfig) -> Self {
        Self {
            config,
            process: Mutex::new(None),
        }
    }

    pub(super) async fn synthesize(&self, text: &str, output: &Path, lang: &str) -> Result<()> {
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
