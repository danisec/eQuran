# EQuran CLI

Rust CLI untuk memutar ayat Al-Qur'an dari EQuran.id dengan pilihan 6 qari dan voice terjemahan Indonesia atau English.

## Fitur

- Daftar 114 surah dari `https://equran.id/api/v2/surat`
- Audio Arab per ayat dari 6 qari EQuran.id
- Terjemahan Indonesia dari EQuran API v2
- Terjemahan English dari `https://equran.id/api/en/surah/{number}`
- Mode otomatis per surah: ayat Arab lalu voice arti, berulang sampai akhir surah
- Cache MP3 dan WAV di `~/.cache/equran-cli/`
- `--dry-run` untuk melihat teks tanpa audio/TTS

## Build

```bash
cargo build --release
cargo build --release --features audio
```

Feature `audio` memakai backend native Rust (`rodio`/`cpal`) untuk MP3 dan WAV. Di Linux, build native audio membutuhkan ALSA development headers, tetapi runtime playback tidak lagi membutuhkan player eksternal seperti `mpv`.

## Contoh penggunaan

```bash
cargo run -- list
cargo run -- info --surah 1
cargo run -- play --surah 1 --qari misyari --lang id --dry-run
cargo run -- play --surah 1 --ayat 1 --qari sudais --lang en --dry-run
cargo run -- play --surah 2 --from-ayat 1 --to-ayat 3 --qari dossari --lang id --dry-run
cargo run --features audio -- play --surah 1 --qari dossari --lang id --no-prefetch
cargo run --features audio -- play --surah 1 --qari dossari --lang id --pregenerate-tts
```

Untuk playback sungguhan tanpa `--dry-run`, build dengan feature `audio` dan siapkan salah satu engine TTS. Untuk bahasa Indonesia, aplikasi memakai TTS Wibowo jika sudah disiapkan, lalu fallback ke `espeak-ng`.

```bash
export EQURAN_TTS_MODEL_ID=/path/to/id_ID-model.onnx
export EQURAN_TTS_MODEL_EN=/path/to/en_US-model.onnx
cargo run --features audio -- play --surah 1 --qari misyari --lang id
```

## Qari

| Shortname | Qari | Key |
| --- | --- | --- |
| juhany | Abdullah Al-Juhany | 01 |
| qasim | Abdul Muhsin Al-Qasim | 02 |
| sudais | Abdurrahman As-Sudais | 03 |
| dossari | Ibrahim Al-Dossari | 04 |
| misyari | Misyari Rasyid Al-Afasy | 05 |
| yasser | Yasser Al-Dosari | 06 |

## Fedora dependencies

```bash
sudo dnf install rust cargo alsa-lib-devel espeak-ng python3 python3-pip
```

## TTS Wibowo untuk Bahasa Indonesia

Untuk suara terjemahan Indonesia yang lebih natural, siapkan backend Wibowo:

```bash
cd /home/dani/equran-cli/tts
bash setup.sh
```

Jika package Coqui TTS belum mendukung Python bawaan sistem, gunakan Python 3.10/3.11:

```bash
cd /home/dani/equran-cli/tts
PYTHON_BIN=python3.11 bash setup.sh
```

Setelah setup, jalankan dari root project:

```bash
cd /home/dani/equran-cli
cargo run --features audio -- play --surah 1 --qari dossari --lang id
```

Aplikasi otomatis mencari:

```text
tts/.venv/bin/python
tts/tts_wibowo.py
```

Jika lokasi berbeda, override dengan:

```bash
export EQURAN_TTS_PYTHON=/path/to/python
export EQURAN_TTS_WIBOWO=/path/to/tts_wibowo.py
```

Urutan backend TTS Indonesia:

1. TTS Wibowo jika `EQURAN_TTS_PYTHON` + `EQURAN_TTS_WIBOWO` tersedia, atau default `tts/.venv/bin/python` + `tts/tts_wibowo.py` ditemukan.
2. `espeak-ng` sebagai fallback terakhir.

Cache TTS menyertakan nama backend agar file lama tidak tertukar:

```text
~/.cache/equran-cli/tts/wibowo_id_001_001.wav
~/.cache/equran-cli/tts/espeak_id_001_001.wav
```

Secara default, playback langsung mulai dan TTS ayat berikutnya di-prefetch di background. Gunakan `--no-prefetch` untuk mematikan background prefetch. Jika ingin menyiapkan seluruh TTS sebelum playback, gunakan `--pregenerate-tts`.

Untuk memutar sebagian surah, gunakan `--from-ayat` dan `--to-ayat`. Contoh Surah Al-Baqarah ayat 1 sampai 3:

```bash
cargo run --features audio -- play --surah 2 --from-ayat 1 --to-ayat 3 --qari dossari --lang id
```

Flag `--ayat` tetap tersedia untuk satu ayat saja dan tidak bisa digabung dengan range.

Untuk English, gunakan Edge-TTS voice `en-US-ChristopherNeural` atau fallback `espeak-ng`.

Runtime playback memakai backend native Rust dan mendukung file cache MP3 qari, MP3 Edge-TTS, serta WAV dari Wibowo/`espeak-ng`. Tidak perlu menginstall `mpv`, `ffplay`, `paplay`, atau `aplay` untuk playback aplikasi.

Playback audio memakai feature Rust `audio`:

```bash
cargo build --release --features audio
```

Model Wibowo diunduh otomatis oleh wrapper saat pertama kali dipakai dan akan di-cache oleh package `g2p_id`.

## RPM

Spec file awal tersedia di `packaging/equran-cli.spec`.

Build manual dari source release tarball:

```bash
rpmbuild -ba packaging/equran-cli.spec
```
