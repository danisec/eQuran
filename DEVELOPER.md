# Developer Documentation

Dokumentasi teknis untuk developer yang ingin build, modify, atau contribute ke EQuran CLI.

## Build Requirements

### Linux (Fedora/RHEL)

```bash
sudo dnf install rust cargo alsa-lib-devel python3 python3-pip
```

### Build Commands

```bash
# Build tanpa audio (text-only mode)
cargo build --release

# Build dengan audio playback
cargo build --release --features audio
```

Feature `audio` menggunakan backend native Rust (`rodio`/`cpal`) untuk MP3 dan WAV playback. Di Linux, build membutuhkan ALSA development headers, tetapi runtime tidak membutuhkan player eksternal seperti `mpv`.

## Development Workflow

### Running from Source

```bash
# List surah
cargo run -- list

# Info surah
cargo run -- info --surah 1

# Dry-run (text only, no audio)
cargo run -- play --surah 1 --qari misyari --lang id --dry-run

# Play single ayat
cargo run -- play --surah 1 --ayat 1 --qari sudais --lang en --dry-run

# Play range
cargo run -- play --surah 2 --from-ayat 1 --to-ayat 3 --qari dossari --lang id --dry-run

# With audio (requires --features audio)
cargo run --features audio -- play --surah 1 --qari dossari --lang id --no-prefetch
cargo run --features audio -- play --surah 1 --qari dossari --lang id --pregenerate-tts
```

## TTS Backend Configuration

### Bahasa Indonesia

Aplikasi mendukung dua TTS backend untuk Bahasa Indonesia:

1. **Natural Indonesian Voice** (Coqui TTS + model Wibowo)
2. **Edge-TTS** (`id-ID-ArdiNeural`)

#### Setup Natural Indonesian Voice

```bash
cd /home/dani/equran-cli/tts
bash setup.sh
```

Jika Python sistem tidak kompatibel dengan Coqui TTS, gunakan Python 3.10/3.11:

```bash
cd /home/dani/equran-cli/tts
PYTHON_BIN=python3.11 bash setup.sh
```

#### Backend Detection Logic

Aplikasi otomatis mencari Natural Indonesian Voice di:

```text
tts/.venv/bin/python
tts/tts_wibowo.py
```

Override lokasi dengan environment variables:

```bash
export EQURAN_TTS_PYTHON=/path/to/python
export EQURAN_TTS_WIBOWO=/path/to/tts_wibowo.py
```

#### Fallback Order

1. Natural Indonesian Voice jika `EQURAN_TTS_PYTHON` + `EQURAN_TTS_WIBOWO` tersedia, atau default `tts/.venv/bin/python` + `tts/tts_wibowo.py` ditemukan
2. Edge-TTS voice `id-ID-ArdiNeural` jika command `edge-tts` tersedia
3. Jika keduanya tidak tersedia, playback terjemahan TTS akan gagal dengan pesan konfigurasi backend

#### Installing Edge-TTS

```bash
python3 -m pip install edge-tts
```

### English

Menggunakan Edge-TTS voice `en-US-ChristopherNeural`.

## Cache System

### Cache Directory Structure

```text
~/.cache/equran-cli/
├── audio/          # MP3 qari dari EQuran.id
└── tts/            # TTS output
    ├── wibowo_id_001_001.wav    # Natural Indonesian Voice
    └── edge_id_001_001.mp3      # Edge-TTS
```

Cache TTS menyertakan nama backend di filename agar file dari backend berbeda tidak tertukar.

### Prefetch Behavior

Secara default, playback langsung mulai dan TTS ayat berikutnya di-prefetch di background untuk seamless playback.

**Disable prefetch:**
```bash
cargo run --features audio -- play --surah 1 --qari dossari --lang id --no-prefetch
```

**Pregenerate all TTS before playback:**
```bash
cargo run --features audio -- play --surah 1 --qari dossari --lang id --pregenerate-tts
```

## API Endpoints

### Surah List
```
GET https://equran.id/api/v2/surat
```

### Surah Detail (Indonesian)
```
GET https://equran.id/api/v2/surat/{number}
```

### Surah Detail (English)
```
GET https://equran.id/api/en/surah/{number}
```

### Audio URL Pattern
```
https://equran.id/storage/audio/full/{qari_key}/{surah:03d}{ayat:03d}.mp3
```

Qari keys:
- `01` - Abdullah Al-Juhany
- `02` - Abdul Muhsin Al-Qasim
- `03` - Abdurrahman As-Sudais
- `04` - Ibrahim Al-Dossari
- `05` - Misyari Rasyid Al-Afasy
- `06` - Yasser Al-Dosari

## Desktop Application

### Voice Pack Distribution

Di aplikasi desktop, TTS Translation tidak bisa diaktifkan sebelum Natural Indonesian Voice selesai disiapkan. Paket desktop utama hanya menyertakan file setup ringan; runtime dan model suara disiapkan lewat flow download agar DEB/RPM/AppImage tidak membundel seluruh `.venv`.

#### Voice Pack Archive Structure

Archive harus diekstrak menjadi struktur berikut di app data directory pengguna:

```text
tts/
  .venv/bin/python
  .venv/bin/tts
  tts_wibowo.py
  requirements.txt
```

#### Manifest Format

Buat manifest JSON dan simpan sebagai `tts/voice-pack-manifest.json` di package, atau layani dari URL dan set environment variable `EQURAN_NATURAL_INDONESIAN_MANIFEST_URL`:

**Single-part manifest:**
```json
{
  "name": "Natural Indonesian Voice",
  "version": "1.0.0",
  "platform": "linux-x86_64",
  "url": "https://github.com/OWNER/equran-cli/releases/download/voice-v1/equran-natural-indonesian-voice-linux-x86_64-v1.tar.zst",
  "sha256": "PUT_SHA256_HERE",
  "size": 1234567890
}
```

**Multi-part manifest** (untuk archive >2GB):
```json
{
  "name": "Natural Indonesian Voice",
  "version": "1.0.0",
  "platform": "linux-x86_64",
  "parts": [
    {
      "url": "https://github.com/OWNER/equran-cli/releases/download/voice-v1/equran-natural-indonesian-voice-linux-x86_64-v1.tar.zst.part-aa",
      "sha256": "PART_AA_SHA256",
      "size": 1900000000
    },
    {
      "url": "https://github.com/OWNER/equran-cli/releases/download/voice-v1/equran-natural-indonesian-voice-linux-x86_64-v1.tar.zst.part-ab",
      "sha256": "PART_AB_SHA256",
      "size": 1171588095
    }
  ],
  "sha256": "FULL_ARCHIVE_SHA256",
  "size": 3071588095
}
```

#### Download Flow

Saat pengguna menekan Download:
1. Aplikasi mengambil manifest
2. Mengunduh archive (single atau multi-part)
3. Memverifikasi `sha256` jika tersedia
4. Mengekstrak archive ke app data directory
5. Mengaktifkan TTS Translation jika `tts/.venv/bin/python` dan `tts/tts_wibowo.py` valid

Jika manifest tidak tersedia, aplikasi tetap bisa fallback ke `tts/setup.sh` untuk mode developer/source install.

## Packaging

### RPM

Spec file tersedia di `packaging/equran-cli.spec`.

Build manual dari source release tarball:

```bash
rpmbuild -ba packaging/equran-cli.spec
```

## Architecture Notes

### Audio Playback

Runtime playback menggunakan backend native Rust dan mendukung:
- MP3 qari cache
- MP3 Edge-TTS output
- WAV Natural Indonesian Voice output

Tidak perlu menginstall `mpv`, `ffplay`, `paplay`, atau `aplay` untuk playback.

### Feature Flags

- `audio` - Enable native audio playback via `rodio`/`cpal`

Build tanpa feature `audio` hanya mendukung `--dry-run` mode (text-only).

## Contributing

Untuk contribute:
1. Fork repository
2. Create feature branch
3. Test dengan `cargo test`
4. Build dengan `cargo build --release --features audio`
5. Submit pull request

## License

[Add license information here]
