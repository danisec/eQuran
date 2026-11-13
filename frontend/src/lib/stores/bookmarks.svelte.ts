import { invoke } from '@tauri-apps/api/core';

export type Bookmark = {
  surah: number;
  ayah: number;
  surahName: string;
  ayahText: string;
  /**
   * Translation chosen when the verse was marked, kept so a mark reads on its
   * own in the rail without refetching the surah. Older marks have none.
   */
  translation?: string;
  /**
   * Language of `translation`. The rail hides a mark's translation when it no
   * longer matches the interface language, rather than showing the wrong one.
   */
  lang?: string;
};

export type BookmarkGroup = {
  surah: number;
  surahName: string;
  items: Bookmark[];
};

export const bookmarkState = $state({
  list: [] as Bookmark[],
  loading: false
});

/**
 * Marks grouped by surah, with the surah holding the most recent mark first.
 *
 * The backend appends on add, so the last entry is the newest. The group that
 * contains it leads, so what the reader just marked appears at the top; the
 * verses inside a group stay in reading order.
 */
export function groupBookmarks(list: Bookmark[]): BookmarkGroup[] {
  const groups = new Map<number, BookmarkGroup>();
  const newest = new Map<number, number>();

  list.forEach((bookmark, index) => {
    const group = groups.get(bookmark.surah) ?? {
      surah: bookmark.surah,
      surahName: bookmark.surahName,
      items: []
    };
    group.items.push(bookmark);
    groups.set(bookmark.surah, group);
    newest.set(bookmark.surah, index);
  });

  for (const group of groups.values()) {
    group.items.sort((a, b) => a.ayah - b.ayah);
  }

  return [...groups.values()].sort(
    (a, b) => (newest.get(b.surah) ?? 0) - (newest.get(a.surah) ?? 0)
  );
}

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

export async function addBookmark(
  surah: number,
  ayah: number,
  surahName: string,
  ayahText: string,
  translation?: string,
  lang?: string
) {
  try {
    bookmarkState.list = await invoke<Bookmark[]>('add_bookmark', {
      surah,
      ayah,
      surahName,
      ayahText,
      translation,
      lang
    });
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

export async function toggleBookmark(
  surah: number,
  ayah: number,
  surahName: string,
  ayahText: string,
  translation?: string,
  lang?: string
) {
  if (isBookmarked(surah, ayah)) {
    await removeBookmark(surah, ayah);
  } else {
    await addBookmark(surah, ayah, surahName, ayahText, translation, lang);
  }
}
