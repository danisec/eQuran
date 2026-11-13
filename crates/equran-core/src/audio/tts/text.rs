//! Sentence and paragraph splitting for long tafsir narration.

pub(super) fn split_text_for_tts(text: &str, max_chars: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    for paragraph in text.split("\n\n").map(str::trim).filter(|p| !p.is_empty()) {
        if paragraph.chars().count() <= max_chars {
            chunks.push(paragraph.to_owned());
            continue;
        }

        let mut current = String::new();
        for sentence in split_sentences(paragraph) {
            let sentence_len = sentence.chars().count();
            let current_len = current.chars().count();
            let separator_len = usize::from(!current.is_empty());

            if current_len + separator_len + sentence_len > max_chars && !current.is_empty() {
                chunks.push(current.trim().to_owned());
                current.clear();
            }

            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(&sentence);
        }

        if !current.trim().is_empty() {
            chunks.push(current.trim().to_owned());
        }
    }

    if chunks.is_empty() && !text.trim().is_empty() {
        chunks.push(text.trim().to_owned());
    }
    chunks
}

fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    for character in text.chars() {
        current.push(character);
        if matches!(character, '.' | '!' | '?' | '।' | '؛') {
            let trimmed = current.trim();
            if !trimmed.is_empty() {
                sentences.push(trimmed.to_owned());
            }
            current.clear();
        }
    }
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        sentences.push(trimmed.to_owned());
    }
    sentences
}
