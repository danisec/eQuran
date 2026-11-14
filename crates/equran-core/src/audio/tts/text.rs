//! Narration cleanup and sentence/paragraph splitting.

/// Removes text the local voice cannot pronounce.
///
/// The English tafsir is converted from an Urdu commentary and keeps the
/// source's Arabic and Urdu script inline: Quranic annotation marks, honorific
/// signs such as `ؓ` (U+0613), Urdu-only letters such as `ۃ` (U+06C3), ornate
/// ayah brackets and zero-width joiners. Supertonic reads the Latin narration
/// and rejects any code point outside its table, so one such character fails a
/// whole chunk and the narration stops mid-surah.
///
/// Both narration languages are written in Latin script, so dropping the
/// foreign script is the correct behaviour: the surrounding English or
/// Indonesian sentence stays complete and the Arabic quote, which the voice
/// could only mispronounce, disappears. The model's own character table is
/// still consulted by the worker as a final guard.
pub(super) fn sanitize_for_speech(text: &str) -> String {
    let mut cleaned = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(character) = chars.next() {
        match character {
            // `﴿ 98 ﴾` marks an ayah number. Drop the whole span so the
            // digits inside do not survive as a stray number.
            '\u{FD3E}' | '\u{FD3F}' => {
                // Logical order differs from the visual order in right-to-left
                // text, so either bracket can open the pair. Treat a lone
                // bracket as punctuation instead of swallowing the rest.
                let close = if character == '\u{FD3E}' {
                    '\u{FD3F}'
                } else {
                    '\u{FD3E}'
                };
                let mut probe = chars.clone();
                let paired = (0..32).any(|_| match probe.next() {
                    Some(next) => next == close,
                    None => false,
                });
                if paired {
                    for next in chars.by_ref() {
                        if next == close {
                            break;
                        }
                    }
                }
                push_separator(&mut cleaned);
            }
            _ if is_invisible(character) || is_arabic_script(character) => {
                // Keep the words on both sides from being glued together.
                push_separator(&mut cleaned);
            }
            '\u{201E}' | '\u{201F}' => cleaned.push('"'),
            '\u{201A}' | '\u{201B}' | '\u{2032}' => cleaned.push('\''),
            '\u{2033}' => cleaned.push('"'),
            '\u{00A0}' | '\u{2007}' | '\u{202F}' => cleaned.push(' '),
            _ => cleaned.push(character),
        }
    }

    tidy(&cleaned)
}

fn push_separator(cleaned: &mut String) {
    if !cleaned.is_empty() && !cleaned.ends_with(char::is_whitespace) {
        cleaned.push(' ');
    }
}

fn is_arabic_script(character: char) -> bool {
    matches!(
        character as u32,
        0x0600..=0x06FF       // Arabic, including Urdu letters and marks
            | 0x0750..=0x077F // Arabic Supplement
            | 0x0870..=0x089F // Arabic Extended-B
            | 0x08A0..=0x08FF // Arabic Extended-A
            | 0xFB50..=0xFDFF // Arabic Presentation Forms-A
            | 0xFE70..=0xFEFF // Arabic Presentation Forms-B
            | 0x1EE00..=0x1EEFF // Arabic Mathematical Alphabetic Symbols
    )
}

fn is_invisible(character: char) -> bool {
    matches!(
        character as u32,
        0x00AD // soft hyphen
            | 0x200B..=0x200F // zero-width spaces and bidi marks
            | 0x202A..=0x202E // bidi embeddings
            | 0x2060..=0x2064 // word joiner and invisible operators
            | 0x2066..=0x2069 // bidi isolates
            | 0xFEFF // zero-width no-break space
    )
}

/// Collapses the gaps left by dropped script and removes punctuation that no
/// longer has anything around it (`Jibra'il ()` becomes `Jibra'il`).
fn tidy(text: &str) -> String {
    let mut collapsed = String::with_capacity(text.len());
    let mut pending_space = false;
    let mut pending_newlines = 0usize;
    for character in text.chars() {
        // Paragraph breaks are the unit the tafsir splitter cuts on later, so
        // they must survive the cleanup.
        if character == '\n' {
            pending_newlines += 1;
            pending_space = false;
            continue;
        }
        if character.is_whitespace() {
            pending_space = !collapsed.is_empty();
            continue;
        }

        if pending_newlines > 0 {
            while collapsed.ends_with(' ') {
                collapsed.pop();
            }
            for _ in 0..pending_newlines.min(2) {
                collapsed.push('\n');
            }
            pending_newlines = 0;
            pending_space = false;
        }

        let is_closing = matches!(
            character,
            ',' | '.' | ';' | ':' | '!' | '?' | ')' | ']' | '}'
        );
        let follows_opening = collapsed
            .chars()
            .next_back()
            .is_some_and(|previous| matches!(previous, '(' | '[' | '{' | '\n'));
        if pending_space && !is_closing && !follows_opening {
            collapsed.push(' ');
        }
        pending_space = false;

        if is_closing {
            while collapsed
                .chars()
                .next_back()
                .is_some_and(|previous| matches!(previous, ' ' | ',' | ';' | ':'))
            {
                collapsed.pop();
            }
        }
        if matches!(character, '(' | '[' | '{') {
            while collapsed.ends_with(' ') {
                collapsed.pop();
            }
        }
        collapsed.push(character);
    }

    collapse_whitespace(&remove_empty_brackets(&collapsed))
}

fn collapse_whitespace(text: &str) -> String {
    text.lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect::<Vec<_>>()
        .join("\n")
}

fn remove_empty_brackets(text: &str) -> String {
    let mut current = text.to_owned();
    loop {
        let mut next = String::with_capacity(current.len());
        let mut chars = current.chars().peekable();
        let mut removed = false;

        while let Some(character) = chars.next() {
            let close = match character {
                '(' => ')',
                '[' => ']',
                '{' => '}',
                _ => {
                    next.push(character);
                    continue;
                }
            };

            let mut lookahead = chars.clone();
            let mut found = false;
            while let Some(&candidate) = lookahead.peek() {
                if candidate == close {
                    found = true;
                    lookahead.next();
                    break;
                }
                if candidate.is_whitespace() || matches!(candidate, ',' | ';' | ':' | '.' | '!') {
                    lookahead.next();
                    continue;
                }
                break;
            }

            if found {
                chars = lookahead;
                removed = true;
            } else {
                next.push(character);
            }
        }

        if !removed {
            return next.trim().to_owned();
        }
        current = next;
    }
}

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

    // A single sentence can be longer than the limit. Leaving it whole would
    // make one oversized synthesis request, which can outlive the worker
    // timeout and stall every request queued behind it.
    chunks
        .into_iter()
        .flat_map(|chunk| split_oversized_chunk(chunk, max_chars))
        .collect()
}

fn split_oversized_chunk(chunk: String, max_chars: usize) -> Vec<String> {
    if chunk.chars().count() <= max_chars {
        return vec![chunk];
    }

    let mut pieces = Vec::new();
    let mut current = String::new();
    for word in chunk.split_whitespace() {
        let separator = usize::from(!current.is_empty());
        if !current.is_empty()
            && current.chars().count() + separator + word.chars().count() > max_chars
        {
            pieces.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        pieces.push(current);
    }
    pieces
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
