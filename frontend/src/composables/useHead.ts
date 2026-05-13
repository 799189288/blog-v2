// Lightweight head-tag manager — keeps a Vue SPA's <title>/meta in sync
// with the current view, including OG and Twitter card tags so links
// shared on social/chat apps render a useful preview.
//
// We don't bring in @unhead/vue: a few lines of DOM manipulation cover
// every tag we currently care about, and SEO crawlers see them after
// hydration. (For real search-engine indexing we'd still need SSR/SSG.)
import { onUnmounted, watchEffect } from 'vue'

export interface HeadOptions {
  title?: string
  description?: string
  image?: string
  type?: 'website' | 'article'
}

const DEFAULT_TITLE = 'Blog'

// Tags we set are marked with this attribute so we can clean them up
// without clobbering anything that came from index.html or other code.
const MANAGED_ATTR = 'data-managed-head'

function upsertMeta(attr: 'name' | 'property', key: string, content: string) {
  let el = document.head.querySelector<HTMLMetaElement>(
    `meta[${attr}="${key}"]`,
  )
  if (!el) {
    el = document.createElement('meta')
    el.setAttribute(attr, key)
    el.setAttribute(MANAGED_ATTR, '')
    document.head.appendChild(el)
  }
  el.setAttribute('content', content)
}

function clearManaged() {
  document.head
    .querySelectorAll(`meta[${MANAGED_ATTR}]`)
    .forEach(el => el.remove())
}

export function useHead(getOptions: () => HeadOptions) {
  watchEffect(() => {
    const o = getOptions()
    const title = o.title ?? DEFAULT_TITLE
    const desc = o.description ?? ''
    const url = window.location.href
    const type = o.type ?? 'website'

    document.title = title
    if (desc) upsertMeta('name', 'description', desc)
    upsertMeta('property', 'og:title', title)
    if (desc) upsertMeta('property', 'og:description', desc)
    upsertMeta('property', 'og:url', url)
    upsertMeta('property', 'og:type', type)
    upsertMeta('name', 'twitter:title', title)
    if (desc) upsertMeta('name', 'twitter:description', desc)
    upsertMeta('name', 'twitter:card', o.image ? 'summary_large_image' : 'summary')
    if (o.image) {
      upsertMeta('property', 'og:image', o.image)
      upsertMeta('name', 'twitter:image', o.image)
    }
  })

  onUnmounted(() => {
    // Leave the title alone — the next view's useHead will overwrite it.
    // Strip managed meta so a previous view's tags don't bleed onto the next.
    clearManaged()
  })
}
