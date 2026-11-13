//! BM25 ranking and result assembly.

use serde::{Deserialize, Serialize};

use super::field::build_field_index;
use super::query::{make_snippet, unique_tokens};
use super::{B, FIELD_NAMES, FIELD_WEIGHTS, K1, SearchIndex};

/// A single search result.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    /// Surah number, 1..=114.
    pub surah: u8,
    /// Ayah number within the surah.
    pub ayah: u16,
    /// Latin name of the surah.
    pub surah_name_latin: String,
    /// Weighted BM25 score.
    pub score: f32,
    /// Short excerpt from the highest-scoring field.
    pub snippet: String,
    /// Name of the field that produced the snippet.
    pub matched_field: String,
}

impl SearchIndex {
    /// Searches the index for `query` and returns at most `limit` hits.
    ///
    /// Every query token must match the entry (AND semantics); results are
    /// ranked by the total weighted BM25 score across all fields.
    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchHit> {
        if limit == 0 {
            return Vec::new();
        }
        let tokens = unique_tokens(query);
        if tokens.is_empty() {
            return Vec::new();
        }

        let count = self.entries.len();
        let fields = self.fields.get_or_init(|| {
            (0..FIELD_NAMES.len())
                .map(|field| build_field_index(&self.entries, field, FIELD_WEIGHTS[field]))
                .collect()
        });

        let mut scores = vec![0f32; count];
        let mut allowed = vec![true; count];
        // (best per-token weighted score, field index) for snippet selection.
        let mut best: Vec<(f32, usize)> = vec![(0.0, 2); count];

        for token in &tokens {
            let mut matched = vec![false; count];
            let mut any = false;
            for (field_pos, field) in fields.iter().enumerate() {
                if field.total_docs == 0 || field.avg_len <= 0.0 {
                    continue;
                }
                // Exact postings first, then prefix postings. A reader types
                // "bismillah" or "rahman" and expects to reach "bismillāhir"
                // and "raḥmān"; requiring the full transliteration would make
                // the most obvious queries miss.
                let mut expanded: Vec<&(usize, u32)> = Vec::new();
                for term in field.terms_with_prefix(token) {
                    if let Some(postings) = field.postings.get(term) {
                        expanded.extend(postings.iter());
                    }
                }
                if expanded.is_empty() {
                    continue;
                }
                any = true;
                let df = expanded.len() as f32;
                let total_docs = field.total_docs as f32;
                let idf = (1.0 + (total_docs - df + 0.5) / (df + 0.5)).ln();
                for &(idx, tf) in expanded {
                    let tf = tf as f32;
                    let doc_len = field.doc_len[idx] as f32;
                    let denom = tf + K1 * (1.0 - B + B * doc_len / field.avg_len);
                    let score = idf * (tf * (K1 + 1.0)) / denom * field.weight;
                    scores[idx] += score;
                    matched[idx] = true;
                    if score > best[idx].0 {
                        best[idx] = (score, field_pos);
                    }
                }
            }
            if !any {
                return Vec::new();
            }
            for (idx, did_match) in matched.iter().enumerate() {
                if !did_match {
                    allowed[idx] = false;
                }
            }
        }

        let mut ranked: Vec<(usize, f32)> = (0..count)
            .filter(|&idx| allowed[idx])
            .map(|idx| (idx, scores[idx]))
            .collect();
        ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        ranked.truncate(limit);

        ranked
            .into_iter()
            .map(|(idx, score)| {
                let entry = &self.entries[idx];
                let field_pos = best[idx].1;
                SearchHit {
                    surah: entry.surah,
                    ayah: entry.ayah,
                    surah_name_latin: entry.surah_name_latin.clone(),
                    score,
                    snippet: make_snippet(entry.original_field(field_pos), &tokens),
                    matched_field: FIELD_NAMES[field_pos].to_owned(),
                }
            })
            .collect()
    }
}
