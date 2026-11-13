use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{Context, Result, bail};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::{
    api::{
        client::ApiClient,
        models::{Ayat, EnglishSurah, Surah},
    },
    audio::{
        cache::{AudioCache, audio_filename},
        player::{PlaybackControl, play_audio_controlled},
        tts::TtsEngine,
    },
    display,
    domain::{Lang, Qari},
};

pub struct PlaybackEngine {
    api: ApiClient,
    cache: AudioCache,
    tts: TtsEngine,
    dry_run: bool,
    prefetch_tts: bool,
    pregenerate_tts: bool,
    /// Read the ayah tafsir after the translation.
    read_tafsir: bool,
    tts_enabled: Arc<AtomicBool>,
    cancel_token: CancellationToken,
    /// Shared with the command layer so the current audio can be paused and
    /// resumed at the sample level instead of being torn down and restarted.
    control: Arc<PlaybackControl>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlaybackPhase {
    Recitation,
    Translation,
    Tafsir,
    Cache,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TtsGenerationPlan {
    current_ayah: bool,
    next_ayah: bool,
}

impl TtsGenerationPlan {
    fn new(tts_enabled: bool, prefetch_tts: bool, has_next_ayah: bool) -> Self {
        Self {
            current_ayah: tts_enabled,
            next_ayah: prefetch_tts && has_next_ayah,
        }
    }
}

impl PlaybackPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Recitation => "recitation",
            Self::Translation => "translation",
            Self::Tafsir => "tafsir",
            Self::Cache => "cache",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlaybackEvent {
    pub surah: u8,
    pub ayah: u16,
    pub phase: PlaybackPhase,
    pub current: usize,
    pub total: usize,
}

pub type PlaybackEventHandler = dyn Fn(PlaybackEvent) + Send + Sync;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AyahSelection {
    ayah_number: Option<u16>,
    from_ayah: Option<u16>,
    to_ayah: Option<u16>,
}

impl AyahSelection {
    pub fn new(ayah_number: Option<u16>, from_ayah: Option<u16>, to_ayah: Option<u16>) -> Self {
        Self {
            ayah_number,
            from_ayah,
            to_ayah,
        }
    }
}

/// One ayah plus the context every playback phase needs.
///
/// Grouping these values keeps the phase helpers from threading the same
/// eight arguments through every call.
struct AyahRequest<'a> {
    surah_number: u8,
    ayah: &'a Ayat,
    next_ayah: Option<&'a Ayat>,
    english_surah: Option<&'a EnglishSurah>,
    qari: Qari,
    lang: Lang,
    current: usize,
    total: usize,
}

impl PlaybackEngine {
    pub fn new(
        api: ApiClient,
        cache: AudioCache,
        dry_run: bool,
        prefetch_tts: bool,
        pregenerate_tts: bool,
    ) -> Self {
        let tts = TtsEngine::new(cache.root().clone());
        Self {
            api,
            cache,
            tts,
            dry_run,
            prefetch_tts,
            pregenerate_tts,
            read_tafsir: false,
            tts_enabled: Arc::new(AtomicBool::new(true)),
            cancel_token: CancellationToken::new(),
            control: Arc::new(PlaybackControl::new()),
        }
    }

    /// Reads the ayah tafsir after the translation when enabled.
    pub fn with_read_tafsir(mut self, read_tafsir: bool) -> Self {
        self.read_tafsir = read_tafsir;
        self
    }

    pub fn with_project_root(mut self, project_root: std::path::PathBuf) -> Self {
        self.tts = TtsEngine::with_project_root(self.cache.root().clone(), project_root);
        self
    }

    pub fn with_project_roots(mut self, project_roots: Vec<std::path::PathBuf>) -> Self {
        self.tts = TtsEngine::with_project_roots(self.cache.root().clone(), project_roots);
        self
    }

    pub fn with_skip_translation(self, skip: bool) -> Self {
        self.tts_enabled.store(!skip, Ordering::Relaxed);
        self
    }

    pub fn with_tts_enabled_flag(mut self, flag: Arc<AtomicBool>) -> Self {
        self.tts_enabled = flag;
        self
    }

    pub fn with_cancel_token(mut self, cancel_token: CancellationToken) -> Self {
        self.cancel_token = cancel_token;
        self
    }

    /// Shares one pause/resume control across every clip in the session
    /// (recitation, translation, tafsir), so the command layer can pause
    /// whatever is sounding without knowing which phase is running.
    pub fn with_playback_control(mut self, control: Arc<PlaybackControl>) -> Self {
        self.control = control;
        self
    }

    pub async fn play(
        &self,
        surah: &Surah,
        english_surah: Option<&EnglishSurah>,
        selection: AyahSelection,
        qari: Qari,
        lang: Lang,
    ) -> Result<()> {
        self.play_with_events(surah, english_surah, selection, qari, lang, None)
            .await
    }

    pub async fn play_with_events(
        &self,
        surah: &Surah,
        english_surah: Option<&EnglishSurah>,
        selection: AyahSelection,
        qari: Qari,
        lang: Lang,
        on_event: Option<&PlaybackEventHandler>,
    ) -> Result<()> {
        display::print_surah_header(surah);
        println!("Qari: {}", qari.label());

        let selected_ayat = self.selected_ayat(surah, selection)?;

        if !self.dry_run && self.pregenerate_tts {
            self.pregenerate_tts(surah, &selected_ayat, english_surah, lang)
                .await?;
        }

        for index in 0..selected_ayat.len() {
            if self.cancel_token.is_cancelled() {
                return Ok(());
            }
            let ayah = selected_ayat[index];
            let next_ayah = selected_ayat.get(index + 1).copied();
            self.play_ayah(
                AyahRequest {
                    surah_number: surah.nomor,
                    ayah,
                    next_ayah,
                    english_surah,
                    qari,
                    lang,
                    current: ayah.nomor_ayat as usize,
                    total: surah.jumlah_ayat as usize,
                },
                on_event,
            )
            .await?;
        }
        Ok(())
    }

    fn selected_ayat<'a>(
        &self,
        surah: &'a Surah,
        selection: AyahSelection,
    ) -> Result<Vec<&'a Ayat>> {
        if let Some(number) = selection.ayah_number {
            let ayah = surah
                .ayat
                .iter()
                .find(|ayah| ayah.nomor_ayat == number)
                .with_context(|| format!("surah {} does not contain ayah {number}", surah.nomor))?;
            return Ok(vec![ayah]);
        }

        if let Some(from) = selection.from_ayah {
            let to = selection.to_ayah.unwrap_or(surah.jumlah_ayat);
            if from > to {
                bail!("from-ayat must be less than or equal to to-ayat");
            }

            let ayat: Vec<&Ayat> = surah
                .ayat
                .iter()
                .filter(|ayah| (from..=to).contains(&ayah.nomor_ayat))
                .collect();
            if ayat.is_empty() {
                bail!(
                    "surah {} does not contain ayat {from} through {to}",
                    surah.nomor
                );
            }
            return Ok(ayat);
        }

        Ok(surah.ayat.iter().collect())
    }

    async fn pregenerate_tts(
        &self,
        surah: &Surah,
        ayat: &[&Ayat],
        english_surah: Option<&EnglishSurah>,
        lang: Lang,
    ) -> Result<()> {
        let total = ayat.len();
        self.tts.ensure_ready()?;
        println!(
            "Preparing {} TTS cache with {} backend...",
            lang.code(),
            self.tts.backend_name()
        );
        for (index, ayah) in ayat.iter().enumerate() {
            let translation = self.translation_for(ayah, english_surah, lang)?;
            println!("Generating TTS {}/{}...", index + 1, total);
            self.tts
                .synthesize_cached(translation, lang, surah.nomor, ayah.nomor_ayat)
                .await?;
        }
        println!("TTS cache ready.");
        Ok(())
    }

    /// Plays one ayah: recitation first, then the optional translation and
    /// tafsir voices, while the prefetch for the next ayah settles in the
    /// background.
    async fn play_ayah(
        &self,
        request: AyahRequest<'_>,
        on_event: Option<&PlaybackEventHandler>,
    ) -> Result<()> {
        let translation =
            self.translation_for(request.ayah, request.english_surah, request.lang)?;
        display::print_ayah(request.ayah, request.lang, translation);
        self.emit_event(on_event, &request, PlaybackPhase::Recitation);

        if self.dry_run {
            return Ok(());
        }

        let plan = TtsGenerationPlan::new(
            self.tts_enabled.load(Ordering::Relaxed),
            self.prefetch_tts,
            request.next_ayah.is_some(),
        );

        let translation_task = plan.current_ayah.then(|| {
            self.spawn_translation_tts(
                request.surah_number,
                request.ayah.nomor_ayat,
                translation.to_owned(),
                request.lang,
            )
        });
        // The tafsir is synthesised in parallel with the recitation so its
        // first chunk is usually ready by the time the translation ends.
        let tafsir_task = self.spawn_tafsir_tts(&request);
        let prefetch_task = self.spawn_prefetch_tts(&request, plan)?;

        if self.cancel_token.is_cancelled() {
            return Ok(());
        }

        self.play_recitation(&request).await?;

        if self.cancel_token.is_cancelled() {
            return Ok(());
        }

        if plan.current_ayah {
            self.play_translation(&request, translation, translation_task, on_event)
                .await?;
        }

        self.play_tafsir(&request, tafsir_task, on_event).await?;
        Self::await_prefetch(prefetch_task).await;

        Ok(())
    }

    /// Streams the qari recitation, downloading it into the cache when needed.
    async fn play_recitation(&self, request: &AyahRequest<'_>) -> Result<()> {
        let audio_url = request
            .ayah
            .audio
            .get(request.qari.key())
            .with_context(|| {
                format!(
                    "missing audio URL for qari {} ayah {}",
                    request.qari.key(),
                    request.ayah.nomor_ayat
                )
            })?;
        let audio_path = self
            .cache
            .get_or_download(
                &self.api,
                audio_url,
                &audio_filename(
                    request.qari.key(),
                    request.surah_number,
                    request.ayah.nomor_ayat,
                ),
            )
            .await?;
        play_audio_controlled(&audio_path, &self.cancel_token, &self.control).await
    }

    /// Plays the generated translation, waiting for the render that was started
    /// before the recitation so the two do not run back to back.
    async fn play_translation(
        &self,
        request: &AyahRequest<'_>,
        translation: &str,
        task: Option<JoinHandle<Result<PathBuf>>>,
        on_event: Option<&PlaybackEventHandler>,
    ) -> Result<()> {
        self.emit_event(on_event, request, PlaybackPhase::Translation);

        let tts_path = match task {
            Some(handle) => handle
                .await
                .context("TTS generation task was cancelled")??,
            None => {
                self.tts
                    .synthesize_cached(
                        translation,
                        request.lang,
                        request.surah_number,
                        request.ayah.nomor_ayat,
                    )
                    .await?
            }
        };

        self.emit_event(on_event, request, PlaybackPhase::Cache);
        play_audio_controlled(&tts_path, &self.cancel_token, &self.control)
            .await
            .or_else(|error| {
                bail!(
                    "failed to play generated TTS file {}: {error}",
                    tts_path.display()
                )
            })
    }

    /// Plays the tafsir chunks in order, aborting the render when the session
    /// was stopped while it was still being generated.
    async fn play_tafsir(
        &self,
        request: &AyahRequest<'_>,
        task: Option<JoinHandle<Result<Vec<PathBuf>>>>,
        on_event: Option<&PlaybackEventHandler>,
    ) -> Result<()> {
        let Some(handle) = task else {
            return Ok(());
        };

        if self.cancel_token.is_cancelled() {
            handle.abort();
            return Ok(());
        }

        self.emit_event(on_event, request, PlaybackPhase::Tafsir);

        let chunk_paths = handle
            .await
            .context("tafsir TTS generation task was cancelled")??;

        for chunk_path in chunk_paths {
            if self.cancel_token.is_cancelled() {
                return Ok(());
            }
            play_audio_controlled(&chunk_path, &self.cancel_token, &self.control)
                .await
                .with_context(|| {
                    format!(
                        "failed to play generated tafsir audio {}",
                        chunk_path.display()
                    )
                })?;
        }

        Ok(())
    }

    /// Renders the current ayah translation in the background.
    fn spawn_translation_tts(
        &self,
        surah_number: u8,
        ayah_number: u16,
        text: String,
        lang: Lang,
    ) -> JoinHandle<Result<PathBuf>> {
        let tts = self.tts.clone();
        tokio::spawn(async move {
            tts.synthesize_cached(&text, lang, surah_number, ayah_number)
                .await
        })
    }

    /// Renders every tafsir chunk in the background, when tafsir reading is on.
    fn spawn_tafsir_tts(
        &self,
        request: &AyahRequest<'_>,
    ) -> Option<JoinHandle<Result<Vec<PathBuf>>>> {
        let text = self
            .read_tafsir
            .then_some(request.ayah.tafsir.as_deref())
            .flatten()
            .map(str::trim)
            .filter(|text| !text.is_empty())?;

        let chunks = TtsEngine::split_tafsir_text(text);
        let tts = self.tts.clone();
        let lang = request.lang;
        let surah_number = request.surah_number;
        let ayah_number = request.ayah.nomor_ayat;

        Some(tokio::spawn(async move {
            let mut paths = Vec::with_capacity(chunks.len());
            for (index, chunk) in chunks.iter().enumerate() {
                paths.push(
                    tts.synthesize_tafsir_chunk_cached(
                        chunk,
                        lang,
                        surah_number,
                        ayah_number,
                        index + 1,
                    )
                    .await?,
                );
            }
            Ok(paths)
        }))
    }

    /// Warms the next ayah's translation while the current one plays.
    fn spawn_prefetch_tts(
        &self,
        request: &AyahRequest<'_>,
        plan: TtsGenerationPlan,
    ) -> Result<Option<JoinHandle<Result<PathBuf>>>> {
        if !plan.next_ayah {
            return Ok(None);
        }

        let next = request
            .next_ayah
            .expect("next ayah exists when prefetch is planned");
        let translation = self
            .translation_for(next, request.english_surah, request.lang)?
            .to_owned();

        Ok(Some(self.spawn_translation_tts(
            request.surah_number,
            next.nomor_ayat,
            translation,
            request.lang,
        )))
    }

    /// Waits for the prefetch so a failed render is not reported as a playback
    /// failure: the next ayah simply renders again when it starts.
    async fn await_prefetch(task: Option<JoinHandle<Result<PathBuf>>>) {
        if let Some(handle) = task {
            match handle.await {
                Ok(Ok(_)) | Ok(Err(_)) | Err(_) => {}
            }
        }
    }

    fn emit_event(
        &self,
        on_event: Option<&PlaybackEventHandler>,
        request: &AyahRequest<'_>,
        phase: PlaybackPhase,
    ) {
        if let Some(handler) = on_event {
            handler(PlaybackEvent {
                surah: request.surah_number,
                ayah: request.ayah.nomor_ayat,
                phase,
                current: request.current,
                total: request.total,
            });
        }
    }

    fn translation_for<'a>(
        &self,
        ayah: &'a Ayat,
        english_surah: Option<&'a EnglishSurah>,
        lang: Lang,
    ) -> Result<&'a str> {
        match lang {
            Lang::Id => Ok(ayah.teks_indonesia.as_str()),
            // The English API only ships the verse translation, which is a
            // single sentence. Reading the tafsir gives English listeners the
            // same depth Indonesian listeners get from `teksIndonesia`.
            Lang::En => ayah
                .tafsir
                .as_deref()
                .filter(|tafsir| !tafsir.trim().is_empty())
                .or_else(|| {
                    english_surah.and_then(|surah| surah.translation_for_ayah(ayah.nomor_ayat))
                })
                .context("English translation was requested but no English text is available"),
        }
    }
}

#[cfg(test)]
mod tests;
