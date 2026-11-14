/**
 * Surah groupings the API does not carry.
 *
 * equran.id publishes a surah's number, name, meaning and place of revelation
 * only. Juz boundaries and the order of revelation are fixed, publicly agreed
 * tables, so they live here as data: the index works offline and no extra
 * request runs before the first paint.
 *
 * - `REVELATION_ORDER` follows the common Egyptian (Cairo) order, matching
 *   quran.com's `revelation_order`: Al-'Alaq is 1 and An-Nasr is 114. Scholarly
 *   traditions differ slightly; this is the one most Indonesian mushafs print.
 * - `JUZ_RANGES` mirrors quran.com's 30 juz. They are contiguous from 1:1 to
 *   114:6, which the unit tests assert. Each juz keeps `parts`, the slice of
 *   every surah it covers, so the index can show "Al-Baqarah 1-141" without
 *   fetching anything.
 */
export type JuzPart = {
  /** Surah number. */
  surah: number;
  /** First ayah of this surah inside the juz. */
  from: number;
  /** Last ayah of this surah inside the juz. */
  to: number;
};

export type JuzRange = {
  juz: number;
  /** Ayat in the juz, the sum of every part. */
  verses: number;
  startSurah: number;
  startAyah: number;
  endSurah: number;
  endAyah: number;
  /** Which surahs the juz touches, and how much of each, in mushaf order. */
  parts: JuzPart[];
};

/** Surah number to its position in the order of revelation (1 = first). */
export const REVELATION_ORDER: Record<number, number> = {
  1: 5,
  2: 87,
  3: 89,
  4: 92,
  5: 112,
  6: 55,
  7: 39,
  8: 88,
  9: 113,
  10: 51,
  11: 52,
  12: 53,
  13: 96,
  14: 72,
  15: 54,
  16: 70,
  17: 50,
  18: 69,
  19: 44,
  20: 45,
  21: 73,
  22: 103,
  23: 74,
  24: 102,
  25: 42,
  26: 47,
  27: 48,
  28: 49,
  29: 85,
  30: 84,
  31: 57,
  32: 75,
  33: 90,
  34: 58,
  35: 43,
  36: 41,
  37: 56,
  38: 38,
  39: 59,
  40: 60,
  41: 61,
  42: 62,
  43: 63,
  44: 64,
  45: 65,
  46: 66,
  47: 95,
  48: 111,
  49: 106,
  50: 34,
  51: 67,
  52: 76,
  53: 23,
  54: 37,
  55: 97,
  56: 46,
  57: 94,
  58: 105,
  59: 101,
  60: 91,
  61: 109,
  62: 110,
  63: 104,
  64: 108,
  65: 99,
  66: 107,
  67: 77,
  68: 2,
  69: 78,
  70: 79,
  71: 71,
  72: 40,
  73: 3,
  74: 4,
  75: 31,
  76: 98,
  77: 33,
  78: 80,
  79: 81,
  80: 24,
  81: 7,
  82: 82,
  83: 86,
  84: 83,
  85: 27,
  86: 36,
  87: 8,
  88: 68,
  89: 10,
  90: 35,
  91: 26,
  92: 9,
  93: 11,
  94: 12,
  95: 28,
  96: 1,
  97: 25,
  98: 100,
  99: 93,
  100: 14,
  101: 30,
  102: 16,
  103: 13,
  104: 32,
  105: 19,
  106: 29,
  107: 17,
  108: 15,
  109: 18,
  110: 114,
  111: 6,
  112: 22,
  113: 20,
  114: 21,
};

/** The 30 juz, in order, with the range each one covers. */
export const JUZ_RANGES: JuzRange[] = [
  {
    juz: 1,
    verses: 148,
    startSurah: 1,
    startAyah: 1,
    endSurah: 2,
    endAyah: 141,
    parts: [
      { surah: 1, from: 1, to: 7 },
      { surah: 2, from: 1, to: 141 }
    ]
  },
  {
    juz: 2,
    verses: 111,
    startSurah: 2,
    startAyah: 142,
    endSurah: 2,
    endAyah: 252,
    parts: [
      { surah: 2, from: 142, to: 252 }
    ]
  },
  {
    juz: 3,
    verses: 126,
    startSurah: 2,
    startAyah: 253,
    endSurah: 3,
    endAyah: 92,
    parts: [
      { surah: 2, from: 253, to: 286 },
      { surah: 3, from: 1, to: 92 }
    ]
  },
  {
    juz: 4,
    verses: 131,
    startSurah: 3,
    startAyah: 93,
    endSurah: 4,
    endAyah: 23,
    parts: [
      { surah: 3, from: 93, to: 200 },
      { surah: 4, from: 1, to: 23 }
    ]
  },
  {
    juz: 5,
    verses: 124,
    startSurah: 4,
    startAyah: 24,
    endSurah: 4,
    endAyah: 147,
    parts: [
      { surah: 4, from: 24, to: 147 }
    ]
  },
  {
    juz: 6,
    verses: 110,
    startSurah: 4,
    startAyah: 148,
    endSurah: 5,
    endAyah: 81,
    parts: [
      { surah: 4, from: 148, to: 176 },
      { surah: 5, from: 1, to: 81 }
    ]
  },
  {
    juz: 7,
    verses: 149,
    startSurah: 5,
    startAyah: 82,
    endSurah: 6,
    endAyah: 110,
    parts: [
      { surah: 5, from: 82, to: 120 },
      { surah: 6, from: 1, to: 110 }
    ]
  },
  {
    juz: 8,
    verses: 142,
    startSurah: 6,
    startAyah: 111,
    endSurah: 7,
    endAyah: 87,
    parts: [
      { surah: 6, from: 111, to: 165 },
      { surah: 7, from: 1, to: 87 }
    ]
  },
  {
    juz: 9,
    verses: 159,
    startSurah: 7,
    startAyah: 88,
    endSurah: 8,
    endAyah: 40,
    parts: [
      { surah: 7, from: 88, to: 206 },
      { surah: 8, from: 1, to: 40 }
    ]
  },
  {
    juz: 10,
    verses: 127,
    startSurah: 8,
    startAyah: 41,
    endSurah: 9,
    endAyah: 92,
    parts: [
      { surah: 8, from: 41, to: 75 },
      { surah: 9, from: 1, to: 92 }
    ]
  },
  {
    juz: 11,
    verses: 151,
    startSurah: 9,
    startAyah: 93,
    endSurah: 11,
    endAyah: 5,
    parts: [
      { surah: 9, from: 93, to: 129 },
      { surah: 10, from: 1, to: 109 },
      { surah: 11, from: 1, to: 5 }
    ]
  },
  {
    juz: 12,
    verses: 170,
    startSurah: 11,
    startAyah: 6,
    endSurah: 12,
    endAyah: 52,
    parts: [
      { surah: 11, from: 6, to: 123 },
      { surah: 12, from: 1, to: 52 }
    ]
  },
  {
    juz: 13,
    verses: 154,
    startSurah: 12,
    startAyah: 53,
    endSurah: 14,
    endAyah: 52,
    parts: [
      { surah: 12, from: 53, to: 111 },
      { surah: 13, from: 1, to: 43 },
      { surah: 14, from: 1, to: 52 }
    ]
  },
  {
    juz: 14,
    verses: 227,
    startSurah: 15,
    startAyah: 1,
    endSurah: 16,
    endAyah: 128,
    parts: [
      { surah: 15, from: 1, to: 99 },
      { surah: 16, from: 1, to: 128 }
    ]
  },
  {
    juz: 15,
    verses: 185,
    startSurah: 17,
    startAyah: 1,
    endSurah: 18,
    endAyah: 74,
    parts: [
      { surah: 17, from: 1, to: 111 },
      { surah: 18, from: 1, to: 74 }
    ]
  },
  {
    juz: 16,
    verses: 269,
    startSurah: 18,
    startAyah: 75,
    endSurah: 20,
    endAyah: 135,
    parts: [
      { surah: 18, from: 75, to: 110 },
      { surah: 19, from: 1, to: 98 },
      { surah: 20, from: 1, to: 135 }
    ]
  },
  {
    juz: 17,
    verses: 190,
    startSurah: 21,
    startAyah: 1,
    endSurah: 22,
    endAyah: 78,
    parts: [
      { surah: 21, from: 1, to: 112 },
      { surah: 22, from: 1, to: 78 }
    ]
  },
  {
    juz: 18,
    verses: 202,
    startSurah: 23,
    startAyah: 1,
    endSurah: 25,
    endAyah: 20,
    parts: [
      { surah: 23, from: 1, to: 118 },
      { surah: 24, from: 1, to: 64 },
      { surah: 25, from: 1, to: 20 }
    ]
  },
  {
    juz: 19,
    verses: 339,
    startSurah: 25,
    startAyah: 21,
    endSurah: 27,
    endAyah: 55,
    parts: [
      { surah: 25, from: 21, to: 77 },
      { surah: 26, from: 1, to: 227 },
      { surah: 27, from: 1, to: 55 }
    ]
  },
  {
    juz: 20,
    verses: 171,
    startSurah: 27,
    startAyah: 56,
    endSurah: 29,
    endAyah: 45,
    parts: [
      { surah: 27, from: 56, to: 93 },
      { surah: 28, from: 1, to: 88 },
      { surah: 29, from: 1, to: 45 }
    ]
  },
  {
    juz: 21,
    verses: 178,
    startSurah: 29,
    startAyah: 46,
    endSurah: 33,
    endAyah: 30,
    parts: [
      { surah: 29, from: 46, to: 69 },
      { surah: 30, from: 1, to: 60 },
      { surah: 31, from: 1, to: 34 },
      { surah: 32, from: 1, to: 30 },
      { surah: 33, from: 1, to: 30 }
    ]
  },
  {
    juz: 22,
    verses: 169,
    startSurah: 33,
    startAyah: 31,
    endSurah: 36,
    endAyah: 27,
    parts: [
      { surah: 33, from: 31, to: 73 },
      { surah: 34, from: 1, to: 54 },
      { surah: 35, from: 1, to: 45 },
      { surah: 36, from: 1, to: 27 }
    ]
  },
  {
    juz: 23,
    verses: 357,
    startSurah: 36,
    startAyah: 28,
    endSurah: 39,
    endAyah: 31,
    parts: [
      { surah: 36, from: 28, to: 83 },
      { surah: 37, from: 1, to: 182 },
      { surah: 38, from: 1, to: 88 },
      { surah: 39, from: 1, to: 31 }
    ]
  },
  {
    juz: 24,
    verses: 175,
    startSurah: 39,
    startAyah: 32,
    endSurah: 41,
    endAyah: 46,
    parts: [
      { surah: 39, from: 32, to: 75 },
      { surah: 40, from: 1, to: 85 },
      { surah: 41, from: 1, to: 46 }
    ]
  },
  {
    juz: 25,
    verses: 246,
    startSurah: 41,
    startAyah: 47,
    endSurah: 45,
    endAyah: 37,
    parts: [
      { surah: 41, from: 47, to: 54 },
      { surah: 42, from: 1, to: 53 },
      { surah: 43, from: 1, to: 89 },
      { surah: 44, from: 1, to: 59 },
      { surah: 45, from: 1, to: 37 }
    ]
  },
  {
    juz: 26,
    verses: 195,
    startSurah: 46,
    startAyah: 1,
    endSurah: 51,
    endAyah: 30,
    parts: [
      { surah: 46, from: 1, to: 35 },
      { surah: 47, from: 1, to: 38 },
      { surah: 48, from: 1, to: 29 },
      { surah: 49, from: 1, to: 18 },
      { surah: 50, from: 1, to: 45 },
      { surah: 51, from: 1, to: 30 }
    ]
  },
  {
    juz: 27,
    verses: 399,
    startSurah: 51,
    startAyah: 31,
    endSurah: 57,
    endAyah: 29,
    parts: [
      { surah: 51, from: 31, to: 60 },
      { surah: 52, from: 1, to: 49 },
      { surah: 53, from: 1, to: 62 },
      { surah: 54, from: 1, to: 55 },
      { surah: 55, from: 1, to: 78 },
      { surah: 56, from: 1, to: 96 },
      { surah: 57, from: 1, to: 29 }
    ]
  },
  {
    juz: 28,
    verses: 137,
    startSurah: 58,
    startAyah: 1,
    endSurah: 66,
    endAyah: 12,
    parts: [
      { surah: 58, from: 1, to: 22 },
      { surah: 59, from: 1, to: 24 },
      { surah: 60, from: 1, to: 13 },
      { surah: 61, from: 1, to: 14 },
      { surah: 62, from: 1, to: 11 },
      { surah: 63, from: 1, to: 11 },
      { surah: 64, from: 1, to: 18 },
      { surah: 65, from: 1, to: 12 },
      { surah: 66, from: 1, to: 12 }
    ]
  },
  {
    juz: 29,
    verses: 431,
    startSurah: 67,
    startAyah: 1,
    endSurah: 77,
    endAyah: 50,
    parts: [
      { surah: 67, from: 1, to: 30 },
      { surah: 68, from: 1, to: 52 },
      { surah: 69, from: 1, to: 52 },
      { surah: 70, from: 1, to: 44 },
      { surah: 71, from: 1, to: 28 },
      { surah: 72, from: 1, to: 28 },
      { surah: 73, from: 1, to: 20 },
      { surah: 74, from: 1, to: 56 },
      { surah: 75, from: 1, to: 40 },
      { surah: 76, from: 1, to: 31 },
      { surah: 77, from: 1, to: 50 }
    ]
  },
  {
    juz: 30,
    verses: 564,
    startSurah: 78,
    startAyah: 1,
    endSurah: 114,
    endAyah: 6,
    parts: [
      { surah: 78, from: 1, to: 40 },
      { surah: 79, from: 1, to: 46 },
      { surah: 80, from: 1, to: 42 },
      { surah: 81, from: 1, to: 29 },
      { surah: 82, from: 1, to: 19 },
      { surah: 83, from: 1, to: 36 },
      { surah: 84, from: 1, to: 25 },
      { surah: 85, from: 1, to: 22 },
      { surah: 86, from: 1, to: 17 },
      { surah: 87, from: 1, to: 19 },
      { surah: 88, from: 1, to: 26 },
      { surah: 89, from: 1, to: 30 },
      { surah: 90, from: 1, to: 20 },
      { surah: 91, from: 1, to: 15 },
      { surah: 92, from: 1, to: 21 },
      { surah: 93, from: 1, to: 11 },
      { surah: 94, from: 1, to: 8 },
      { surah: 95, from: 1, to: 8 },
      { surah: 96, from: 1, to: 19 },
      { surah: 97, from: 1, to: 5 },
      { surah: 98, from: 1, to: 8 },
      { surah: 99, from: 1, to: 8 },
      { surah: 100, from: 1, to: 11 },
      { surah: 101, from: 1, to: 11 },
      { surah: 102, from: 1, to: 8 },
      { surah: 103, from: 1, to: 3 },
      { surah: 104, from: 1, to: 9 },
      { surah: 105, from: 1, to: 5 },
      { surah: 106, from: 1, to: 4 },
      { surah: 107, from: 1, to: 7 },
      { surah: 108, from: 1, to: 3 },
      { surah: 109, from: 1, to: 6 },
      { surah: 110, from: 1, to: 3 },
      { surah: 111, from: 1, to: 5 },
      { surah: 112, from: 1, to: 4 },
      { surah: 113, from: 1, to: 5 },
      { surah: 114, from: 1, to: 6 }
    ]
  },
];

/** Position of a surah in the order of revelation. */
export function revelationOf(surahNumber: number): number {
  return REVELATION_ORDER[surahNumber] ?? surahNumber;
}

/** Ayat a juz part covers. */
export function partVerses(part: JuzPart): number {
  return part.to - part.from + 1;
}
