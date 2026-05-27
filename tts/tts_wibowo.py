#!/usr/bin/env python3
import argparse
import json
import os
import sys
import types
from subprocess import run

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


def synthesize(text: str, output: str) -> None:
    normalized_text = text_normalization(text)
    phonemes = g2p(normalized_text)

    if not os.path.exists(model_file):
        os.makedirs(cache_dir, exist_ok=True)
        download(model_url, out=cache_dir)

    config_file = os.path.join(cache_dir, "equran-config.json")
    with open(config_base_file) as f:
        config = json.load(f)
    config["model_args"]["speakers_file"] = speakers_file
    with open(config_file, "w") as f:
        json.dump(config, f)

    result = run(
        [
            sys.executable,
            "-m",
            "TTS.bin.synthesize",
            "--text",
            phonemes,
            "--model_path",
            model_file,
            "--config_path",
            config_file,
            "--speaker_idx",
            "wibowo",
            "--out_path",
            output,
        ],
        capture_output=True,
        text=True,
        check=False,
    )

    if result.returncode != 0:
        print(result.stderr, file=sys.stderr)
        sys.exit(result.returncode)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--text", required=True)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()
    synthesize(args.text, args.output)


if __name__ == "__main__":
    main()
