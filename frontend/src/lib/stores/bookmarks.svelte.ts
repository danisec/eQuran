import { invoke } from '@tauri-apps/api/core';

export type Bookmark = {
  surah: number;
  ayah: number;
  surahName: string;
  ayahText: string;
};

export const bookmarkState = $state({
  list: [] as Bookmark[],
  loading: false
});

export async function loadBookmarks() {
  bookmarkState.loading = true;
  try {
    bookmarkState.list = await invoke<Bookmark[]>('get_bookmarks');
  } catch {
    bookmarkState.list = [];
  } finally {
    bookmarkState.loading = false;
  }
}

export async function addBookmark(surah: number, ayah: number, surahName: string, ayahText: string) {
  try {
    bookmarkState.list = await invoke<Bookmark[]>('add_bookmark', { surah, ayah, surahName, ayahText });
  } catch {
    // silently fail
  }
}

export async function removeBookmark(surah: number, ayah: number) {
  try {
    bookmarkState.list = await invoke<Bookmark[]>('remove_bookmark', { surah, ayah });
  } catch {
    // silently fail
  }
}

export function isBookmarked(surah: number, ayah: number): boolean {
  return bookmarkState.list.some((b) => b.surah === surah && b.ayah === ayah);
}

export async function toggleBookmark(surah: number, ayah: number, surahName: string, ayahText: string) {
  if (isBookmarked(surah, ayah)) {
    await removeBookmark(surah, ayah);
  } else {
    await addBookmark(surah, ayah, surahName, ayahText);
  }
}
