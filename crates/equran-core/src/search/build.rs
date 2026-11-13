//! Fetches every surah and assembles the searchable index.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::api::client::ApiClient;

use super::entry::SearchEntry;
use super::normalize::revelation_label;
use super::{INDEX_VERSION, MAX_CONCURRENCY, SURAH_COUNT, SearchIndex, field_index_slot, now_unix};

/// Fetches and assembles every entry for a single surah.
///
/// The Indonesian text is mandatory; English and tafsir failures are tolerated
/// and leave those fields empty so the index stays usable offline.
async fn fetch_surah_entries(api: &ApiClient, number: u8) -> Result<Vec<SearchEntry>> {
    let surah = api
        .fetch_surah(number)
        .await
        .with_context(|| format!("failed to fetch surah {number} for the search index"))?;
    let surah_name_latin = surah.nama_latin.clone();
    let revelation = revelation_label(&surah.tempat_turun);

    let english = api.fetch_surah_english(number).await.ok();
    let tafsir = api.fetch_tafsir(number).await.ok();

    let english_map: HashMap<u16, String> = english
        .map(|surah| {
            surah
                .ayahs
                .into_iter()
                .map(|ayah| (ayah.number_in_surah, ayah.text_english))
                .collect()
        })
        .unwrap_or_default();
    let tafsir_map: HashMap<u16, String> = tafsir
        .map(|surah| {
            surah
                .tafsir
                .into_iter()
                .map(|ayah| (ayah.ayat, ayah.teks))
                .collect()
        })
        .unwrap_or_default();

    let mut entries = Vec::with_capacity(surah.ayat.len());
    for ayat in surah.ayat {
        let teks_english = english_map.get(&ayat.nomor_ayat).cloned();
        let tafsir_text = tafsir_map.get(&ayat.nomor_ayat).cloned();
        entries.push(SearchEntry::assemble(
            number,
            ayat.nomor_ayat,
            surah_name_latin.clone(),
            revelation.clone(),
            ayat.teks_arab,
            ayat.teks_latin,
            ayat.teks_indonesia,
            teks_english,
            tafsir_text,
        ));
    }
    Ok(entries)
}

/// Builds the whole index from the network.
///
/// All 114 surahs are fetched with at most [`MAX_CONCURRENCY`] requests in
/// flight. `progress(done, total)` is called once per completed surah.
pub async fn build(
    api: &ApiClient,
    progress: impl Fn(usize, usize) + Send + Sync,
) -> Result<SearchIndex> {
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENCY));
    let mut tasks: JoinSet<Result<Vec<SearchEntry>>> = JoinSet::new();
    for number in 1..=SURAH_COUNT {
        let api = api.clone();
        let semaphore = Arc::clone(&semaphore);
        tasks.spawn(async move {
            let permit = semaphore
                .acquire_owned()
                .await
                .context("search index build semaphore closed")?;
            let entries = fetch_surah_entries(&api, number).await?;
            drop(permit);
            Ok(entries)
        });
    }

    let total = SURAH_COUNT as usize;
    let mut done = 0usize;
    let mut entries = Vec::new();
    while let Some(joined) = tasks.join_next().await {
        let surah_entries = joined.context("search index worker task panicked")??;
        entries.extend(surah_entries);
        done += 1;
        progress(done, total);
    }
    entries.sort_by_key(|entry| (entry.surah, entry.ayah));
    Ok(SearchIndex {
        version: INDEX_VERSION,
        built_at: now_unix(),
        entries,
        fields: field_index_slot(),
    })
}
