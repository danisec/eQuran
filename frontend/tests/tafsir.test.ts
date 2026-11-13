import assert from 'node:assert/strict';
import { test } from 'node:test';

import { formatTafsirText } from '../src/lib/tafsir.ts';

/** Paragraph breaks are the only change; the words must stay identical. */
function words(text: string): string {
  return text.replace(/\s+/g, ' ').trim();
}

test('text that already has paragraphs is left alone', () => {
  const text = 'Paragraf pertama.\n\nParagraf kedua.';
  assert.equal(formatTafsirText(text), text);
});

test('a short entry without enough sentences is left alone', () => {
  const text = 'One sentence. Another sentence.';
  assert.equal(formatTafsirText(text), text);
});

test('long commentary is broken into paragraphs of three sentences', () => {
  const text = 'One. Two. Three. Four. Five. Six. Seven.';
  assert.equal(
    formatTafsirText(text),
    'One. Two. Three.\n\nFour. Five. Six.\n\nSeven.'
  );
});

test('abbreviations do not end a sentence', () => {
  const text = 'See e.g. the first case. It applies here. This is clear. Done.';
  assert.equal(
    formatTafsirText(text),
    'See e.g. the first case. It applies here. This is clear.\n\nDone.'
  );
});

test('initials and decimals stay inside their sentence', () => {
  const text = 'Dr. A. Smith wrote 2.5 pages. He stopped. We read. They left.';
  assert.equal(
    formatTafsirText(text),
    'Dr. A. Smith wrote 2.5 pages. He stopped. We read.\n\nThey left.'
  );
});

test('closing quotes stay with their sentence', () => {
  const text = 'He said "stop." Then left. We stayed. It rained.';
  assert.equal(
    formatTafsirText(text),
    'He said "stop." Then left. We stayed.\n\nIt rained.'
  );
});

test('a question mark glued to the next word still counts as a sentence end', () => {
  const text = 'One. Two. Three?Four. Five. Six. Seven.';
  assert.equal(
    formatTafsirText(text),
    'One. Two. Three?\n\nFour. Five. Six.\n\nSeven.'
  );
});

test('the words are preserved exactly', () => {
  const text =
    'The first point is long and detailed. The second follows it closely. ' +
    'The third closes the argument. The fourth opens a new topic. ' +
    'The fifth finishes it.';
  const formatted = formatTafsirText(text);

  assert.ok(formatted.includes('\n\n'));
  assert.equal(words(formatted), words(text));
});
