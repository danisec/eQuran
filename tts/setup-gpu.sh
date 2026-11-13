#!/usr/bin/env bash
# Installs a GPU-accelerated ONNX Runtime for Supertonic into tts/.venv-gpu.
#
# The CPU runtime in tts/.venv (or the system python3) stays untouched: eQuran
# prefers .venv-gpu only when it can import supertonic, so a failed install
# simply falls back to CPU. tts_supertonic.py then picks up whichever execution
# providers the wheel ships and keeps CPU as the fallback for unsupported nodes.
set -euo pipefail

cd "$(dirname "$0")"
PYTHON_BIN="${PYTHON_BIN:-python3}"
VENV_DIR="${VENV_DIR:-.venv-gpu}"

detect_backend() {
    case "$(uname -s)" in
        Darwin)
            echo "coreml"
            return
            ;;
        MINGW* | MSYS* | CYGWIN*)
            echo "directml"
            return
            ;;
    esac

    if command -v nvidia-smi >/dev/null 2>&1 && nvidia-smi -L >/dev/null 2>&1; then
        echo "cuda"
        return
    fi

    if [ -d /opt/rocm ] || command -v rocminfo >/dev/null 2>&1; then
        echo "rocm"
        return
    fi

    echo "none"
}

BACKEND="${BACKEND:-$(detect_backend)}"

case "$BACKEND" in
    cuda) WHEEL="onnxruntime-gpu" ;;
    rocm) WHEEL="onnxruntime-rocm" ;;
    directml) WHEEL="onnxruntime-directml" ;;
    coreml) WHEEL="onnxruntime" ;;
    none)
        echo "No supported GPU backend detected (no NVIDIA driver, no ROCm, not macOS/Windows)."
        echo "eQuran will keep using the CPU runtime; nothing to do."
        exit 0
        ;;
    *)
        echo "Unknown BACKEND '$BACKEND' (expected cuda|rocm|directml|coreml|none)." >&2
        exit 2
        ;;
esac

echo "Installing $WHEEL into $VENV_DIR (backend: $BACKEND)"

"$PYTHON_BIN" -m venv "$VENV_DIR"
# shellcheck disable=SC1091
source "$VENV_DIR/bin/activate"

python -m pip install --upgrade pip
python -m pip install -r requirements.txt
python -m pip install --upgrade --force-reinstall "$WHEEL"

python - <<'PY'
import onnxruntime as ort

providers = ort.get_available_providers()
print("onnxruntime", ort.__version__)
print("providers:", ", ".join(providers))
if providers == ["CPUExecutionProvider"]:
    raise SystemExit(
        "The installed wheel exposes no GPU provider; eQuran will fall back to CPU."
    )
PY

echo "✓ GPU runtime ready in $VENV_DIR"
