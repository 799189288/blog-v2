<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { NSpin, NTag, NSpace, NDivider } from 'naive-ui'
import { RouterLink, useRoute } from 'vue-router'
import { MdPreview, MdCatalog, config } from 'md-editor-v3'
import mermaid from 'mermaid'
import katex from 'katex'
import 'katex/dist/katex.min.css'
import { useI18n } from 'vue-i18n'
import dayjs from 'dayjs'
import * as postsApi from '../api/posts'
import * as commentsApi from '../api/comments'
import CommentList from '../components/CommentList.vue'
import CommentForm from '../components/CommentForm.vue'
import BackButton from '../components/BackButton.vue'
import { useHead } from '../composables/useHead'
import { useTheme } from '../composables/useTheme'
import { rewriteUploads } from '../utils/uploadUrl'
import type { PostDetail, Comment, PostNav } from '../types'

config({
  editorExtensions: {
    mermaid: { instance: mermaid },
    katex: { instance: katex },
  },
})

const props = defineProps<{ slug: string }>()
const route = useRoute()
const { locale } = useI18n()

// Stable id linking MdPreview ↔ MdCatalog. MdCatalog discovers headings
// from the rendered preview by this id.
const previewId = 'public-post-preview'

const post = ref<PostDetail | null>(null)
const comments = ref<Comment[]>([])
const nav = ref<PostNav | null>(null)
const loading = ref(true)
const notFound = ref(false)

const editorLang = computed<'zh-CN' | 'en-US'>(() => (locale.value === 'zh' ? 'zh-CN' : 'en-US'))
const { isDark } = useTheme()
const editorTheme = computed<'light' | 'dark'>(() => (isDark.value ? 'dark' : 'light'))

// Optional preview token from the URL. Drafts only — the admin shares
// `?token=...` URLs so a draft post can be reviewed before publishing.
const previewToken = computed(() => {
  const t = route.query.token
  return typeof t === 'string' && t.length > 0 ? t : undefined
})

const isDraftPreview = computed(() => post.value?.status === 'draft')

// Rewrite `/uploads/...` link targets to absolute backend URLs when the
// public site and backend run on different origins (split-domain
// deploys). On same-origin setups this is a no-op.
const renderedMd = computed(() => rewriteUploads(post.value?.content_md ?? ''))

async function load() {
  loading.value = true
  notFound.value = false
  nav.value = null
  try {
    post.value = await postsApi.getBySlug(props.slug, previewToken.value)
    if (isDraftPreview.value) {
      // Draft previews don't show comments (post isn't public; comment
      // table joins on published posts) and skip the related-posts call.
      comments.value = []
      nav.value = null
    } else {
      // Fetch comments + nav in parallel — both depend only on slug, and
      // nav loading shouldn't block the comments section from rendering.
      const [c, n] = await Promise.allSettled([
        commentsApi.listApproved(props.slug),
        postsApi.getRelated(props.slug),
      ])
      comments.value = c.status === 'fulfilled' ? c.value : []
      nav.value = n.status === 'fulfilled' ? n.value : null
    }
  } catch (e: any) {
    if (e?.response?.status === 404) notFound.value = true
    else throw e
  } finally {
    loading.value = false
  }
}

onMounted(load)
watch(() => route.params.slug, load)
watch(() => route.query.token, load)

useHead(() => ({
  title: post.value?.title,
  description: post.value?.excerpt ?? undefined,
  type: 'article',
}))

async function reloadComments() {
  comments.value = await commentsApi.listApproved(props.slug)
}
</script>

<template>
  <BackButton />
  <div class="post-layout">
    <div class="post-main">
      <NSpin :show="loading">
        <div v-if="notFound" class="not-found">{{ $t('post.notFound') }}</div>
        <article v-else-if="post">
          <div v-if="isDraftPreview" class="draft-banner" role="status">
            {{ $t('post.draftBanner') }}
          </div>
          <h1>{{ post.title }}</h1>
          <div class="meta">
            <span v-if="post.published_at">{{ dayjs(post.published_at).format('YYYY-MM-DD') }}</span>
            <span class="views" :title="$t('post.views')">
              <span class="eye">👁</span> {{ post.views }}
            </span>
            <span v-if="post.reading_time_min > 0" class="reading-time" :title="$t('post.readingTime')">
              ⏱ {{ $t('post.readingTimeFull', { min: post.reading_time_min, words: post.word_count }) }}
            </span>
            <NSpace inline :size="6" class="meta-tags">
              <RouterLink
                v-for="t in post.tags"
                :key="t.id"
                :to="{ name: 'tag', params: { slug: t.slug } }"
                class="tag-link"
              >
                <NTag size="small" round>{{ t.name }}</NTag>
              </RouterLink>
            </NSpace>
          </div>
          <MdPreview
            :editor-id="previewId"
            :model-value="renderedMd"
            :language="editorLang"
            :theme="editorTheme"
            preview-theme="default"
            class="post-md-preview"
          />

          <template v-if="!isDraftPreview">
            <NDivider />

            <nav v-if="nav && (nav.prev || nav.next)" class="prev-next" :aria-label="$t('post.navAria')">
              <RouterLink
                v-if="nav.prev"
                :to="{ name: 'post', params: { slug: nav.prev.slug } }"
                class="nav-link nav-prev"
              >
                <div class="nav-label">← {{ $t('post.prev') }}</div>
                <div class="nav-title">{{ nav.prev.title }}</div>
              </RouterLink>
              <span v-else class="nav-spacer" />
              <RouterLink
                v-if="nav.next"
                :to="{ name: 'post', params: { slug: nav.next.slug } }"
                class="nav-link nav-next"
              >
                <div class="nav-label">{{ $t('post.next') }} →</div>
                <div class="nav-title">{{ nav.next.title }}</div>
              </RouterLink>
              <span v-else class="nav-spacer" />
            </nav>

            <section v-if="nav && nav.related.length" class="related">
              <h3>{{ $t('post.related') }}</h3>
              <ul class="related-list">
                <li v-for="r in nav.related" :key="r.slug">
                  <RouterLink :to="{ name: 'post', params: { slug: r.slug } }" class="related-link">
                    <div class="related-title">{{ r.title }}</div>
                    <div v-if="r.excerpt" class="related-excerpt">{{ r.excerpt }}</div>
                  </RouterLink>
                </li>
              </ul>
            </section>

            <NDivider v-if="nav && (nav.prev || nav.next || nav.related.length)" />

            <h3>{{ $t('post.comments', { count: comments.length }) }}</h3>
            <CommentList :comments="comments" :slug="post.slug" @submitted="reloadComments" />

            <NDivider />

            <h3>{{ $t('post.leaveComment') }}</h3>
            <CommentForm :slug="post.slug" @submitted="reloadComments" />
          </template>
        </article>
      </NSpin>
    </div>

    <!-- TOC lives OUTSIDE the NSpin wrapper so its `position: sticky`
         can track page scroll instead of NSpin's static container. -->
    <aside v-if="post" class="post-toc" :aria-label="$t('post.toc')">
      <div class="toc-sticky">
        <div class="toc-title">{{ $t('post.toc') }}</div>
        <MdCatalog
          :editor-id="previewId"
          :theme="editorTheme"
          scroll-element="html"
          :offset-top="16"
          class="toc-body"
        />
      </div>
    </aside>
  </div>
</template>

<style scoped>
.post-layout {
  /* Single column — the TOC is `position: fixed` and sits to the right
     of the content-wrap as an overlay, so we don't reserve a grid cell
     for it. On narrow screens the TOC is hidden, article stays full. */
  display: block;
}
.post-main { min-width: 0; }
.post-toc {
  /* Viewport-fixed sidebar to the right of the centered .content-wrap.
     We tried `position: sticky` first but Naive UI's NLayout family
     adds overflow on internal containers, which truncates the sticky
     scroll-port and makes it impossible to follow the page scroll.
     `fixed` bypasses the whole containing-block / overflow dance. */
  position: fixed;
  /* The content-wrap is max 960px centered. Place TOC's right edge near
     the viewport right, but never closer than 24px and never inside the
     content-wrap's right edge ((100vw - 960) / 2 - 220px gives a 20px
     gap when the viewport is exactly wide enough). */
  right: max(24px, calc((100vw - 960px) / 2 - 220px));
  top: 96px;
  width: 200px;
  max-height: calc(100vh - 120px);
  overflow-y: auto;
  font-size: 13px;
  z-index: 5;
}
.toc-sticky {
  /* The wrapper used to host position:sticky; now just a passthrough
     since the parent <aside> is the fixed element. Keeping the class
     so we don't have to touch the template again. */
}
.toc-title {
  font-weight: 600;
  font-size: 12px;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  opacity: 0.6;
  margin-bottom: 12px;
}
.toc-body {
  background: transparent !important;
}
/* MdCatalog renders divs (not ul/li) and ships its own moving indicator
   on the left of the active item. We hide that indicator and replace it
   with a static colored left-border per link — much steadier visually
   and works without knowing the indicator's exact positioning logic. */
.post-toc :deep(.md-editor-catalog-indicator) { display: none !important; }
.post-toc :deep(.md-editor-catalog-link) {
  display: block;
  padding: 6px 12px;
  border-left: 2px solid rgba(127, 127, 127, 0.18);
  color: inherit;
  cursor: pointer;
  opacity: 0.65;
  transition: opacity 0.15s, border-color 0.15s, color 0.15s;
}
.post-toc :deep(.md-editor-catalog-link:hover) { opacity: 1; }
.post-toc :deep(.md-editor-catalog-link.md-editor-catalog-active) {
  opacity: 1;
  font-weight: 600;
  color: #18a058;
  border-left-color: #18a058;
}

@media (max-width: 1399px) {
  /* Below ~1400px there's no room to the right of the 960px content-wrap
     for a 200px TOC without overlapping the article. Drop the sidebar;
     readers on smaller screens scroll the article and tap headings in
     the body to navigate. */
  .post-toc { display: none; }
}

.meta { font-size: 13px; opacity: 0.7; margin-bottom: 24px; display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
.meta-tags { margin-left: 12px; }
.draft-banner {
  background: rgba(240, 160, 32, 0.12);
  border: 1px solid rgba(240, 160, 32, 0.5);
  color: #b86d00;
  padding: 8px 14px;
  border-radius: 6px;
  font-size: 13px;
  margin-bottom: 18px;
}
.views { display: inline-flex; align-items: center; gap: 4px; }
.views .eye { font-size: 14px; }
.reading-time { display: inline-flex; align-items: center; gap: 4px; }
.tag-link { text-decoration: none; cursor: pointer; }
.tag-link :deep(.n-tag) { cursor: pointer; }
.not-found { padding: 60px 0; text-align: center; opacity: 0.6; }
.post-md-preview {
  background: transparent !important;
}
.post-md-preview :deep(.md-editor-preview-wrapper) {
  padding: 0;
}
.prev-next {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  margin: 16px 0;
}
.nav-link {
  display: block;
  padding: 12px 14px;
  border: 1px solid rgba(127, 127, 127, 0.2);
  border-radius: 6px;
  text-decoration: none;
  color: inherit;
  transition: border-color 0.15s, background 0.15s;
}
.nav-link:hover { border-color: rgba(127, 127, 127, 0.5); }
.nav-next { text-align: right; }
.nav-label { font-size: 12px; opacity: 0.6; margin-bottom: 4px; }
.nav-title { font-weight: 500; }
.nav-spacer { display: block; }
@media (max-width: 600px) {
  .prev-next { grid-template-columns: 1fr; }
  .nav-next { text-align: left; }
}
.related { margin: 24px 0 8px; }
.related h3 { margin-bottom: 12px; }
.related-list { list-style: none; padding: 0; margin: 0; }
.related-list li { padding: 8px 0; border-bottom: 1px solid rgba(127, 127, 127, 0.12); }
.related-list li:last-child { border-bottom: none; }
.related-link { text-decoration: none; color: inherit; display: block; }
.related-link:hover .related-title { text-decoration: underline; }
.related-title { font-weight: 500; }
.related-excerpt {
  font-size: 13px;
  opacity: 0.65;
  margin-top: 3px;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

@media (max-width: 768px) {
  h1 { font-size: 24px; line-height: 1.3; }
  .meta { gap: 8px 10px; font-size: 12px; }
  .meta-tags { margin-left: 0; }
  .post-md-preview :deep(pre) { overflow-x: auto; max-width: 100%; }
  .post-md-preview :deep(.katex-display) {
    overflow-x: auto;
    overflow-y: hidden;
    padding: 4px 0;
  }
  .post-md-preview :deep(table) { display: block; overflow-x: auto; }
  .post-md-preview :deep(img) { max-width: 100%; height: auto; }
  .post-md-preview :deep(.md-editor-preview) {
    padding: 12px !important;
    font-size: 15px;
  }
  .related-list :deep(li) { flex-wrap: wrap; gap: 6px 14px; }
}
@media (max-width: 480px) {
  h1 { font-size: 20px; }
  .nav-link { padding: 10px 12px; }
  .nav-label { font-size: 11px; }
  .nav-title { font-size: 14px; }
}
</style>
