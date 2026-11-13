/**
 * Which surface the reader is looking at. The app has no router: it is one
 * window with two surfaces, so a small state machine is clearer than pulling in
 * SvelteKit routing for a prerendered static build.
 */
export type View = 'home' | 'reader';

const STORAGE_KEY = 'equran-view';

function initialView(): View {
  const saved = localStorage.getItem(STORAGE_KEY);
  return saved === 'reader' ? 'reader' : 'home';
}

export const viewState = $state({
  view: 'home' as View
});

export function initView() {
  viewState.view = initialView();
}

export function showHome() {
  viewState.view = 'home';
  localStorage.setItem(STORAGE_KEY, 'home');
}

export function showReader() {
  viewState.view = 'reader';
  localStorage.setItem(STORAGE_KEY, 'reader');
}
