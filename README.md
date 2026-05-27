# EQuran CLI

A command-line application for listening to the Holy Quran with high-quality Arabic recitations from 6 renowned reciters and voice translations in Indonesian or English.

## What Can It Do?

- **Listen to All 114 Surahs** with high-quality Arabic recitations
- **Choose Your Favorite Reciter** from 6 renowned reciters
- **Voice Translations** in Indonesian or English
- **Play by Verse or Verse Range** as needed
- **Automatic Mode** - Arabic recitation followed by translation, repeating until the end of the surah
- **Offline-Ready** - Audio is automatically cached for offline playback

## Available Reciters

Choose from one of 6 reciters:

| Reciter Name | Code |
|--------------|------|
| Abdullah Al-Juhany | `juhany` |
| Abdul Muhsin Al-Qasim | `qasim` |
| Abdurrahman As-Sudais | `sudais` |
| Ibrahim Al-Dossari | `dossari` |
| Misyari Rasyid Al-Afasy | `misyari` |
| Yasser Al-Dosari | `yasser` |

## How to Use

### List All Surahs

```bash
equran-cli list
```

### View Surah Information

```bash
equran-cli info --surah 1
```

### Play Complete Surah

Play Surah Al-Fatihah with reciter Misyari and Indonesian translation:

```bash
equran-cli play --surah 1 --qari misyari --lang id
```

Play with English translation:

```bash
equran-cli play --surah 1 --qari sudais --lang en
```

### Play Single Verse

Play the first verse of Surah Al-Fatihah:

```bash
equran-cli play --surah 1 --ayat 1 --qari dossari --lang id
```

### Play Verse Range

Play Surah Al-Baqarah verses 1 to 5:

```bash
equran-cli play --surah 2 --from-ayat 1 --to-ayat 5 --qari misyari --lang id
```

### View Text Without Audio

Use `--dry-run` to view Arabic text and translation without playing audio:

```bash
equran-cli play --surah 1 --qari misyari --lang id --dry-run
```

## Installation

### Linux (Fedora/RHEL)

```bash
sudo dnf install equran-cli
```

### From Source

Requires Rust toolchain:

```bash
git clone https://github.com/OWNER/equran-cli
cd equran-cli
cargo build --release --features audio
sudo cp target/release/equran-cli /usr/local/bin/
```

## Translation Voice Quality

### Indonesian

The application supports two voice engines:

1. **Natural Indonesian Voice** (Recommended) - High-quality natural voice
2. **Edge-TTS** - Standard voice from Microsoft Edge

To use Natural Indonesian Voice, run setup once:

```bash
cd tts
bash setup.sh
```

If not set up, the application automatically uses Edge-TTS.

### English

Uses Edge-TTS voice `en-US-ChristopherNeural`.

## Advanced Options

### Prepare Audio Before Playback

By default, audio plays immediately and the next verse is prepared in the background. To prepare all audio before starting:

```bash
equran-cli play --surah 1 --qari dossari --lang id --pregenerate-tts
```

### Disable Background Prefetch

```bash
equran-cli play --surah 1 --qari dossari --lang id --no-prefetch
```

## Cache Location

Downloaded audio is stored at:

```
~/.cache/equran-cli/
```

Cached files enable offline playback without re-downloading.

## Help

To view all available options:

```bash
equran-cli --help
equran-cli play --help
```

## Data Sources

- Arabic Audio: [EQuran.id](https://equran.id)
- Translations: EQuran API v2

---

**Technical Note for Developers**: Build documentation, dependencies, and advanced configuration are available in [DEVELOPER.md](DEVELOPER.md)
