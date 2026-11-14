/**
 * Which grouping the home index shows, and the direction of the A–Z list.
 *
 * The home surface unmounts when the reader opens (`{#if viewState.view ===
 * 'home'}`), so component-local state would silently snap back to the first tab
 * on every return. Holding it here keeps the reader's place in the index, and
 * the localStorage copy keeps it across restarts, like the panel and view
 * state.
 */
export type IndexTab = 'surah' | 'juz' | 'revelation' | 'alphabet';

const STORAGE_KEY = 'equran-home-index';
const TABS: readonly IndexTab[] = ['surah', 'juz', 'revelation', 'alphabet'];

export const homeIndexState = $state({
  tab: 'surah' as IndexTab,
  /** A–Z runs ascending until the reader turns it around. */
  alphabetDesc: false
});

export function initHomeIndex() {
  const saved = localStorage.getItem(STORAGE_KEY);
  if (!saved) return;
  try {
    const { tab, alphabetDesc } = JSON.parse(saved);
    if (TABS.includes(tab)) homeIndexState.tab = tab;
    if (typeof alphabetDesc === 'boolean') homeIndexState.alphabetDesc = alphabetDesc;
  } catch {
    /* keep defaults */
  }
}

function persist() {
  localStorage.setItem(
    STORAGE_KEY,
    JSON.stringify({ tab: homeIndexState.tab, alphabetDesc: homeIndexState.alphabetDesc })
  );
}

export function setIndexTab(tab: IndexTab) {
  homeIndexState.tab = tab;
  persist();
}

export function toggleAlphabetDirection() {
  homeIndexState.alphabetDesc = !homeIndexState.alphabetDesc;
  persist();
}
