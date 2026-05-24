#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"
PYTHON_BIN="${PYTHON_BIN:-python3}"
"$PYTHON_BIN" -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install -r requirements.txt
python -m pip install --no-deps 'git+https://github.com/drat/TTS-Indonesia-Gratis.git'

if [ ! -x .venv/bin/tts ]; then
  echo "error: Coqui TTS binary not found at .venv/bin/tts" >&2
  exit 1
fi

echo "✓ TTS Wibowo environment ready"
