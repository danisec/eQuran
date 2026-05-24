type ThemeMode = 'light' | 'dark';

const STORAGE_KEY = 'equran-theme';

export const themeState = $state({
  mode: 'light' as ThemeMode
});

function preferredTheme(): ThemeMode {
  const savedTheme = localStorage.getItem(STORAGE_KEY);
  if (savedTheme === 'dark' || savedTheme === 'light') {
    return savedTheme;
  }

  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

export function applyTheme(nextTheme: ThemeMode) {
  themeState.mode = nextTheme;
  document.documentElement.classList.toggle('dark', nextTheme === 'dark');
  document.documentElement.style.colorScheme = nextTheme;
  localStorage.setItem(STORAGE_KEY, nextTheme);
}

export function initTheme() {
  applyTheme(preferredTheme());
}

export function toggleTheme() {
  applyTheme(themeState.mode === 'dark' ? 'light' : 'dark');
}
