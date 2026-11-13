/** A run of snippet text, either plain or matching the current query. */
export type HighlightSegment = {
  text: string;
  match: boolean;
};

/**
 * Splits `text` into plain and matching segments for a search query.
 *
 * The query is lowercased and split on whitespace; one-character words are
 * ignored so a stray letter does not highlight half a snippet. Matching is
 * case-insensitive and always takes the earliest occurrence. When several
 * words start at the same offset, the longest one wins, so the highlight never
 * stops short inside a longer query word.
 *
 * The segments always concatenate back to `text` exactly: callers can render
 * them in order without losing or duplicating characters.
 */
export function highlightSegments(text: string, query: string): HighlightSegment[] {
  const words = query
    .trim()
    .toLowerCase()
    .split(/\s+/)
    .filter((word) => word.length > 1);

  if (words.length === 0) return [{ text, match: false }];

  const haystack = text.toLowerCase();
  const segments: HighlightSegment[] = [];
  let index = 0;

  while (index < text.length) {
    let best = -1;
    let bestLength = 0;

    for (const word of words) {
      const found = haystack.indexOf(word, index);
      if (found === -1) continue;

      const isLongerAtSameOffset = found === best && word.length > bestLength;
      if (best === -1 || found < best || isLongerAtSameOffset) {
        best = found;
        bestLength = word.length;
      }
    }

    if (best === -1) break;

    if (best > index) {
      segments.push({ text: text.slice(index, best), match: false });
    }
    segments.push({ text: text.slice(best, best + bestLength), match: true });
    index = best + bestLength;
  }

  if (index < text.length) {
    segments.push({ text: text.slice(index), match: false });
  }

  return segments;
}
