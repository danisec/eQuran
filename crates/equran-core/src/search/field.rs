//! Inverted index and length statistics for one scored field.

use std::collections::HashMap;

use super::entry::SearchEntry;

// ---------------------------------------------------------------------------
// Query and scoring
// ---------------------------------------------------------------------------

/// An inverted index and statistics for a single scored field.
#[derive(Debug)]
pub(super) struct FieldIndex {
    /// term -> (entry index, term frequency)
    pub(super) postings: HashMap<String, Vec<(usize, u32)>>,
    /// Token count of each entry's field.
    pub(super) doc_len: Vec<usize>,
    /// Average token count across entries that have this field.
    pub(super) avg_len: f32,
    /// Number of entries with a non-empty value for this field.
    pub(super) total_docs: usize,
    /// BM25 field weight.
    pub(super) weight: f32,
}

impl FieldIndex {
    /// Returns every indexed term that starts with `token`.
    ///
    /// Exact matches come first so an exact hit is never diluted by a longer
    /// one. Terms shorter than the token are skipped: a reader typing
    /// "bismillah" should not match a document that only contains "bis".
    pub(super) fn terms_with_prefix(&self, token: &str) -> Vec<&str> {
        let mut exact = None;
        let mut prefixed = Vec::new();
        for term in self.postings.keys() {
            if term == token {
                exact = Some(term.as_str());
                break;
            }
            if term.len() > token.len() && term.starts_with(token) {
                prefixed.push(term.as_str());
            }
        }
        let mut terms = Vec::with_capacity(prefixed.len() + 1);
        terms.extend(exact);
        terms.extend(prefixed);
        terms
    }
}

/// Builds the inverted index for one field.
pub(super) fn build_field_index(entries: &[SearchEntry], field: usize, weight: f32) -> FieldIndex {
    let mut postings: HashMap<String, Vec<(usize, u32)>> = HashMap::new();
    let mut doc_len = vec![0usize; entries.len()];
    let mut total_len = 0usize;
    let mut total_docs = 0usize;

    for (idx, entry) in entries.iter().enumerate() {
        let text = entry.norm_field(field);
        let mut term_freq: HashMap<&str, u32> = HashMap::new();
        let mut count = 0usize;
        for term in text.split_whitespace() {
            *term_freq.entry(term).or_insert(0) += 1;
            count += 1;
        }
        doc_len[idx] = count;
        if count > 0 {
            total_docs += 1;
            total_len += count;
        }
        for (term, freq) in term_freq {
            postings
                .entry(term.to_owned())
                .or_default()
                .push((idx, freq));
        }
    }

    let avg_len = if total_docs > 0 {
        total_len as f32 / total_docs as f32
    } else {
        0.0
    };
    FieldIndex {
        postings,
        doc_len,
        avg_len,
        total_docs,
        weight,
    }
}
