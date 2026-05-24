import { getCurrentWindow } from '@tauri-apps/api/window';

export const focusState = $state({
  enabled: false,
  selectedAyah: 1
});

async function setWindowFullscreen(fullscreen: boolean) {
  const currentWindow = getCurrentWindow();

  try {
    await currentWindow.setFullscreen(fullscreen);
    return;
  } catch (fullscreenError) {
    console.warn('Unable to update Tauri fullscreen state, trying simple fullscreen', fullscreenError);
  }

  try {
    await currentWindow.setSimpleFullscreen(fullscreen);
  } catch (simpleFullscreenError) {
    console.warn('Unable to update Tauri simple fullscreen state', simpleFullscreenError);
  }
}

export function enableFocusMode() {
  focusState.enabled = true;
  void setWindowFullscreen(true);
}

export function disableFocusMode() {
  focusState.enabled = false;
  void setWindowFullscreen(false);
}

export function toggleFocusMode() {
  if (focusState.enabled) {
    disableFocusMode();
  } else {
    enableFocusMode();
  }
}

export function setFocusAyah(ayah: number) {
  focusState.selectedAyah = ayah;
}
