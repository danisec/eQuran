//! Script-aware normalisation and tokenisation shared by the index and query.

/// Folds whitespace runs into single spaces and trims the result.
fn collapse_whitespace(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Returns true when the character is an Arabic combining mark (harakat or a
/// Quranic annotation) that should be ignored for matching.
fn is_arabic_mark(ch: char) -> bool {
    matches!(ch,
        '\u{064B}'..='\u{0655}' // fathatan..hamza above (includes madda above)
        | '\u{0670}'            // superscript alef
        | '\u{06D6}'..='\u{06ED}' // Quranic annotation signs
    )
}

/// Returns true when the string contains at least one Arabic codepoint.
pub fn contains_arabic(input: &str) -> bool {
    input.chars().any(|ch| {
        matches!(ch,
            '\u{0600}'..='\u{06FF}'
            | '\u{0750}'..='\u{077F}'
            | '\u{08A0}'..='\u{08FF}'
            | '\u{FB50}'..='\u{FDFF}'
            | '\u{FE70}'..='\u{FEFF}'
        )
    })
}

/// Normalises Arabic text for matching.
///
/// Harakat/diacritics are stripped, tatweel is removed, common letter variants
/// are unified (`أ إ آ ٱ → ا`, `ى → ي`, `ة → ه`, `ؤ → و`, `ئ → ي`) and
/// whitespace is collapsed. The result keeps letters and digits only.
pub fn normalize_arabic(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        if is_arabic_mark(ch) || ch == '\u{0640}' {
            continue;
        }
        let unified = match ch {
            '\u{0623}' | '\u{0625}' | '\u{0622}' | '\u{0671}' => '\u{0627}', // أ إ آ ٱ → ا
            '\u{0649}' => '\u{064A}',                                        // ى → ي
            '\u{0629}' => '\u{0647}',                                        // ة → ه
            '\u{0624}' => '\u{0648}',                                        // ؤ → و
            '\u{0626}' => '\u{064A}',                                        // ئ → ي
            other => other,
        };
        out.push(unified);
    }
    collapse_whitespace(&out)
}

/// Folds a single latin character to its ascii-ish base, if it has one.
fn fold_latin(ch: char) -> Option<&'static str> {
    let folded = match ch {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' | 'ǎ' | 'ǟ' | 'ǡ' | 'ǻ' | 'ȁ' | 'ȃ'
        | 'ȧ' | 'ḁ' | 'ạ' | 'ả' | 'ấ' | 'ầ' | 'ẩ' | 'ẫ' | 'ậ' | 'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ' => {
            "a"
        }
        'æ' => "ae",
        'ƀ' | 'ƃ' | 'ƅ' | 'ḃ' | 'ḅ' | 'ḇ' => "b",
        'ç' | 'ć' | 'ĉ' | 'ċ' | 'č' | 'ƈ' | 'ḉ' => "c",
        'ð' | 'ď' | 'đ' | 'ƌ' | 'ǆ' | 'ǳ' | 'ḋ' | 'ḍ' | 'ḏ' | 'ḑ' | 'ḓ' => "d",
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' | 'ȅ' | 'ȇ' | 'ȩ' | 'ḕ' | 'ḗ' | 'ḙ'
        | 'ḛ' | 'ḝ' | 'ẹ' | 'ẻ' | 'ẽ' | 'ế' | 'ề' | 'ể' | 'ễ' | 'ệ' => "e",
        'ƒ' | 'ḟ' => "f",
        'ĝ' | 'ğ' | 'ġ' | 'ģ' | 'ǧ' | 'ǵ' | 'ḡ' => "g",
        'ĥ' | 'ħ' | 'ƕ' | 'ȟ' | 'ḣ' | 'ḥ' | 'ḧ' | 'ḩ' | 'ḫ' | 'ẖ' => "h",
        'ì' | 'í' | 'î' | 'ï' | 'ĩ' | 'ī' | 'ĭ' | 'į' | 'ı' | 'ǐ' | 'ȉ' | 'ȋ' | 'ḭ' | 'ḯ' | 'ỉ'
        | 'ị' => "i",
        'ĵ' | 'ǰ' => "j",
        'ķ' | 'ĸ' | 'ƙ' | 'ǩ' | 'ḱ' | 'ḳ' | 'ḵ' => "k",
        'ĺ' | 'ļ' | 'ľ' | 'ł' | 'ƚ' | 'ƛ' | 'ǉ' | 'ḷ' | 'ḹ' | 'ḻ' | 'ḽ' => "l",
        'ḿ' | 'ṁ' | 'ṃ' => "m",
        'ñ' | 'ń' | 'ņ' | 'ň' | 'ŋ' | 'ƞ' | 'ǌ' | 'ǹ' | 'ṅ' | 'ṇ' | 'ṉ' | 'ṋ' => {
            "n"
        }
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ō' | 'ŏ' | 'ő' | 'ơ' | 'ǒ' | 'ǫ' | 'ǭ' | 'ǿ' | 'ȍ' | 'ȏ'
        | 'ȫ' | 'ȭ' | 'ȯ' | 'ȱ' | 'ṍ' | 'ṏ' | 'ṑ' | 'ṓ' | 'ọ' | 'ỏ' | 'ố' | 'ồ' | 'ổ' | 'ỗ'
        | 'ộ' | 'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ' => "o",
        'œ' => "oe",
        'ƥ' | 'ṕ' | 'ṗ' => "p",
        'ƣ' => "q",
        'ŕ' | 'ŗ' | 'ř' | 'ȑ' | 'ȓ' | 'ṙ' | 'ṛ' | 'ṝ' | 'ṟ' => "r",
        'ś' | 'ŝ' | 'ş' | 'š' | 'ƨ' | 'ƪ' | 'ș' | 'ṡ' | 'ṣ' | 'ṥ' | 'ṧ' | 'ṩ' => {
            "s"
        }
        'ß' => "ss",
        'ţ' | 'ť' | 'ŧ' | 'ƫ' | 'ƭ' | 'ț' | 'ṫ' | 'ṭ' | 'ṯ' | 'ṱ' | 'ẗ' => "t",
        'þ' => "th",
        'ù' | 'ú' | 'û' | 'ü' | 'ũ' | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' | 'ư' | 'ǔ' | 'ǖ' | 'ǘ' | 'ǚ'
        | 'ǜ' | 'ȕ' | 'ȗ' | 'ṳ' | 'ṵ' | 'ṷ' | 'ṹ' | 'ṻ' | 'ụ' | 'ủ' | 'ứ' | 'ừ' | 'ử' | 'ữ'
        | 'ự' => "u",
        'ṽ' | 'ṿ' => "v",
        'ŵ' | 'ẁ' | 'ẃ' | 'ẅ' | 'ẇ' | 'ẉ' | 'ẘ' => "w",
        'ẋ' | 'ẍ' => "x",
        'ý' | 'ÿ' | 'ŷ' | 'ƴ' | 'ȳ' | 'ẏ' | 'ẙ' | 'ỳ' | 'ỵ' | 'ỷ' | 'ỹ' => "y",
        'ź' | 'ż' | 'ž' | 'ƶ' | 'ƹ' | 'ƺ' | 'ƽ' | 'ẑ' | 'ẓ' | 'ẕ' => "z",
        _ => return None,
    };
    Some(folded)
}

/// Returns true for the apostrophe-like marks that are dropped entirely so a
/// word such as `Qur'an` matches `quran`.
fn is_apostrophe(ch: char) -> bool {
    matches!(
        ch,
        '\'' | '\u{02BC}' | '\u{02BE}' | '\u{02BF}' | '\u{2018}' | '\u{2019}'
    )
}

/// Normalises latin (transliteration/translation) text for matching.
///
/// Text is lowercased, combining diacritics in `U+0300..=U+036F` are dropped,
/// precomposed accents are folded to their ascii base (`ā → a`, `ḥ → h`,
/// `ṣ → s`, ...), apostrophes are removed and punctuation becomes whitespace.
pub fn normalize_latin(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        if ('\u{0300}'..='\u{036F}').contains(&ch) || is_apostrophe(ch) {
            continue;
        }
        if let Some(folded) = fold_latin(ch) {
            out.push_str(folded);
            continue;
        }
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            continue;
        }
        if ch.is_alphabetic() {
            for lowered in ch.to_lowercase() {
                if ('\u{0300}'..='\u{036F}').contains(&lowered) {
                    continue;
                }
                if let Some(folded) = fold_latin(lowered) {
                    out.push_str(folded);
                } else if lowered.is_alphanumeric() {
                    out.push(lowered);
                } else {
                    out.push(' ');
                }
            }
            continue;
        }
        out.push(' ');
    }
    collapse_whitespace(&out)
}

/// Alias of [`normalize_latin`] for Indonesian text.
pub fn normalize_indonesian(input: &str) -> String {
    normalize_latin(input)
}

/// Tokenises free text into normalised terms.
///
/// Each whitespace-delimited chunk is routed to the Arabic or latin normaliser
/// based on its script, then split again so punctuation-joined fragments become
/// separate terms.
pub fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    for raw in input.split_whitespace() {
        let normalized = if contains_arabic(raw) {
            normalize_arabic(raw)
        } else {
            normalize_latin(raw)
        };
        for piece in normalized.split_whitespace() {
            tokens.push(piece.to_owned());
        }
    }
    tokens
}

/// Maps the API revelation label to the canonical Arabic-derived spelling.
pub(super) fn revelation_label(raw: &str) -> String {
    if raw.to_lowercase().contains("madin") {
        "Madaniyah".to_owned()
    } else {
        "Makkiyah".to_owned()
    }
}
