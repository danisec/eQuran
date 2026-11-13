import { playbackState, togglePlayback, nextAyah, prevAyah } from '$lib/stores/playback.svelte';
import { surahState } from '$lib/stores/surah.svelte';
import { disableFocusMode, focusState } from '$lib/stores/focus.svelte';
import { openSettings, toggleMarks, toggleSidebar } from '$lib/stores/panels.svelte';
import { showHome } from '$lib/stores/view.svelte';

export function handleGlobalKeydown(event: KeyboardEvent) {
  const target = event.target as HTMLElement;
  const isInput = target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.tagName === 'SELECT';

  if (event.key === 'Escape' && focusState.enabled) {
    event.preventDefault();
    disableFocusMode();
    return;
  }

  if (event.key === ' ' && !isInput) {
    event.preventDefault();
    togglePlayback(surahState.selected.nomor, surahState.selected.jumlahAyat);
  }

  if ((event.metaKey || event.ctrlKey) && event.key === 'ArrowRight') {
    event.preventDefault();
    nextAyah(surahState.selected.nomor, surahState.selected.jumlahAyat);
  }

  if ((event.metaKey || event.ctrlKey) && event.key === 'ArrowLeft') {
    event.preventDefault();
    prevAyah(surahState.selected.nomor, surahState.selected.jumlahAyat);
  }

  if ((event.metaKey || event.ctrlKey) && (event.key === 'b' || event.key === 'B') && !isInput) {
    event.preventDefault();
    toggleSidebar();
  }

  // Cmd/Ctrl+M for the reader's own marks panel
  if ((event.metaKey || event.ctrlKey) && (event.key === 'm' || event.key === 'M')) {
    event.preventDefault();
    toggleMarks();
  }

  // Cmd/Ctrl+, is the platform convention for preferences
  if ((event.metaKey || event.ctrlKey) && event.key === ',') {
    event.preventDefault();
    openSettings();
  }
}
