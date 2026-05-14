// Reactive theme preference shared across the app.
//
// Three states:
//   * 'auto'  → follow OS prefers-color-scheme; switches live if the user
//               flips their system theme while the page is open.
//   * 'light' → forced light, regardless of OS preference.
//   * 'dark'  → forced dark.
//
// The user's *choice* persists across reloads via localStorage. The
// *resolved* boolean (`isDark`) is the thing components actually consume.
//
// We share a single module-level ref so multiple consumers (App, header
// toggle button, MdPreview theme prop) stay in sync without prop drilling.

import { computed, ref, watch } from 'vue'

export type ThemeMode = 'auto' | 'light' | 'dark'

const STORAGE_KEY = 'blog_theme_mode'

function readStored(): ThemeMode {
  try {
    const v = localStorage.getItem(STORAGE_KEY)
    if (v === 'light' || v === 'dark' || v === 'auto') return v
  } catch {
    // localStorage can throw in private mode / when disabled — fall through.
  }
  return 'auto'
}

const mode = ref<ThemeMode>(readStored())

// Live OS preference — kept reactive so 'auto' mode flips when the user
// changes their system theme while the tab is open.
const osDark = ref(
  typeof window !== 'undefined' &&
    window.matchMedia('(prefers-color-scheme: dark)').matches,
)
if (typeof window !== 'undefined') {
  const mq = window.matchMedia('(prefers-color-scheme: dark)')
  mq.addEventListener('change', e => {
    osDark.value = e.matches
  })
}

watch(mode, m => {
  try {
    localStorage.setItem(STORAGE_KEY, m)
  } catch {
    /* ignore */
  }
})

const isDark = computed(() =>
  mode.value === 'auto' ? osDark.value : mode.value === 'dark',
)

export function useTheme() {
  return {
    mode,
    isDark,
    setMode(next: ThemeMode) {
      mode.value = next
    },
    /// Cycle auto → light → dark → auto. Handy for a single toggle button.
    cycle() {
      mode.value =
        mode.value === 'auto' ? 'light' : mode.value === 'light' ? 'dark' : 'auto'
    },
  }
}
