# Changelog

Notable changes to eQuran. This project follows [semantic
versioning](https://semver.org) for its public interfaces once they settle;
until 1.0 is tagged on `master`, treat the CLI flags and the settings file as the
things most worth watching.

## Unreleased

### Added

* Windows speech-runtime setup: the app now runs `tts/setup.ps1` through Windows
  PowerShell (with `-ExecutionPolicy Bypass` scoped to that invocation) and
  discovers the venv interpreter at `Scripts\python.exe`, so the translation and
  tafsir voice can be installed on Windows instead of reporting the runtime as
  missing. The manual setup hint in Settings is likewise PowerShell on Windows.
* The Windows installer now bundles a relocatable CPython under `tts/python/`,
  so **Set up voice** works on a machine with no system Python (no more
  `VCRUNTIME140.dll`/`python was not found` dead end). The app prefers the
  bundled interpreter when no venv exists yet, and `setup.ps1` no longer mistakes
  the Microsoft Store `python` alias for a real interpreter.

### Fixed

* The Windows installer no longer depends on the Visual C++ Redistributable. It
  previously shipped a binary that imported `VCRUNTIME140.dll` and
  `VCRUNTIME140_1.dll`, so a machine without the redist refused to launch with
  "VCRUNTIME140.dll was not found". The MSVC target now builds with `+crt-static`,
  embedding the runtime into the executable.

## 1.1.4

### Added

* The home index switches between four groupings instead of listing surahs 1–114
  only: **Juz** (30 titled groups, each showing the slice of the surah the juz
  holds and a `lanjutan` marker when it starts mid-surah), **Revelation order**
  (the Egyptian order) and **A–Z** with a direction toggle. Juz boundaries and
  the order of revelation are fixed tables, so no request is needed.

### Fixed

* The chosen home grouping and the A–Z direction survive a trip into the reader.
  The state used to live inside the home component, which unmounts when a surah
  opens, so the index snapped back to **Surah** on every return; it now lives in
  a store and is remembered in `localStorage`.

## 1.1.3

### Changed

* A new application icon: an open mushaf on ink with the verse rosette as its
  seal, drawn in the app's own palette instead of a green-and-gold ornament.
  It stays legible from a 16px launcher entry to the 512px package image, and
  every platform icon (Linux, Windows, macOS, Android, iOS) is regenerated from
  one master.

### Fixed

* The window titlebar now follows the app's dark/light mode where the system
  GTK theme supports both variants, instead of staying on the system theme while
  the content flips.

## 1.1.2

### Fixed

* After **Set up voice** succeeds, the translation voice is enabled again by
  default. The "not ready" status turned the toggle off, and a successful setup
  left it off, so a reader had to enable it by hand before hearing translations.

## 1.1.1

### Added

* **Set up voice** in Settings → Translation voice. The app now prepares the
  Supertonic runtime itself, writing the venv into its own data directory so no
  password is needed, and streams install progress into the panel. A copyable
  command is offered as a fallback when the in-app setup cannot run.
* The status notice now distinguishes a runtime that was never set up (the app
  fell back to the system Python) from one that exists but is broken.

### Fixed

* The Settings notice claimed a Python version problem even when no runtime had
  been set up, and told the reader to "use Python 3.12 or 3.14" without saying
  how. It now names what was actually searched and points at the setup action.
* The packaged setup instructions could not work: `setup.sh` writes its venv
  next to itself, which is root-owned after an install. The script now honours
  `VENV_DIR`, and the app uses it to redirect the venv to a writable path.

## 1.1.0

The speech engine changed: translation and tafsir are now spoken on-device by
**Supertonic 3**, replacing the Coqui TTS engine and its downloaded voice pack.
The Linux packages are rebuilt around the new runtime.

### Added

* On-device **Supertonic 3** speech for translation and tafsir, with CPU and GPU
  setup scripts. The model (~400 MB) is fetched on first synthesis; there is no
  voice-pack download and no fallback engine.
* A storage panel in Settings → Advanced: cache used against the cap, a
  per-kind breakdown (qari, translation, tafsir), and a **Clear cache** button.
* Startup cache maintenance: leftover `.part` files from interrupted renders are
  swept, and least-recently-used media is evicted down to a 2 GB cap
  (`EQURAN_TTS_CACHE_LIMIT_MB`, `0` disables it).
* English tafsir narration from UmmahAPI (*Ma'arif al-Qur'an*).
* Continuous integration for the Rust workspace and the frontend.

### Changed

* English playback now narrates the verse translation before the tafsir, in the
  same order as Indonesian.
* Translation and tafsir narration stream chunk by chunk, so the first sentence
  is heard as soon as it is rendered instead of after the whole verse.
* `clear_cache` and the startup prune refuse to run while a session holds the
  audio device.
* The English tafsir panel reflows the single-line API comment into paragraphs at
  sentence boundaries for reading; the raw text still drives narration.

### Fixed

* A readiness probe that failed to spawn was cached as "the speech package is
  missing", so a momentary hiccup disabled the voice for the rest of the session.
* The tafsir cache is keyed by language, so an Indonesian and an English render
  of the same ayah no longer collide.
* The interface follows the chosen translation language instead of staying
  Indonesian.

### Earlier work

Before this release the desktop app was rebuilt around search, marks and a
session-based player, the TTS runtime was split into focused modules, and the
previous voice-pack installer was replaced by the Supertonic 3 setup scripts. See
the commit history for the details.
