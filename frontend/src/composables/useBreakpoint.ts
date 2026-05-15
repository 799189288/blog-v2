// Reactive viewport breakpoint signal shared across the app.
//
// Returns two booleans so templates and JS can branch without each
// instantiating its own matchMedia listener:
//   * isMobile — viewport ≤ 768px (the primary mobile/tablet boundary)
//   * isNarrow — viewport ≤ 480px (micro-adjustments for very narrow screens)
//
// CSS still owns layout (each component has its own scoped @media blocks);
// this composable exists for cases where the *template* needs to swap —
// table → card list, hamburger button visibility, mobile isolation pages.
//
// Module-level state mirrors useTheme: one matchMedia listener for the
// whole app rather than per-consumer.

import { readonly, ref } from 'vue'

const MD_QUERY = '(max-width: 768px)'
const SM_QUERY = '(max-width: 480px)'

const isMobile = ref(false)
const isNarrow = ref(false)

if (typeof window !== 'undefined') {
  const md = window.matchMedia(MD_QUERY)
  const sm = window.matchMedia(SM_QUERY)
  isMobile.value = md.matches
  isNarrow.value = sm.matches
  md.addEventListener('change', e => {
    isMobile.value = e.matches
  })
  sm.addEventListener('change', e => {
    isNarrow.value = e.matches
  })
}

export function useBreakpoint() {
  return {
    isMobile: readonly(isMobile),
    isNarrow: readonly(isNarrow),
  }
}
