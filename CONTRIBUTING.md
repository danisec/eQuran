# Contributing

Thanks for taking the time. This is a small project, so the fastest way to get a
change merged is to keep it focused and tested.

## Getting set up

```bash
git clone https://github.com/danisec/eQuran.git
cd eQuran
bash tts/setup.sh                 # optional: enables the translation voice
cd frontend && npm ci && cd ..    # desktop frontend
```

Build and run:

```bash
cargo build --release --features audio      # command-line player
cd crates/equran-desktop && cargo tauri dev # desktop app
```

The Tauri config expects to be run from `crates/equran-desktop`; it points at the
frontend with a relative path.

## Before you open a pull request

Run the same checks CI runs:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features

cd frontend
npm run check
npm test
```

All six must pass. `cargo clippy` is denied at warning level, so a stray
`collapsible_if` is a failure, not a suggestion.

## What a good change looks like

* **A change needs a test.** A bug fix needs a test that fails without the fix; a
  new behavior needs a test that covers it. The existing suites are the pattern to
  follow.
* **Small and focused.** One concern per pull request. A refactor bundled with a
  fix is two pull requests.
* **Match the surrounding style.** Comments explain *why* a thing is the way it
  is, not *what* the line does. The codebase is deliberate about this; a comment
  that restates the code will be asked to change.
* **No new runtime dependencies without a reason** in the pull request. The
  dependency surface is kept small on purpose.
* **Keep the UI in its system.** The frontend has one design system — a quiet
  neutral ramp, one accent, one icon set, no emoji. New surfaces should look like
  they were always there. Spacing is on a 4px grid.

## Architecture notes

`DEVELOPER.md` documents the moving parts: the playback session model, the TTS
runtime resolution, the cache layout and pruning, the search index, and the
colour system. Read the relevant section before changing one of those areas.

The workspace has three crates:

| Crate | What it is |
|---|---|
| `crates/equran-core` | Library: API client, playback engine, TTS, cache, search index |
| `crates/equran-desktop` | Tauri app and its commands |
| `.` (`equran-cli`) | Command-line player |

Keeping logic in `equran-core` means the CLI and the desktop app share it; put
behavior there unless it is genuinely platform-specific.

## Reporting bugs

Open an issue with what you did, what you expected, and what happened. For
anything that looks security-sensitive, see [`SECURITY.md`](SECURITY.md) instead
of opening a public issue.

## Licence

There is no CLA. By contributing you agree your work is licensed under the MIT
licence in [`LICENSE`](LICENSE).
