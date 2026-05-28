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

### Desktop Application (with GUI)

The eQuran desktop application includes a graphical interface and comes with **Natural Indonesian Voice TTS** pre-configured for high-quality translation audio.

#### RPM-based Linux (Fedora, RHEL, CentOS, etc.)

Download and install the RPM package:

```bash
# Download from GitHub Releases
wget https://github.com/danisec/eQuran/releases/download/voice-v1/eQuran-1.0.0-1.x86_64.rpm

# Install using dnf (Fedora/RHEL 8+)
sudo dnf install ./eQuran-1.0.0-1.x86_64.rpm

# Or using rpm
sudo rpm -ivh eQuran-1.0.0-1.x86_64.rpm
```

Launch from your application menu or run `equran-desktop` in terminal.

#### DEB-based Linux (Debian, Ubuntu, Linux Mint, etc.)

Download and install the DEB package:

```bash
# Download from GitHub Releases
wget https://github.com/danisec/eQuran/releases/download/voice-v1/eQuran_1.0.0_amd64.deb

# Install using apt
sudo apt install ./eQuran_1.0.0_amd64.deb

# Or using dpkg
sudo dpkg -i eQuran_1.0.0_amd64.deb

# If there are dependency issues, run:
sudo apt-get install -f
```

Launch from your application menu or run `equran-desktop` in terminal.

### CLI Version (from source)

For command-line only usage, build from source:

```bash
git clone https://github.com/danisec/eQuran.git
cd equran-cli
cargo build --release --features audio
sudo cp target/release/equran-cli /usr/local/bin/
```

For Natural Indonesian Voice TTS support with CLI version, run setup:

```bash
cd tts
bash setup.sh
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

- Arabic Audio, Tafsir: [EQuran.id](https://equran.id)
- TTS Indonesia: [TTS Indonesia Gratis](https://github.com/drat/TTS-Indonesia-Gratis)

---
