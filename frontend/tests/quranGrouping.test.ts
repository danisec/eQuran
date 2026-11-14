import assert from 'node:assert/strict';
import test from 'node:test';

import { JUZ_RANGES, REVELATION_ORDER, partVerses, revelationOf } from '../src/lib/data/quranGrouping.ts';

test('the order of revelation is a permutation of 1..114', () => {
  const order = Object.values(REVELATION_ORDER).sort((a, b) => a - b);
  assert.equal(order.length, 114);
  for (let position = 1; position <= 114; position += 1) {
    assert.equal(order[position - 1], position, `position ${position} missing`);
  }
});

test('known revelation positions match the standard Egyptian order', () => {
  assert.equal(revelationOf(96), 1); // Al-'Alaq
  assert.equal(revelationOf(68), 2); // Al-Qalam
  assert.equal(revelationOf(1), 5); // Al-Fatihah
  assert.equal(revelationOf(114), 21); // An-Nas
  assert.equal(revelationOf(110), 114); // An-Nasr, the last revealed
});

test('every surah has a revelation position', () => {
  for (let nomor = 1; nomor <= 114; nomor += 1) {
    assert.ok(revelationOf(nomor) >= 1 && revelationOf(nomor) <= 114);
  }
});

test('there are 30 juz numbered 1..30 in order', () => {
  assert.equal(JUZ_RANGES.length, 30);
  JUZ_RANGES.forEach((range, index) => {
    assert.equal(range.juz, index + 1);
  });
});

test('juz ranges are contiguous from 1:1 to 114:6', () => {
  const first = JUZ_RANGES[0];
  const last = JUZ_RANGES[JUZ_RANGES.length - 1];
  assert.equal(first.startSurah, 1);
  assert.equal(first.startAyah, 1);
  assert.equal(last.endSurah, 114);
  assert.equal(last.endAyah, 6);

  for (let index = 0; index < JUZ_RANGES.length - 1; index += 1) {
    const current = JUZ_RANGES[index];
    const next = JUZ_RANGES[index + 1];
    if (next.startSurah === current.endSurah) {
      assert.equal(
        next.startAyah,
        current.endAyah + 1,
        `juz ${current.juz} ends ${current.endSurah}:${current.endAyah} but juz ${next.juz} starts ${next.startSurah}:${next.startAyah}`
      );
    } else {
      assert.ok(
        next.startSurah > current.endSurah,
        `juz ${next.juz} starts before juz ${current.juz} ends`
      );
      assert.equal(next.startAyah, 1, `juz ${next.juz} should start at ayah 1`);
    }
  }
});

test('each juz lists the surahs it actually spans', () => {
  for (const range of JUZ_RANGES) {
    const parts = range.parts;
    assert.equal(parts[0].surah, range.startSurah, `juz ${range.juz} first surah`);
    assert.equal(parts[0].from, range.startAyah, `juz ${range.juz} first ayah`);
    assert.equal(parts[parts.length - 1].surah, range.endSurah, `juz ${range.juz} last surah`);
    assert.equal(parts[parts.length - 1].to, range.endAyah, `juz ${range.juz} last ayah`);
    assert.equal(new Set(parts.map((p) => p.surah)).size, parts.length, `juz ${range.juz} repeats a surah`);
    for (let index = 1; index < parts.length; index += 1) {
      assert.equal(parts[index].surah, parts[index - 1].surah + 1, `juz ${range.juz} skips a surah`);
      assert.equal(parts[index].from, 1, `juz ${range.juz} continues a surah at ayah 1`);
    }
  }
});

test('juz ayat counts add up to the whole book', () => {
  const total = JUZ_RANGES.reduce((sum, range) => sum + range.verses, 0);
  assert.equal(total, 6236);
  for (const range of JUZ_RANGES) {
    const summed = range.parts.reduce((sum, part) => sum + partVerses(part), 0);
    assert.equal(range.verses, summed, `juz ${range.juz} count does not match its parts`);
  }
});

test('parts name the real surah ranges from the mushaf', () => {
  const juz1 = JUZ_RANGES[0];
  assert.deepEqual(juz1.parts, [
    { surah: 1, from: 1, to: 7 },
    { surah: 2, from: 1, to: 141 }
  ]);
  const juz30 = JUZ_RANGES[JUZ_RANGES.length - 1];
  assert.equal(juz30.parts[0].surah, 78);
  assert.equal(juz30.verses, 564);
  assert.equal(juz30.parts.length, 37);
});
