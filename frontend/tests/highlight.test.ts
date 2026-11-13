import assert from 'node:assert/strict';
import { test } from 'node:test';

import { highlightSegments } from '../src/lib/highlight.ts';

/** The rendered segments must reproduce the snippet byte for byte. */
function joined(text: string, query: string): string {
  return highlightSegments(text, query)
    .map((segment) => segment.text)
    .join('');
}

test('an empty query is one plain segment', () => {
  assert.deepEqual(highlightSegments('Rahman and Rahim', '   '), [
    { text: 'Rahman and Rahim', match: false }
  ]);
});

test('single-character query words are ignored', () => {
  assert.deepEqual(highlightSegments('a b c', 'a b'), [
    { text: 'a b c', match: false }
  ]);
});

test('matches case-insensitively without changing the snippet text', () => {
  assert.deepEqual(highlightSegments('Bismillāhir-raḥmān', 'RAḤMĀN'), [
    { text: 'Bismillāhir-', match: false },
    { text: 'raḥmān', match: true }
  ]);
});

test('matches are emitted in snippet order, not query order', () => {
  assert.deepEqual(highlightSegments('beta alpha', 'alpha beta'), [
    { text: 'beta', match: true },
    { text: ' ', match: false },
    { text: 'alpha', match: true }
  ]);
});

test('the longest word wins when several start at the same offset', () => {
  assert.deepEqual(highlightSegments('rahman', 'rah rahman'), [
    { text: 'rahman', match: true }
  ]);
});

test('text after the last match is kept as a plain segment', () => {
  assert.deepEqual(highlightSegments('the merciful lord', 'merciful'), [
    { text: 'the ', match: false },
    { text: 'merciful', match: true },
    { text: ' lord', match: false }
  ]);
});

test('no match returns the whole snippet as plain text', () => {
  assert.deepEqual(highlightSegments('Al-Fatihah', 'yasin'), [
    { text: 'Al-Fatihah', match: false }
  ]);
});

test('arabic snippets are matched without normalisation loss', () => {
  const text = 'بِسْمِ اللَّهِ الرَّحْمَٰنِ';
  assert.equal(joined(text, 'الرحمن'), text);
});

test('segments always reconstruct the snippet exactly', () => {
  const samples: Array<[string, string]> = [
    ['Rahman and Rahim', 'rahman'],
    ['beta alpha beta', 'beta alpha'],
    ['', 'anything'],
    ['one', 'one two'],
    ['---', 'a'],
    ['Ṣirāṭ al-mustaqīm', 'ṣirāṭ mustaqīm']
  ];

  for (const [text, query] of samples) {
    assert.equal(joined(text, query), text, `reconstruction failed for ${JSON.stringify(text)}`);
  }
});
