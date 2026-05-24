use anyhow::{Result, bail};
use clap::{Parser, Subcommand};
use equran_core::domain::{Lang, Qari};

#[derive(Debug, Parser)]
#[command(name = "equran")]
#[command(
    version,
    about = "Al-Quran CLI player with qari audio and translation voice"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    List,
    Info {
        #[arg(long, value_parser = parse_surah_number)]
        surah: u8,
    },
    Play {
        #[arg(long, value_parser = parse_surah_number)]
        surah: u8,
        #[arg(long)]
        ayat: Option<u16>,
        #[arg(long, value_parser = parse_ayah_number, conflicts_with = "ayat", requires = "to_ayat")]
        from_ayat: Option<u16>,
        #[arg(long, value_parser = parse_ayah_number, conflicts_with = "ayat", requires = "from_ayat")]
        to_ayat: Option<u16>,
        #[arg(long, default_value = "misyari")]
        qari: Qari,
        #[arg(long, default_value = "id")]
        lang: Lang,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        no_prefetch: bool,
        #[arg(long)]
        pregenerate_tts: bool,
    },
}

fn parse_surah_number(value: &str) -> Result<u8> {
    let number: u8 = value.parse()?;
    if !(1..=114).contains(&number) {
        bail!("surah must be between 1 and 114");
    }
    Ok(number)
}

fn parse_ayah_number(value: &str) -> Result<u16> {
    let number: u16 = value.parse()?;
    if number == 0 {
        bail!("ayah must be greater than 0");
    }
    Ok(number)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn maps_qari_shortnames_to_api_keys() {
        assert_eq!("misyari".parse::<Qari>().expect("qari").key(), "05");
        assert_eq!("sudais".parse::<Qari>().expect("qari").key(), "03");
        assert!("unknown".parse::<Qari>().is_err());
    }

    #[test]
    fn rejects_invalid_surah_numbers() {
        assert!(parse_surah_number("1").is_ok());
        assert!(parse_surah_number("114").is_ok());
        assert!(parse_surah_number("0").is_err());
        assert!(parse_surah_number("115").is_err());
    }

    #[test]
    fn rejects_invalid_ayah_numbers() {
        assert!(parse_ayah_number("1").is_ok());
        assert!(parse_ayah_number("0").is_err());
    }

    #[test]
    fn parses_ayah_range_for_playback() {
        let cli = Cli::try_parse_from([
            "equran",
            "play",
            "--surah",
            "2",
            "--from-ayat",
            "1",
            "--to-ayat",
            "3",
        ])
        .expect("range args should parse");

        let Command::Play {
            from_ayat,
            to_ayat,
            ayat,
            ..
        } = cli.command
        else {
            panic!("expected play command");
        };

        assert_eq!(ayat, None);
        assert_eq!(from_ayat, Some(1));
        assert_eq!(to_ayat, Some(3));
    }

    #[test]
    fn rejects_single_ayah_combined_with_range() {
        let result = Cli::try_parse_from([
            "equran",
            "play",
            "--surah",
            "2",
            "--ayat",
            "1",
            "--from-ayat",
            "1",
            "--to-ayat",
            "3",
        ]);

        assert!(result.is_err());
    }
}
