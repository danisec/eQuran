type ThemeMode = 'light' | 'dark';

type Origin = { x: number; y: number };

const STORAGE_KEY = 'equran-theme';

/** Matches the reveal animation below; kept together so they cannot drift. */
const REVEAL_DURATION = 420;
const REVEAL_EASING = 'cubic-bezier(0.22, 1, 0.36, 1)';

/**
 * Above this many rendered verses the circular reveal is skipped.
 *
 * Choosing the reveal means snapshotting the whole page twice, and that cost
 * grows with the number of verse cards on screen. Measured on the production
 * build, the whole swap (click to settled) takes:
 *
 *     Al-Fatihah   7 ayahs   ~530 ms
 *     Al-Mulk     30 ayahs   ~730 ms
 *     Ar-Ra'd     43 ayahs   ~875 ms
 *     Yasin       83 ayahs  ~1030 ms
 *     Maryam      98 ayahs  ~1230 ms
 *     Al-Kahf    110 ayahs  ~1510 ms
 *
 * The animation itself stays on the compositor and never drops a frame; what
 * grows is the pause before it starts, while the snapshots are rasterised. An
 * instant swap is ~250 ms whatever the surah, so past this point the reveal
 * costs more than it gives. The line is drawn where the interaction is still
 * around a second, which keeps it for the short surahs most reading happens in.
 */
const MAX_VERSES_FOR_REVEAL = 80;

export const themeState = $state({
  mode: 'light' as ThemeMode
});

/**
 * Pending timer that lifts the transition freeze.
 *
 * Held at module scope so a second toggle cancels the first one's release
 * instead of leaving two timers racing to remove the class.
 */
let releaseTimer: ReturnType<typeof setTimeout> | undefined;

function preferredTheme(): ThemeMode {
  const savedTheme = localStorage.getItem(STORAGE_KEY);
  if (savedTheme === 'dark' || savedTheme === 'light') {
    return savedTheme;
  }

  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

/** Applies the class and the native colour scheme; no animation of its own. */
function commit(nextTheme: ThemeMode) {
  themeState.mode = nextTheme;
  document.documentElement.classList.toggle('dark', nextTheme === 'dark');
  document.documentElement.style.colorScheme = nextTheme;
  localStorage.setItem(STORAGE_KEY, nextTheme);
}

/**
 * Freezes per-element transitions while the theme flips.
 *
 * Every bordered or surfaced element carries its own `transition`, so adding
 * `.dark` would otherwise start one animation per element: over a thousand on a
 * surah like Al-Baqarah, all fighting the reveal for the main thread. Freezing
 * them turns the swap into a single style recalculation.
 */
function freezeTransitions() {
  if (releaseTimer !== undefined) {
    clearTimeout(releaseTimer);
    releaseTimer = undefined;
  }
  document.documentElement.classList.add('theme-swapping');
}

/**
 * Re-enables transitions after `delay` ms. Always called, because a stalled
 * snapshot must never leave the interface with every animation disabled.
 */
function releaseTransitions(delay: number) {
  if (releaseTimer !== undefined) clearTimeout(releaseTimer);
  releaseTimer = setTimeout(() => {
    document.documentElement.classList.remove('theme-swapping');
    releaseTimer = undefined;
  }, delay);
}

export function applyTheme(nextTheme: ThemeMode) {
  commit(nextTheme);
}

/**
 * Whether the current surface is light enough to snapshot for the reveal.
 *
 * The reader tags every verse wrapper with `data-ayah`, so counting them is a
 * direct measure of what the view transition would have to rasterise. Beranda
 * and Mode fokus render no such wrappers and always come back light. Nothing is
 * cached: the count is one cheap query, and a surah can be swapped while the
 * theme control stays mounted.
 */
function revealIsAffordable(): boolean {
  return document.querySelectorAll('[data-ayah]').length <= MAX_VERSES_FOR_REVEAL;
}

export function initTheme() {
  // No reveal on startup: the theme is simply already correct when the window
  // appears, and animating it would look like a glitch during launch.
  commit(preferredTheme());
}

/**
 * Reveals the new theme as a circle growing from the control that asked for it.
 *
 * The action happens at one point, so the change should come from that point;
 * a cross-fade everywhere reads as a flicker with no cause. Per-element
 * transitions are frozen for the duration (see `freezeTransitions`), and the
 * swap is always released even if the snapshot stalls.
 *
 * The reveal is skipped — leaving an instant swap — on surfaces too heavy to
 * snapshot smoothly (`revealIsAffordable`), in engines without the View
 * Transition API, and for readers who ask for less motion.
 */
export function toggleTheme(origin?: Origin) {
  const nextTheme: ThemeMode = themeState.mode === 'dark' ? 'light' : 'dark';
  const reduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  // `startViewTransition` is not in every engine this app can run on.
  const startViewTransition = (
    document as Document & {
      startViewTransition?: (callback: () => void) => { ready: Promise<void>; finished: Promise<void> };
    }
  ).startViewTransition;

  // Freeze before committing: flipping the class itself must not animate.
  freezeTransitions();

  if (
    reduced ||
    typeof startViewTransition !== 'function' ||
    !origin ||
    !revealIsAffordable()
  ) {
    commit(nextTheme);
    // Release on the next task so the frozen frame can paint first.
    releaseTransitions(0);
    return;
  }

  let transition: { ready: Promise<void>; finished: Promise<void> };
  try {
    transition = startViewTransition.call(document, () => commit(nextTheme));
  } catch {
    // A snapshot failed to start; fall back to an instant, frozen swap.
    commit(nextTheme);
    releaseTransitions(0);
    return;
  }

  // Belt and braces: release when the transition reports done, and after a
  // generous timeout in case `finished` never settles.
  anchorFinished(transition);
  releaseTransitions(REVEAL_DURATION + 800);

  void transition.ready
    .then(() => {
      const { x, y } = origin;
      const radius = Math.hypot(Math.max(x, innerWidth - x), Math.max(y, innerHeight - y));
      const clipPath = [
        `circle(0px at ${x}px ${y}px)`,
        `circle(${radius}px at ${x}px ${y}px)`
      ];

      // The root snapshot is the whole window; growing it from the origin is
      // what makes the reveal read as one surface changing, not many repainting.
      document.documentElement.animate(
        { clipPath },
        {
          duration: REVEAL_DURATION,
          easing: REVEAL_EASING,
          pseudoElement: '::view-transition-new(root)'
        }
      );
    })
    .catch(() => {
      // Snapshot failed; the theme is already committed, so do nothing.
    });
}

function anchorFinished(transition: { finished: Promise<void> }) {
  void transition.finished
    .catch(() => {})
    .finally(() => {
      releaseTransitions(0);
    });
}
