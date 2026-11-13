const STORAGE_KEY = 'equran-panels';

/**
 * Above this many rendered verses the rail tween is skipped.
 *
 * Collapsing the left rail animates its grid track (280px <-> 64px). Every frame
 * of that animation changes the width available to the reader, so the browser
 * re-runs layout for every verse card on screen. Measured on the production
 * build (rail edge sampled at 120fps), the tween passes through:
 *
 *     Al-Fatihah   7 ayahs   11 layout steps   smooth
 *     Al-Baqarah 286 ayahs    4 layout steps   visible stutter
 *
 * The cost grows with the rendered verse count exactly as the theme reveal's
 * does (`MAX_VERSES_FOR_REVEAL` in theme.svelte.ts), so the same answer applies:
 * past this point the collapse snaps instead of tweening. Beranda and Mode
 * fokus render no verse wrappers and always stay smooth.
 */
const MAX_VERSES_FOR_RAIL_TWEEN = 80;

/**
 * Shell panel state.
 *
 * Settings used to be a permanent 320px column, which cost the reading column
 * a fifth of the window on a narrow screen for five controls that are set once.
 * They are now a modal. The sidebar is the navigation rail on the left; the
 * marks rail (the reader's own verses) is its own panel on the right.
 */
export const panelState = $state({
  sidebarOpen: true,
  marksOpen: false,
  settingsOpen: false
});

export function initPanels() {
  const saved = localStorage.getItem(STORAGE_KEY);
  if (!saved) return;
  try {
    const { sidebarOpen, marksOpen } = JSON.parse(saved);
    if (typeof sidebarOpen === 'boolean') panelState.sidebarOpen = sidebarOpen;
    if (typeof marksOpen === 'boolean') panelState.marksOpen = marksOpen;
  } catch {
    /* keep defaults */
  }
}

function persist() {
  localStorage.setItem(
    STORAGE_KEY,
    JSON.stringify({ sidebarOpen: panelState.sidebarOpen, marksOpen: panelState.marksOpen })
  );
}

/**
 * Whether the shell is light enough to tween the rail width.
 *
 * The reader tags every verse wrapper with `data-ayah`, so counting them is a
 * direct measure of what the tween would have to re-lay-out each frame. It is
 * the same query the theme reveal uses, on the same nodes. Beranda and Mode
 * fokus render no wrappers and always come back light.
 */
function railTweenIsAffordable(): boolean {
  return document.querySelectorAll('[data-ayah]').length <= MAX_VERSES_FOR_RAIL_TWEEN;
}

/**
 * Pending timer that lifts the snap freeze. Held at module scope so a second
 * toggle cancels the first one's release instead of racing it.
 */
let railReleaseTimer: ReturnType<typeof setTimeout> | undefined;

export function toggleSidebar() {
  // On a heavy surah, tweening the track reflows every card each frame and
  // stutters (see `MAX_VERSES_FOR_RAIL_TWEEN`). Freeze the track transition for
  // one task so the change lands in a single, instant layout pass instead.
  if (!railTweenIsAffordable()) {
    document.documentElement.classList.add('rail-snapping');
    if (railReleaseTimer !== undefined) clearTimeout(railReleaseTimer);
    railReleaseTimer = setTimeout(() => {
      document.documentElement.classList.remove('rail-snapping');
      railReleaseTimer = undefined;
    }, 0);
  }
  panelState.sidebarOpen = !panelState.sidebarOpen;
  persist();
}

export function toggleMarks() {
  panelState.marksOpen = !panelState.marksOpen;
  persist();
}

/**
 * Reveal the marks rail. The count button in the reader header is a way in, not
 * a switch, so it only ever opens — closing is the panel's own control.
 */
export function openMarks() {
  if (panelState.marksOpen) return;
  panelState.marksOpen = true;
  persist();
}

export function openSettings() {
  panelState.settingsOpen = true;
}

export function closeSettings() {
  panelState.settingsOpen = false;
}

export function toggleSettings() {
  if (panelState.settingsOpen) {
    closeSettings();
  } else {
    openSettings();
  }
}
