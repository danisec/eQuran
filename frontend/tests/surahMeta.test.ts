import assert from 'node:assert/strict';
import test from 'node:test';

import { SURAH_META_EN } from '../src/lib/data/surahMeta.ts';

test('every surah has an English meaning and revelation place', () => {
  for (let nomor = 1; nomor <= 114; nomor += 1) {
    const meta = SURAH_META_EN[nomor];
    assert.ok(meta, `surah ${nomor} is missing metadata`);
    assert.ok(meta.meaning.trim().length > 0, `surah ${nomor} has an empty meaning`);
    assert.ok(
      meta.revelation === 'Meccan' || meta.revelation === 'Medinan',
      `surah ${nomor} has an unknown revelation place`
    );
  }
  assert.equal(Object.keys(SURAH_META_EN).length, 114);
});

test('known meanings match the published English translations', () => {
  assert.equal(SURAH_META_EN[1]?.meaning, 'The Opening');
  assert.equal(SURAH_META_EN[2]?.meaning, 'The Cow');
  assert.equal(SURAH_META_EN[114]?.meaning, 'Mankind');
});

test('the last two surahs are Meccan', () => {
  // equran.id's English endpoint labels both as Medinan; the standard division
  // (and quran.com) has them Meccan, so the static table is the source of truth.
  assert.equal(SURAH_META_EN[113]?.revelation, 'Meccan');
  assert.equal(SURAH_META_EN[114]?.revelation, 'Meccan');
});
