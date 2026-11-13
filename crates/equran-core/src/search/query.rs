//! Query token matching and snippet extraction.

use std::collections::HashSet;

use super::SNIPPET_MAX_CHARS;
use super::normalize::{contains_arabic, normalize_arabic, normalize_latin, tokenize};

/// Returns the unique normalised query tokens, preserving order.
pub(super) fn unique_tokens(query: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut tokens = Vec::new();
    for token in tokenize(query) {
        if seen.insert(token.clone()) {
            tokens.push(token);
        }
    }
    tokens
}

/// Normalises a single word using the script-appropriate normaliser.
fn normalize_word(word: &str) -> String {
    if contains_arabic(word) {
        normalize_arabic(word)
    } else {
        normalize_latin(word)
    }
}

/// Returns true when the normalised `word` matches any query token.
fn word_matches_token(word: &str, tokens: &[String]) -> bool {
    let normalized = normalize_word(word);
    if normalized.is_empty() {
        return false;
    }
    tokens.iter().any(|token| {
        normalized == *token
            || normalized.starts_with(token.as_str())
            || token.starts_with(normalized.as_str())
    })
}

/// Finds the character index of the first word that matches a query token.
fn find_match_center(text: &str, tokens: &[String]) -> Option<usize> {
    let mut word = String::new();
    let mut word_start = 0usize;
    for (char_pos, ch) in text.chars().enumerate() {
        if ch.is_whitespace() {
            if !word.is_empty() {
                if word_matches_token(&word, tokens) {
                    return Some(word_start);
                }
                word.clear();
            }
        } else {
            if word.is_empty() {
                word_start = char_pos;
            }
            word.push(ch);
        }
    }
    if !word.is_empty() && word_matches_token(&word, tokens) {
        return Some(word_start);
    }
    None
}

/// Builds a snippet of at most [`SNIPPET_MAX_CHARS`] characters, roughly
/// centred on the first matching word.
pub(super) fn make_snippet(text: &str, tokens: &[String]) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() <= SNIPPET_MAX_CHARS {
        return trimmed.to_owned();
    }

    let center = find_match_center(trimmed, tokens).unwrap_or(0);
    let half = SNIPPET_MAX_CHARS / 2;
    let start = center
        .saturating_sub(half)
        .min(chars.len().saturating_sub(SNIPPET_MAX_CHARS));
    let end = (start + SNIPPET_MAX_CHARS).min(chars.len());

    let mut budget = SNIPPET_MAX_CHARS;
    let mut snippet = String::new();
    if start > 0 {
        snippet.push('…');
        budget -= 1;
    }
    let take = (end - start).min(budget);
    snippet.extend(chars[start..start + take].iter());
    if end < chars.len() && snippet.chars().count() < SNIPPET_MAX_CHARS {
        snippet.push('…');
    }
    snippet.trim().to_owned()
}
