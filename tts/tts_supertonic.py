#!/usr/bin/env python3
"""Supertonic 3 speech synthesis for eQuran.

Two modes:

* ``--text/--output`` renders a single utterance and exits (CLI fallback).
* ``--service`` keeps the model resident and answers newline-delimited JSON
  requests on stdin, which avoids paying the ~7 s model load for every chunk.
"""
import argparse
import json
import os
import re
import select
import sys
import wave
from typing import Any

DEFAULT_VOICE = os.environ.get("EQURAN_TTS_SUPERTONIC_VOICE", "M2")
# Supertonic defaults to 8 diffusion steps. Fewer steps cut synthesis time
# roughly linearly; 5 keeps narration quality while being ~40% faster.
DEFAULT_STEPS = int(os.environ.get("EQURAN_TTS_SUPERTONIC_STEPS", "5"))
DEFAULT_DEVICE = os.environ.get("EQURAN_TTS_DEVICE", "auto")
MAX_DEFAULT_THREADS = 8
# Seconds of inactivity after which the service exits and releases the model.
DEFAULT_IDLE_TIMEOUT = int(os.environ.get("EQURAN_TTS_IDLE_TIMEOUT", "600"))
PROVIDER_CHAIN = (
    ("TensorrtExecutionProvider", "TensorRT"),
    ("CUDAExecutionProvider", "CUDA"),
    ("ROCMExecutionProvider", "ROCm"),
    ("MIGraphXExecutionProvider", "MIGraphX"),
    ("DmlExecutionProvider", "DirectML"),
    ("CoreMLExecutionProvider", "CoreML"),
    ("OpenVINOExecutionProvider", "OpenVINO"),
)
CPU_PROVIDER = "CPUExecutionProvider"


def thread_budget() -> int:
    override = os.environ.get("EQURAN_TTS_THREADS", "")
    if override.isdigit() and int(override) > 0:
        return int(override)
    cpus = os.cpu_count() or 4
    return max(1, min(cpus // 2 if cpus >= 4 else cpus, MAX_DEFAULT_THREADS))


def provider_chain(device: str) -> list[str]:
    import onnxruntime as ort

    available = set(ort.get_available_providers())
    override = os.environ.get("EQURAN_TTS_ONNX_PROVIDERS", "")
    explicit = [part.strip() for part in override.split(",") if part.strip()]
    if explicit:
        wanted = explicit
    elif device == "cpu":
        wanted = [CPU_PROVIDER]
    else:
        wanted = [name for name, _ in PROVIDER_CHAIN]
    chosen = [name for name in wanted if name in available]
    if CPU_PROVIDER not in chosen:
        chosen.append(CPU_PROVIDER)
    return chosen


class SupertonicSynthesizer:
    def __init__(self, voice: str = DEFAULT_VOICE, device: str = DEFAULT_DEVICE) -> None:
        self.voice_name = voice
        self.providers = provider_chain(device)
        self.threads = thread_budget()
        self.tts = self._load_tts()
        self.style = self.tts.get_voice_style(voice_name=self.voice_name)

    def _load_tts(self) -> Any:
        try:
            import supertonic.loader

            supertonic.loader.DEFAULT_ONNX_PROVIDERS = list(self.providers)
            from supertonic import TTS

            return TTS(
                auto_download=True,
                intra_op_num_threads=self.threads,
                inter_op_num_threads=1,
            )
        except Exception as error:
            raise RuntimeError(f"failed to load Supertonic TTS: {error}") from error

    def describe(self) -> dict[str, Any]:
        return {"providers": self.providers, "threads": self.threads}

    def speakable_text(self, text: str) -> str:
        """Remove characters the loaded model cannot encode.

        The Rust side already drops Arabic and Urdu script from the narration,
        but the model's character table is the final authority. Filtering with
        it here means one unexpected symbol becomes a short pause instead of
        failing the request and stopping the tafsir mid-surah.
        """
        processor = self.tts.model.text_processor
        kept = []
        for character in text:
            if character.isspace() or processor.validate_text(character)[0]:
                kept.append(character)
            else:
                kept.append(" ")

        cleaned = re.sub(r"\s+", " ", "".join(kept)).strip()
        if not cleaned:
            raise ValueError("text has nothing the narrator can read")
        return cleaned

    def synthesize(
        self,
        text: str,
        output: str,
        lang: str = "id",
        steps: int = DEFAULT_STEPS,
    ) -> None:
        text = self.speakable_text(text)
        wav, _duration = self.tts.synthesize(
            text=text,
            voice_style=self.style,
            lang=lang,
            total_steps=steps,
            verbose=False,
        )

        import numpy as np

        audio = np.asarray(wav).squeeze()
        if audio.ndim > 1:
            audio = audio.reshape(-1)
        audio = np.clip(audio, -1.0, 1.0)
        pcm = (audio * 32767.0).astype(np.int16)

        with wave.open(output, "wb") as wav_file:
            wav_file.setnchannels(1)
            wav_file.setsampwidth(2)
            wav_file.setframerate(self.tts.sample_rate)
            wav_file.writeframes(pcm.tobytes())


def synthesize(
    text: str,
    output: str,
    lang: str = "id",
    voice: str = DEFAULT_VOICE,
    steps: int = DEFAULT_STEPS,
    device: str = DEFAULT_DEVICE,
) -> None:
    SupertonicSynthesizer(voice=voice, device=device).synthesize(text, output, lang, steps)


def run_service(device: str = DEFAULT_DEVICE, idle_timeout: int = DEFAULT_IDLE_TIMEOUT) -> None:
    synthesizer = None
    current_voice = None

    while True:
        line = read_request(idle_timeout)
        if line is None:
            # Idle: exit so the model and its thread pool are released instead
            # of holding ~1 GB resident for an app that is not speaking.
            return

        try:
            request = json.loads(line)
            text = require_string(request, "text")
            output = require_string(request, "output")
            lang = request.get("lang") or "id"
            voice = request.get("voice") or DEFAULT_VOICE
            steps = int(request.get("steps") or DEFAULT_STEPS)

            if synthesizer is None or current_voice != voice:
                synthesizer = SupertonicSynthesizer(voice=voice, device=device)
                current_voice = voice

            synthesizer.synthesize(text, output, lang, steps)
            write_response({"ok": True})
        except Exception as error:
            write_response({"ok": False, "error": str(error)})


def read_request(idle_timeout: int) -> str | None:
    """Return one request line, or None once the service has been idle too long."""
    if idle_timeout > 0:
        try:
            readable, _, _ = select.select([sys.stdin], [], [], idle_timeout)
        except (OSError, ValueError):
            readable = [sys.stdin]
        if not readable:
            return None

    line = sys.stdin.readline()
    return line or None


def require_string(request: Any, key: str) -> str:
    if not isinstance(request, dict):
        raise ValueError("request must be a JSON object")
    value = request.get(key)
    if not isinstance(value, str) or not value:
        raise ValueError(f"request.{key} must be a non-empty string")
    return value


def write_response(response: dict[str, Any]) -> None:
    print(json.dumps(response), flush=True)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--service", action="store_true")
    parser.add_argument("--text")
    parser.add_argument("--output")
    parser.add_argument("--lang", default="id")
    parser.add_argument("--voice", default=DEFAULT_VOICE)
    parser.add_argument("--steps", type=int, default=DEFAULT_STEPS)
    parser.add_argument("--device", default=DEFAULT_DEVICE, choices=("auto", "cpu", "gpu"))
    parser.add_argument("--idle-timeout", type=int, default=DEFAULT_IDLE_TIMEOUT)
    args = parser.parse_args()

    if args.service:
        run_service(device=args.device, idle_timeout=args.idle_timeout)
        return

    if not args.text or not args.output:
        parser.error("--text and --output are required unless --service is used")

    synthesize(args.text, args.output, args.lang, args.voice, args.steps, args.device)


if __name__ == "__main__":
    main()
