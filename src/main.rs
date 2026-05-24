mod cli;

use anyhow::Result;
use clap::Parser;

use equran_core::{
    api::client::ApiClient,
    audio::cache::AudioCache,
    display,
    domain::Lang,
    playback::{AyahSelection, PlaybackEngine},
};

use crate::cli::{Cli, Command};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let api = ApiClient::new()?;

    match cli.command {
        Command::List => {
            let surahs = api.fetch_surah_list().await?;
            display::print_surah_list(&surahs);
        }
        Command::Info { surah } => {
            let detail = api.fetch_surah(surah).await?;
            display::print_surah_info(&detail);
        }
        Command::Play {
            surah,
            ayat,
            from_ayat,
            to_ayat,
            qari,
            lang,
            dry_run,
            no_prefetch,
            pregenerate_tts,
        } => {
            let detail = api.fetch_surah(surah).await?;
            let english_detail = if lang == Lang::En {
                Some(api.fetch_surah_english(surah).await?)
            } else {
                None
            };
            let cache = AudioCache::new()?;
            let player = PlaybackEngine::new(api, cache, dry_run, !no_prefetch, pregenerate_tts);
            let selection = AyahSelection::new(ayat, from_ayat, to_ayat);
            player
                .play(&detail, english_detail.as_ref(), selection, qari, lang)
                .await?;
        }
    }

    Ok(())
}
