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

### Bahasa Indonesia dan Inggris

Aplikasi menggunakan **Supertonic 3** (ONNX, multilingual, on-device) untuk TTS
Bahasa Indonesia dan Inggris. Tidak ada fallback ke backend lain: jika runtime
tidak siap, TTS dinonaktifkan dengan pesan yang bisa ditindaklanjuti.

#### Setup Runtime

```bash
cd tts
bash setup.sh
```

Model (~400 MB) diunduh otomatis saat sintesis pertama dan disimpan di
`~/.cache/supertonic3/`.

#### Resolusi Runtime

Aplikasi mencari interpreter dan script dengan urutan:

```text
1. EQURAN_TTS_PYTHON (jika diset)
2. tts/.venv/bin/python (jika ada)
3. python3 dari PATH
```

Script engine dicari di `tts/tts_supertonic.py` (bisa dioverride dengan
`EQURAN_TTS_SUPERTONIC`). Voice dipilih lewat `EQURAN_TTS_SUPERTONIC_VOICE`
(default `M2`; pilihan `M1`–`M5`, `F1`–`F5`).

#### Hybrid CPU/GPU

Script memilih execution provider ONNX Runtime saat start-up. Urutannya
`TensorRT → CUDA → ROCm → MIGraphX → DirectML → CoreML → OpenVINO → CPU`;
provider yang tidak terpasang dilewati, dan `CPUExecutionProvider` selalu
ditambahkan di akhir. ONNX Runtime membagi graph ke provider pertama yang bisa
menjalankannya, jadi hasilnya eksekusi hybrid: bagian yang didukung GPU jalan
di GPU, sisanya di CPU. Tanpa build ONNX Runtime berakselerasi, semuanya di CPU.

| Variabel | Default | Fungsi |
|---|---|---|
| `EQURAN_TTS_DEVICE` | `auto` | `auto`/`gpu` pakai GPU bila ada, `cpu` paksa CPU |
| `EQURAN_TTS_THREADS` | `min(cpu/2, 8)` | Thread intra-op; default = jumlah core fisik |
| `EQURAN_TTS_ONNX_PROVIDERS` | — | Daftar provider eksplisit, mis. `CUDAExecutionProvider,CPUExecutionProvider` |
| `EQURAN_TTS_IDLE_TIMEOUT` | `600` | Detik idle sebelum service keluar dan melepas model (`0` = selamanya) |
| `EQURAN_TTS_SUPERTONIC_STEPS` | `5` | Langkah diffusion (8 = default library, lebih lambat) |

Service keluar setelah idle agar ~1 GB RSS dan thread pool tidak menetap saat
aplikasi tidak bersuara; request berikutnya men-spawn ulang service.

#### Pemasangan di perangkat lain

Tidak ada konfigurasi per-device yang perlu ditulis tangan. Semuanya dideteksi
saat runtime:

```text
1. EQURAN_TTS_PYTHON (jika diset)          → dipakai apa adanya
2. <resource>/tts/.venv-gpu/bin/python     → hanya jika bisa import supertonic
3. <resource>/tts/.venv/bin/python         → jika ada
4. python3 dari PATH
```

Untuk mengaktifkan GPU di mesin baru:

```bash
cd <direktori resource>/tts
bash setup-gpu.sh          # deteksi backend, buat .venv-gpu
```

`setup-gpu.sh` mendeteksi backend sendiri: NVIDIA (`nvidia-smi`) →
`onnxruntime-gpu`, `/opt/rocm` → `onnxruntime-rocm`, Windows →
`onnxruntime-directml`, macOS → CoreML bawaan. Jika tidak ada GPU yang
didukung, script keluar dengan pesan jelas dan aplikasi tetap memakai CPU.
Venv GPU yang gagal import `supertonic` diabaikan, jadi TTS tidak pernah mati
karena percobaan GPU yang setengah jadi.

Kesiapan runtime diperiksa dengan `python -c "import supertonic"` dan hasilnya
di-cache per interpreter agar tidak dijalankan berulang kali.

### English

Supertonic 3 multilingual, voice yang sama dengan Bahasa Indonesia.
Teks yang dibacakan untuk `lang = en` mengikuti urutan yang sama dengan bahasa
Indonesia: **terjemahan ayat** dari [equran.id/api/en](https://equran.id/api/en)
(`textEnglish`), lalu **tafsir Inggris** dari
[UmmahAPI](https://ummahapi.com/api/docs) (sumber `maarif`) bila opsi
"Bacakan tafsir" aktif. Tafsir hanya diambil saat opsi itu menyala; tanpa
tafsir, narasi berhenti setelah terjemahan ayat dan tidak jatuh ke teks lain.
Panel tafsir memakai sumber yang sama saat bahasa antarmuka Inggris, sedangkan
bahasa Indonesia tetap memakai `quran-api-id.vercel.app`. UmmahAPI mengirim
komentar sebagai satu string tanpa baris baru, jadi panel menyusun ulang
paragraf di batas kalimat untuk tampilan (`frontend/src/lib/tafsir.ts`); teks
mentah tetap dipakai untuk narasi TTS.

Endpoint tafsir UmmahAPI bisa dipakai tanpa kunci. Untuk kuota/atribusi
pribadi, set `UMMAH_API_KEY` di environment aplikasi; nilainya tidak pernah
disimpan di repository.

Cache TTS narasi memakai nama per chunk: `translation_supertonic_en_*` untuk
terjemahan yang dibacakan dan `tafsir_supertonic_en_*` untuk fase tafsir. Teks
Inggris bisa jauh melebihi 320 karakter, jadi tidak pernah dikirim sebagai satu
permintaan sintesis (permintaan sebesar itu melampaui timeout worker dan
memblokir permintaan berikutnya di balik mutex service tunggal).

## Cache System

### Cache Directory Structure

```text
~/.cache/equran-cli/
├── audio/          # MP3 qari dari EQuran.id
└── tts/            # TTS output
    ├── supertonic_id_001_001.wav
    ├── translation_supertonic_en_001_001_001.wav
    ├── tafsir_supertonic_id_001_001_001.wav
    └── tafsir_supertonic_en_001_001_001.wav
```

File TTS ditulis ke file `.part` unik lalu di-rename secara atomik, sehingga
sintesis yang gagal atau timeout tidak meninggalkan cache yang rusak. Cache
dianggap valid hanya jika ukurannya lebih dari 44 byte (header WAV).

### Pruning dan Batas Ukuran

Narasi adalah satu-satunya bagian yang tumbuh tanpa batas (WAV PCM ~88 KB per
detik, tafsir satu ayat bisa puluhan MB), jadi media dijaga di dalam batas
ukuran bukannya dibiarkan mengisi disk. Saat aplikasi desktop start,
`AudioCache::prune` berjalan di worker (tidak menahan pembukaan jendela):

1. **Sweep `.part` yatim** — file `.part` yang lebih tua dari 10 menit
   (`ORPHAN_PART_MAX_AGE`) dihapus; request synthesis sendiri timeout di 300 s,
   jadi tidak ada render hidup yang bisa memilikinya.
2. **Evict LRU ke batas** — bila total media melewati cap, file dihapus dari
   mtime terlama sampai di bawah cap. Cache menggunakan file terbaru sebagai
   working set dan menyintesis ulang yang lama saat dibutuhkan.

Cap default 2 GB, diatur lewat `EQURAN_TTS_CACHE_LIMIT_MB` (`0` = tanpa cap).
Index pencarian (`index/quran-index.json`) tidak ikut dihitung maupun dihapus:
ia punya siklus hidup sendiri. Operasi destruktif (prune, clear) hanya berjalan
saat tidak ada sesi aktif (`PlaybackState::is_active`), agar tidak menghapus
file yang sedang dibaca audio thread.

### Storage di UI

Settings → Advanced menampilkan ledger penyimpanan: total terpakai vs cap, jumlah
berkas, rincian per jenis (Qari, Terjemahan, Tafsir), plus tombol **Kosongkan
cache**. Angka diambil dari command `get_cache_usage`; tombol memanggil
`clear_cache` yang menolak berjalan selagi playback aktif.

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

### Runtime Distribution

Paket desktop menyertakan file setup ringan (`tts/setup.sh`,
`tts/requirements.txt`, `tts/tts_supertonic.py`). Runtime Python dan model
Supertonic tidak dibundel; keduanya disiapkan sekali lewat `tts/setup.sh`, dan
model diunduh otomatis oleh SDK Supertonic saat sintesis pertama.

Tidak ada lagi voice-pack archive, manifest, atau flow ekstraksi `tar.zst`.
Status TTS dihitung dari ketersediaan script engine dan hasil
`python -c "import supertonic"`.

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
- WAV PCM 16-bit hasil Supertonic 3

Tidak perlu menginstall `mpv`, `ffplay`, `paplay`, atau `aplay` untuk playback.

Playback adalah **satu sesi**, bukan satu ayat. Sesi memegang surah, ayat awal,
ayat akhir, dan posisi sekarang; bar, penanda ayat aktif, serta tombol
sebelum/berikutnya membaca sesi itu, bukan surah yang sedang dibuka. Karena
itu menjelajah mushaf saat audio berjalan tidak mengubah judul bar atau
memindahkan sesi.

- **Dengarkan** di header Baca memutar surah dari ayat 1 sampai akhir; tombol
  yang sama menjadi Jeda/Lanjutkan untuk sesi surah itu.
- Tombol play di kartu ayat memulai antrean **dari ayat itu sampai akhir
  surah**, bukan satu ayat. `from = to` hanya dipakai untuk satu ayat bila
  memang diminta.
- `pause_playback`/`resume_playback` hanya men-set flag pada `PlaybackControl`
  yang sama dengan yang dipakai thread audio (rekaman, terjemahan, dan tafsir),
  jadi jeda melanjutkan dari sample yang sama. `stop_playback` membatalkan
  token dan mengakhiri sesi.
- Selesai bukan `idle`: fase `finished` menahan bar di layar dengan status
  "Surah selesai" dan tombol putar ulang. Bar baru hilang lewat tombol tutup
  atau saat sesi baru dimulai.
- Sebelum/berikutnya tidak melingkar; di ayat 1 dan ayat terakhir tombolnya
  tidak melakukan apa-apa. `repeat` mengulang dari ayat awal sesi, bukan selalu
  dari 1.
- Kegagalan pemutaran tampil di bar (dengan tombol Coba lagi), bukan di panel
  Pengaturan. Runtime TTS yang belum siap hanya melewati fase terjemahan/tafsir
  dan mengirim `playback-notice`; bacaan qari tetap diputar.

### Tafsir Voice

Tombol Voice pada panel tafsir memutar narasi per chunk (320 karakter) dengan
prefetch 3 chunk ke depan. Kontrolnya:

- **Voice / Berhenti** — mulai atau hentikan total.
- **Jeda / Lanjut** — muncul selama narasi aktif. `pause_tafsir_voice` dan
  `resume_tafsir_voice` hanya men-set flag; thread audio mem-polling flag itu
  setiap 25 ms dan memanggil `Sink::pause()`/`Sink::play()`. Karena `Sink`
  tetap hidup, resume melanjutkan dari sample yang sama, bukan mengulang chunk.
- Status `is_tafsir_voice_active` dan `is_tafsir_voice_paused` di-polling UI
  setiap 500 ms, karena tombol berada di komponen yang berbeda dari kontrol
  playback utama.

### Read Tafsir (Playback Otomatis)

Toggle **Read Tafsir** di Playback Settings membuat playback ayat berurutan:
qari → terjemahan → tafsir, lalu lanjut ke ayat berikutnya. Implementasinya di
`PlaybackEngine::play_ayah`:

1. Tafsir surah di-fetch sekali saat `start_playback` (`fetch_tafsir`) dan
   ditempel ke `Ayat::tafsir`. Kegagalan fetch tidak fatal — narasi tafsir
   dilewati, playback tetap jalan.
2. Teks narasi dipotong per 320 karakter (`TtsEngine::split_narration_text`).
   Chunk disintesis lewat channel sambil diputar (`tokio::sync::mpsc`), jadi
   kalimat pertama berbunyi begitu chunk pertama siap, bukan setelah seluruh
   tafsir selesai dirender. Untuk `lang = en`, terjemahan yang dibacakan adalah
   tafsir itu sendiri, sehingga fase tafsir tidak diulang.
3. Setelah terjemahan, engine meng-emit `PlaybackPhase::Tafsir` lalu memutar
   chunk berurutan. Satu chunk yang gagal hanya melewatkan kalimat itu; sisa
   narasi dan sesi tetap berjalan.

Cache memakai nama yang sama dengan tombol Voice (`tafsir_supertonic_id_*`),
jadi audio yang sudah pernah didengar lewat tombol Voice tidak disintesis ulang.

### Home Page dan Pencarian Lokal

Desktop punya dua surface: **Beranda** dan **Baca**. State-nya di
`frontend/src/lib/stores/view.svelte.ts` (`home` | `reader`), dipersist ke
`localStorage`. Beranda berisi pencarian global, chip topik, lanjut-membaca,
tanda terakhir, dan hitungan nyata.

Shell-nya tiga kolom: rel navigasi kiri (Beranda/Baca/Tanda), kolom konten di
tengah, dan **rel tanda** di kanan (`MarksRail.svelte`) untuk ayat milik
pembaca. Tidak ada kolom pengaturan permanen — pengaturan adalah dialog.
Rel tanda bisa dibuka lewat baris **Tanda** di kiri, tombol bookmark di header
Baca, atau `Cmd/Ctrl+M`; statusnya (`panelState.marksOpen`) dipersist bareng
`sidebarOpen`. Di jendela sempit (<1180px) rel kanan keluar dari grid dan
mengapung di atas kolom baca agar kolom baca tidak mengecil.

Rel kiri bisa **dicuitkan** (collapsed) jadi kolom ikon 64px, bukan hilang:
`panelState.sidebarOpen` menyetel lebar track (`280px` ↔ `64px`). Dalam mode
ini label nav disembunyikan (tooltip menggantikan), badge jumlah tanda pindah
ke atas ikon, dan footer (Pengaturan, tema, tombol ciutkan) menumpuk vertikal.
Tombol ciutkan/bentangkan hanya ada di footer rail itu sendiri supaya selalu
terjangkau dari Beranda maupun Baca; `Cmd/Ctrl+B` tetap jalan.

Baris nav yang sedang aktif adalah **pil aksen** (`--accent`) di dalam rail:
latar `--accent`, teks `--on-accent`, meta `--on-accent-muted`, radius
`--r-control`. Pil ini berada di dalam inset nav, tidak menyentuh tepi rail —
rail tetap kolom yang tenang dan hanya baris itu sendiri yang membawa warna.
Baris non-aktif `hover` memakai `--surface`.

> Riwayat: beberapa eksperimen bentuk (tab menyatu kolom baca, inverted
> border-radius, flare "curved border" dengan `::before`/`::after`) sudah
> dibatalkan. Bentuk akhirnya pil aksen sederhana seperti di atas; tidak ada sisa
> `--nav-curve`, `::before`/`::after` nav, `radial-gradient`, atau token
> `--hover`.

Perubahan lebar itu **ditween** (bukan snap) supaya transisi terasa tenang:
track grid, kolom ikon/label tiap baris, dan padding footer semuanya memakai
`--speed`/`--ease` yang sama, sementara label memudar lewat `opacity` (bukan
`display`, yang tak bisa dianimasikan) dan footer menumpuk lewat `flex-wrap`
ketika lebar tak lagi cukup. Namun menween lebar berarti reflow tiap kartu
ayat per frame, jadi di surah berat transit──nya **di-snap**: `toggleSidebar`
menghitung `[data-ayah]` dan melewati tween di atas
`MAX_VERSES_FOR_RAIL_TWEEN` (80, sama ambangnya dengan reveal tema di
`theme.svelte.ts`), menambahkan kelas `.rail-snapping` selama satu task.
Terukur di build produksi: Al-Fatihah (7 ayat) lewat 11 langkah layout, mulus;
Al-Baqarah (286 ayat) 4 langkah, terlihat tersendat — karenanya di-snap.
`prefers-reduced-motion` tetap mematikan semua transisi lewat aturan global.

Indeks pencarian dibangun lokal di `crates/equran-core/src/search/`:

- **Normalisasi** — `normalize_arabic` membuang harakat (U+064B..U+0652),
  U+0670, U+06D6..U+06ED dan tatweel, lalu menyatukan varian huruf
  (`أ إ آ ٱ → ا`, `ى → ي`, `ة → ه`, `ؤ → و`, `ئ → ي`). `normalize_latin`
  me-lowercase, melepas combining mark (U+0300..U+036F) dan memetakan huruf
  beraksen, sehingga `aḥmad` = `ahmad` dan `Qur'an` = `quran`.
- **Sumber** — 114 surah Indonesia, 114 tafsir, dan 114 surah Inggris,
  digabung per (surah, ayat). Maksimum 4 request paralel. Kegagalan
  tafsir/Inggris tidak fatal; field-nya dibiarkan kosong.
- **Data** — ±6.236 ayat, ±26 MB JSON mentah. Indeks dibangun ~7,5 detik
  dan disimpan di `<cache_dir>/equran-cli/index/quran-index.json`.
  Penyimpanan atomik (`.part` lalu `rename`), dengan `version` untuk
  invalidasi.
- **Pencarian** — BM25 (k1=1,2, b=0,75) dengan bobot field: arab 1,0,
  latin 0,9, indonesia 1,0, inggris 0,8, **tafsir 1,2** (tafsir adalah
  sumber terbaik untuk pencarian topik). Semua token harus cocok (AND).
  Token yang lebih pendek dari term juga dicocokkan sebagai prefix, jadi
  `bismillah` menemukan `bismillāhir` dan `rahman` menemukan `raḥmān`.
- **Performa** — inverted index dibangun sekali per indeks dan disimpan di
  `OnceLock`, bukan dibangun ulang setiap query. Pencarian di-debounce 260 ms.

Indeks **tidak** dibangun saat startup. Build terjadi saat query pertama
(atau `rebuild_index`), dengan progress event `index-progress`
(`{ done, total }`) yang ditampilkan di Beranda.

Command Tauri: `search_quran`, `get_index_status`, `rebuild_index`.

Panel **Tanya tentang ayat** (asisten generatif) sengaja belum dibangun.
Tempatnya sudah disiapkan di Beranda. Rencananya retrieval + sitasi
`[2:153]` dengan provider OpenAI-compatible dan API key di OS keyring,
bukan file settings.

### Bahasa Antarmuka

UI mengikuti toggle **Language (Translation)**: `id` atau `en`. Kamus ada di
`frontend/src/lib/i18n.ts` (`t('key')` dengan placeholder `%s`). Key yang
hilang jatuh ke Bahasa Indonesia lalu ke key itu sendiri, supaya string yang
terlupa terlihat dan bukan kosong.

### Sistem Warna

`frontend/src/app.css` memakai token CSS (`--page`, `--surface`, `--raised`,
`--text`, `--accent`, `--mark`, ...) yang di-swap oleh class `.dark`. Tidak ada
lagi override `!important` per kelas Tailwind. Label teks punya tiga peran
(`--text`, `--text-secondary`, `--text-muted`) dan satu accent yang dipakai
hemat; `--mark` (rust) khusus penanda milik pembaca (bookmark).

Kontrol geser dan progres (toggle, progress bar, cache bar) memakai pasangan
token sendiri: `--track` (dasar lintasan) dan `--knob` (tombol toggle). Di tema
terang `--knob` adalah putih dan dipisahkan dari lintasan oleh `--shadow-card`.
Tema gelap tidak punya shadow (`--shadow-card: none`), jadi di sana `--track`
adalah satu langkah lebih terang dari `--surface` dan `--knob` adalah cakram
terang (`#d7e2ee`) — tanpa ini tombol dan lintasan berwarna sama dan switch mati
terbaca sebagai kotak kosong.

Nilai tema gelap dijaga agar semua peran teks melewati WCAG AA (4.5:1) pada
permukaan tempat mereka benar-benar muncul. `--text-muted` di tema gelap harus
terbaca di atas `--raised` (kartu ayat dan rel Tanda), bukan hanya di atas
`--page`, jadi nilainya `#8a9cae`; `--mark` `#d78d72`; dan
`--on-accent-muted` di tema gelap dipekatkan ke `rgba(14, 22, 32, 0.85)` supaya
meta pada baris nav aktif tetap terbaca di atas `--accent`.

Bentuk khas produk ini adalah **rosette** oktagram (`VerseRosette.svelte`),
satu bentuk di tiga ukuran: kecil di rel tanda dan pencari surah, sedang di
kartu ayat, besar sebagai watermark Beranda. Ikon memakai satu set
(`Icon.svelte`, 24-viewBox, stroke 1,5) — tidak ada emoji di antarmuka.

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

MIT — lihat [`LICENSE`](LICENSE). Teks Qur'an, rekaman qari, dan tafsir dimiliki
oleh sumbernya masing-masing (lihat bagian *Data sources and attribution* di
`README.md`); lisensi ini mencakup kode, bukan konten tersebut.
