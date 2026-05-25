#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# linuxdeploy bundles an older strip binary that fails on modern distro
# libraries using ELF .relr.dyn sections. Disabling strip keeps AppImage
# bundling working on current Linux toolchains.
export NO_STRIP=1

# Some systems mount AppImages with restrictions; extract-and-run is the
# recommended non-FUSE fallback used by AppImage tooling in CI/headless builds.
export APPIMAGE_EXTRACT_AND_RUN=1

cd "$ROOT_DIR/crates/equran-desktop"
cargo tauri build --bundles appimage
