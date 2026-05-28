#!/usr/bin/env python3
import argparse
import json
import os
import sys
import types
from typing import Any

if "simpleaudio" not in sys.modules:
    simpleaudio_stub = types.ModuleType("simpleaudio")

    class _WaveObject:
        @classmethod
        def from_wave_file(cls, _path):
            raise RuntimeError("simpleaudio playback is not used by equran-cli")

    simpleaudio_stub.WaveObject = _WaveObject
    sys.modules["simpleaudio"] = simpleaudio_stub

from g2p_id.scripts.tts import (
    cache_dir,
    config_base_file,
    g2p,
    model_file,
    model_url,
    speakers_file,
    text_normalization,
)
from wget import download


class WibowoSynthesizer:
    def __init__(self) -> None:
        self.config_file = self._prepare_config()
        self.tts = self._load_tts()

    def _prepare_config(self) -> str:
        if not os.path.exists(model_file):
            os.makedirs(cache_dir, exist_ok=True)
            download(model_url, out=cache_dir)

        config_file = os.path.join(cache_dir, "equran-config.json")
        with open(config_base_file) as f:
            config = json.load(f)
        config["model_args"]["speakers_file"] = speakers_file
        with open(config_file, "w") as f:
            json.dump(config, f)
        return config_file

    def _load_tts(self) -> Any:
        try:
            from TTS.api import TTS

            tts = TTS(
                model_path=model_file,
                config_path=self.config_file,
                progress_bar=False,
                gpu=False,
            )
            if hasattr(tts, "to"):
                return tts.to("cpu")
            return tts
        except Exception as api_error:
            try:
                from TTS.utils.synthesizer import Synthesizer

                return Synthesizer(
                    tts_checkpoint=model_file,
                    tts_config_path=self.config_file,
                    tts_speakers_file=speakers_file,
                    tts_languages_file=None,
                    vocoder_checkpoint=None,
                    vocoder_config=None,
                    encoder_checkpoint=None,
                    encoder_config=None,
                    use_cuda=False,
                )
            except Exception as synthesizer_error:
                raise RuntimeError(
                    "failed to load TTS Wibowo model: "
                    f"TTS.api error={api_error}; Synthesizer error={synthesizer_error}"
                ) from synthesizer_error

    def synthesize(self, text: str, output: str) -> None:
        normalized_text = text_normalization(text)
        phonemes = g2p(normalized_text)
        if hasattr(self.tts, "tts_to_file"):
            self.tts.tts_to_file(text=phonemes, speaker="wibowo", file_path=output)
            return

        wav = self.tts.tts(
            text=phonemes,
            speaker_name="wibowo",
            language_name=None,
            speaker_wav=None,
        )
        self.tts.save_wav(wav, output)


def synthesize(text: str, output: str) -> None:
    WibowoSynthesizer().synthesize(text, output)


def run_service() -> None:
    synthesizer = WibowoSynthesizer()
    for line in sys.stdin:
        try:
            request = json.loads(line)
            text = require_string(request, "text")
            output = require_string(request, "output")
            synthesizer.synthesize(text, output)
            write_response({"ok": True})
        except Exception as error:
            write_response({"ok": False, "error": str(error)})


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
    args = parser.parse_args()

    if args.service:
        run_service()
        return

    if not args.text or not args.output:
        parser.error("--text and --output are required unless --service is used")

    synthesize(args.text, args.output)


if __name__ == "__main__":
    main()
