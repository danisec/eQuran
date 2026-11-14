#!/usr/bin/env bash
# Prepares the local Supertonic 3 runtime used for translation and tafsir audio.
set -euo pipefail

cd "$(dirname "$0")"

# The venv normally sits next to this script. An installed eQuran cannot write
# there (the package directory is root-owned), so the desktop app sets VENV_DIR
# to a writable path under its own data directory and runs this same script.
VENV_DIR="${VENV_DIR:-.venv}"

# onnxruntime publishes wheels only up to a certain CPython version, and the
# newest system Python is often past it. Picking the newest supported
# interpreter here is the difference between a working venv and a pip resolver
# that backtracks forever.
SUPPORTED_VERSIONS=(3.14 3.13 3.12)

find_python() {
  if [[ -n "${PYTHON_BIN:-}" ]]; then
    echo "$PYTHON_BIN"
    return 0
  fi
  for version in "${SUPPORTED_VERSIONS[@]}"; do
    if command -v "python${version}" >/dev/null 2>&1; then
      echo "python${version}"
      return 0
    fi
  done
  return 1
}

if ! PYTHON=$(find_python); then
  cat >&2 <<'EOF'
No supported Python found. onnxruntime requires Python 3.12, 3.13 or 3.14.

Install one of them, then run this script again. On Fedora:
  sudo dnf install python3.12
On Debian/Ubuntu:
  sudo apt install python3.12 python3.12-venv
EOF
  exit 1
fi

echo "Using $("$PYTHON" --version 2>&1) at $(command -v "$PYTHON")"
echo "Installing the runtime into $VENV_DIR"

"$PYTHON" -m venv "$VENV_DIR"
# shellcheck disable=SC1091
source "$VENV_DIR/bin/activate"

python -m pip install --upgrade pip
python -m pip install -r requirements.txt

# Fail fast instead of discovering a broken runtime on the first playback.
python -c "import supertonic, onnxruntime" >/dev/null

echo "✓ Supertonic 3 runtime ready in $VENV_DIR"
