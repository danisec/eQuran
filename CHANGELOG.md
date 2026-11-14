# Changelog

Notable changes to eQuran. This project follows [semantic
versioning](https://semver.org) for its public interfaces once they settle;
until 1.0 is tagged on `master`, treat the CLI flags and the settings file as the
things most worth watching.

## Unreleased

### Added

* A storage panel in Settings → Advanced: cache used against the cap, a
  per-kind breakdown (qari, translation, tafsir), and a **Clear cache** button.
* Startup cache maintenance: leftover `.part` files from interrupted renders are
  swept, and least-recently-used media is evicted down to a 2 GB cap
  (`EQURAN_TTS_CACHE_LIMIT_MB`, `0` disables it).
* English tafsir narration from UmmahAPI (*Ma'arif al-Qur'an*).

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

* The tafsir cache is keyed by language, so an Indonesian and an English render
  of the same ayah no longer collide.
* The interface follows the chosen translation language instead of staying
  Indonesian.

## Earlier work

Before the cache and English-tafsir work, the desktop app was rebuilt around
search, marks and a session-based player, the TTS runtime was split into focused
modules, and the previous voice-pack installer was replaced by the Supertonic 3
setup scripts. See the commit history for the details.
