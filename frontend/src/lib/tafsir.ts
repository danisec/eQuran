/**
 * UmmahAPI sends English commentary as one unbroken string: the source book has
 * paragraphs, but the API strips them. Rendering that with `white-space:
 * pre-line` produces a wall of text, so we rebuild readable paragraphs at
 * sentence boundaries.
 *
 * Text that already contains line breaks (the Indonesian source) is returned
 * untouched.
 */

const SENTENCES_PER_PARAGRAPH = 3;

/** Periods that belong to an abbreviation, not to the end of a sentence. */
const ABBREVIATIONS = [
  'e.g.',
  'i.e.',
  'etc.',
  'vs.',
  'mr.',
  'mrs.',
  'ms.',
  'dr.',
  'cf.',
  'ibid.',
  'pp.',
  'vol.',
  'fig.',
  'approx.'
];

function isAbbreviation(text: string, periodIndex: number): boolean {
  const fragment = text.slice(Math.max(0, periodIndex - 12), periodIndex + 1).toLowerCase();
  if (ABBREVIATIONS.some((abbreviation) => fragment.endsWith(abbreviation))) {
    return true;
  }

  // A single initial such as "A." or "M." does not end a sentence.
  return /(?:^|[\s("'“])[a-z]\.$/.test(fragment);
}

/**
 * Offsets just after a sentence-ending punctuation mark (and any closing
 * quote or bracket). The whitespace after the offset belongs to the next
 * sentence.
 */
function sentenceEnds(text: string): number[] {
  const ends: number[] = [];

  for (let index = 0; index < text.length; index += 1) {
    const char = text[index];
    if (char !== '.' && char !== '!' && char !== '?') {
      continue;
    }

    let end = index + 1;
    while (end < text.length && /["'”’)\]]/.test(text[end])) {
      end += 1;
    }

    if (end >= text.length) {
      continue;
    }

    const separated = /\s/.test(text[end]);
    // A period glued to the next word is usually an abbreviation, a domain
    // name or a typo; question and exclamation marks are safe to split.
    if (!separated && char === '.') {
      continue;
    }

    let next = end;
    while (next < text.length && /\s/.test(text[next])) {
      next += 1;
    }
    if (next >= text.length) {
      continue;
    }

    // A new sentence starts with a capital letter, a quote/bracket or Arabic
    // text. This also keeps decimals and lowercase continuations together.
    const startsSentence = /[A-Z"“(\[]/.test(text[next]) || /[\u0600-\u06FF]/.test(text[next]);
    if (!startsSentence) {
      continue;
    }

    if (char === '.' && isAbbreviation(text, index)) {
      continue;
    }

    ends.push(end);
    index = end - 1;
  }

  return ends;
}

export function formatTafsirText(text: string): string {
  if (text.includes('\n')) {
    return text;
  }

  const ends = sentenceEnds(text);
  if (ends.length < SENTENCES_PER_PARAGRAPH) {
    return text;
  }

  const paragraphs: string[] = [];
  let start = 0;

  for (let index = SENTENCES_PER_PARAGRAPH - 1; index < ends.length; index += SENTENCES_PER_PARAGRAPH) {
    const end = ends[index];
    const paragraph = text.slice(start, end).trim();
    if (paragraph) {
      paragraphs.push(paragraph);
    }

    start = end;
    while (start < text.length && /\s/.test(text[start])) {
      start += 1;
    }
  }

  const rest = text.slice(start).trim();
  if (rest) {
    paragraphs.push(rest);
  }

  return paragraphs.join('\n\n');
}
