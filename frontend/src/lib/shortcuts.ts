import { playbackState, togglePlayback, nextAyah, prevAyah } from '$lib/stores/playback.svelte';
import { surahState } from '$lib/stores/surah.svelte';
import { disableFocusMode, focusState } from '$lib/stores/focus.svelte';

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
    togglePlayback(surahState.selected.nomor);
  }

  if ((event.metaKey || event.ctrlKey) && event.key === 'ArrowRight') {
    event.preventDefault();
    nextAyah(surahState.selected.nomor, surahState.selected.jumlahAyat);
  }

  if ((event.metaKey || event.ctrlKey) && event.key === 'ArrowLeft') {
    event.preventDefault();
    prevAyah(surahState.selected.nomor, surahState.selected.jumlahAyat);
  }
}
