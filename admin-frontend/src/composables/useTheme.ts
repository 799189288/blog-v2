// Theme preference for the admin console. Independent from the public
// site (different storage key) so each can have its own taste.
//
// See frontend/src/composables/useTheme.ts for the rationale behind the
// three-state design — this file is a deliberate copy rather than a
// shared package since the two SPAs ship as separate bundles.

import { computed, ref, watch } from 'vue'

export type ThemeMode = 'auto' | 'light' | 'dark'

const STORAGE_KEY = 'blog_admin_theme_mode'

function readStored(): ThemeMode {
  try {
    const v = localStorage.getItem(STORAGE_KEY)
    if (v === 'light' || v === 'dark' || v === 'auto') return v
  } catch {
    /* localStorage disabled */
  }
  return 'auto'
}

const mode = ref<ThemeMode>(readStored())

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
    cycle() {
      mode.value =
        mode.value === 'auto' ? 'light' : mode.value === 'light' ? 'dark' : 'auto'
    },
  }
}
