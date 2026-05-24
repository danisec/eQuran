use crate::{
    api::models::{Ayat, Surah, SurahListItem},
    domain::Lang,
};

pub fn print_surah_list(items: &[SurahListItem]) {
    println!(
        "{:>3}  {:<28} {:<14} {:<12} {:>5}  Arti",
        "No", "Nama Latin", "Arab", "Turun", "Ayat"
    );
    for item in items {
        println!(
            "{:>3}  {:<28} {:<14} {:<12} {:>5}  {}",
            item.nomor, item.nama_latin, item.nama, item.tempat_turun, item.jumlah_ayat, item.arti
        );
    }
}

pub fn print_surah_header(surah: &Surah) {
    println!("{} ({})", surah.nama_latin, surah.nama);
    println!(
        "Surah {} · {} ayat · {} · {}",
        surah.nomor, surah.jumlah_ayat, surah.tempat_turun, surah.arti
    );
}

pub fn print_surah_info(surah: &Surah) {
    print_surah_header(surah);
    println!();
    println!("{}", strip_html(&surah.deskripsi));
    println!();
    println!("Audio full tersedia untuk {} qari.", surah.audio_full.len());
}

pub fn print_ayah(ayah: &Ayat, lang: Lang, translation: &str) {
    println!();
    println!("Ayat {}", ayah.nomor_ayat);
    println!("{}", ayah.teks_arab);
    println!("{}", ayah.teks_latin.trim());
    match lang {
        Lang::Id => println!("ID: {translation}"),
        Lang::En => println!("EN: {translation}"),
    }
}

fn strip_html(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut inside_tag = false;
    for character in input.chars() {
        match character {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => output.push(character),
            _ => {}
        }
    }
    output.replace("&nbsp;", " ").replace("<br>", "\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_basic_html_tags() {
        assert_eq!(strip_html("A <i>test</i> text"), "A test text");
    }
}
