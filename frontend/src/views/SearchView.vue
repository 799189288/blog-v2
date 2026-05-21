<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { NInput, NButton, NSpin, NPagination, NEmpty } from 'naive-ui'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import * as postsApi from '../api/posts'
import * as tagsApi from '../api/tags'
import { useHead } from '../composables/useHead'
import type { PostSummary, TagWithCount } from '../types'

const route = useRoute()
const router = useRouter()
const { t } = useI18n()

useHead(() => ({ title: t('search.title') }))

const q = ref<string>((route.query.q as string) ?? '')
const submittedQ = ref<string>(q.value)
const page = ref(1)
const perPage = 10
const total = ref(0)
const posts = ref<PostSummary[]>([])
const loading = ref(false)
const error = ref(false)
const topTags = ref<TagWithCount[]>([])

const hasSubmitted = computed(() => submittedQ.value.trim().length > 0)
const hasResults = computed(() => hasSubmitted.value && posts.value.length > 0)
const isEmptyResult = computed(() => hasSubmitted.value && !loading.value && posts.value.length === 0)

onMounted(async () => {
  try {
    const tags = await tagsApi.list()
    topTags.value = tags
      .filter(t => t.post_count > 0)
      .sort((a, b) => b.post_count - a.post_count)
      .slice(0, 6)
  } catch {
    // suggestions are non-critical, ignore errors
  }
})

async function load() {
  const term = q.value.trim()
  submittedQ.value = term
  if (!term) {
    posts.value = []
    total.value = 0
    return
  }
  loading.value = true
  error.value = false
  try {
    const res = await postsApi.search(term, page.value)
    posts.value = res.items
    total.value = res.total
  } catch {
    error.value = true
    posts.value = []
    total.value = 0
  } finally {
    loading.value = false
  }
}

function onSubmit() {
  page.value = 1
  router.replace({ name: 'search', query: { q: q.value.trim() } })
  load()
}

watch(() => route.query.q, val => {
  q.value = (val as string) ?? ''
  page.value = 1
  load()
}, { immediate: true })

function highlight(text: string, keyword: string): string {
  if (!keyword.trim()) return escapeHtml(text)
  const escaped = keyword.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  const re = new RegExp(`(${escaped})`, 'gi')
  return escapeHtml(text).replace(re, '<mark>$1</mark>')
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}

function searchSuggestion(word: string) {
  q.value = word
  onSubmit()
}
</script>

<template>
  <section class="search-page">
    <header class="search-hero">
      <h1 class="search-title">{{ t('search.title') }}</h1>
      <p class="search-subtitle">{{ t('search.subtitle') }}</p>
      <form class="search-form" @submit.prevent="onSubmit">
        <NInput
          v-model:value="q"
          :placeholder="t('search.keywordsPlaceholder')"
          size="large"
          autofocus
          clearable
          class="search-input"
          @keyup.enter="onSubmit"
        />
        <NButton type="primary" size="large" attr-type="submit" class="search-btn">
          {{ t('common.search') }}
        </NButton>
      </form>
      <p v-if="hasResults" class="results-count">
        {{ t('search.resultsCount', { n: total, q: submittedQ }) }}
      </p>
    </header>

    <NSpin :show="loading">
      <NEmpty v-if="error" :description="$t('common.loadError')" style="padding: 60px 0" />

      <template v-else-if="hasResults">
        <article v-for="p in posts" :key="p.id" class="result-card">
          <h2 class="result-title">
            <RouterLink :to="{ name: 'post', params: { slug: p.slug } }">
              <!-- eslint-disable-next-line vue/no-v-html -->
              <span v-html="highlight(p.title, submittedQ)" />
            </RouterLink>
          </h2>
          <p v-if="p.excerpt" class="result-excerpt">
            <!-- eslint-disable-next-line vue/no-v-html -->
            <span v-html="highlight(p.excerpt, submittedQ)" />
          </p>
        </article>
        <div v-if="total > perPage" class="pager">
          <NPagination
            v-model:page="page"
            :item-count="total"
            :page-size="perPage"
            @update:page="load"
          />
        </div>
      </template>

      <div v-else-if="isEmptyResult" class="state-empty">
        <div class="state-icon" aria-hidden="true">∅</div>
        <p class="state-text">{{ t('search.noResultsHint', { q: submittedQ }) }}</p>
        <div v-if="topTags.length" class="suggestions">
          <span class="suggestions-label">{{ t('search.trySuggestions') }}</span>
          <button
            v-for="tag in topTags"
            :key="tag.id"
            class="suggestion-chip"
            @click="searchSuggestion(tag.name)"
          >{{ tag.name }}</button>
        </div>
      </div>

      <div v-else-if="!hasSubmitted && !loading" class="state-empty subtle">
        <p class="state-text">{{ t('search.emptyHint') }}</p>
      </div>
    </NSpin>
  </section>
</template>

<style scoped>
.search-page {
  max-width: 720px;
  margin: 0 auto;
}
.search-hero {
  padding: 12px 0 28px;
  margin-bottom: 18px;
  border-bottom: 1px solid var(--n-border-color, rgba(127, 127, 127, 0.15));
}
.search-title {
  margin: 0 0 8px;
  font-size: 32px;
  line-height: 1.2;
}
.search-subtitle {
  margin: 0 0 20px;
  font-size: 14px;
  opacity: 0.7;
  line-height: 1.6;
}
.search-form {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}
.search-input {
  flex: 1 1 280px;
  min-width: 0;
}
.search-btn {
  flex: 0 0 auto;
  min-width: 88px;
}
.results-count {
  margin: 16px 0 0;
  font-size: 13px;
  opacity: 0.65;
}

/* ── Result cards with highlight ─────────────────────────────────────────── */
.result-card {
  padding: 16px 18px 16px 22px;
  margin-bottom: 10px;
  border: 1px solid var(--n-border-color, rgba(127, 127, 127, 0.18));
  border-radius: 8px;
  position: relative;
  transition: border-color 0.2s, transform 0.15s, box-shadow 0.15s;
}
.result-card::before {
  content: '';
  position: absolute;
  left: 0; top: 12px; bottom: 12px;
  width: 3px;
  background: var(--brand-color, #c0392b);
  border-radius: 0 2px 2px 0;
  transition: width 0.2s;
}
.result-card:hover { border-color: rgba(192, 57, 43, 0.4); transform: translateY(-1px); box-shadow: 0 4px 12px rgba(0,0,0,0.07); }
.result-card:hover::before { width: 5px; }
.result-title {
  margin: 0 0 6px;
  font-size: 20px;
  line-height: 1.3;
}
.result-title a { text-decoration: none; color: inherit; }
.result-title a:hover { text-decoration: underline; }
.result-excerpt {
  margin: 0;
  font-size: 14px;
  opacity: 0.7;
  line-height: 1.6;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}
/* Highlight marks */
:deep(mark) {
  background: rgba(192, 57, 43, 0.18);
  color: var(--brand-color, #c0392b);
  border-radius: 2px;
  padding: 0 2px;
  font-weight: 600;
}

/* ── Empty / suggestion states ───────────────────────────────────────────── */
.state-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 60px 20px;
  text-align: center;
}
.state-empty.subtle { padding-top: 40px; opacity: 0.85; }
.state-icon {
  font-family: 'EB Garamond', Georgia, serif;
  font-size: 56px;
  line-height: 1;
  color: var(--brand-color, #c0392b);
  opacity: 0.45;
}
.state-text {
  margin: 0;
  font-size: 14px;
  opacity: 0.75;
  max-width: 420px;
  line-height: 1.6;
}
.suggestions {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
  justify-content: center;
  margin-top: 4px;
}
.suggestions-label {
  font-size: 13px;
  opacity: 0.6;
}
.suggestion-chip {
  padding: 4px 12px;
  border-radius: 999px;
  border: 1px solid rgba(127, 127, 127, 0.3);
  background: transparent;
  color: inherit;
  font-size: 13px;
  cursor: pointer;
  transition: border-color 0.15s, color 0.15s, background 0.15s;
}
.suggestion-chip:hover {
  border-color: var(--brand-color, #c0392b);
  color: var(--brand-color, #c0392b);
  background: rgba(192, 57, 43, 0.06);
}

.pager {
  display: flex;
  justify-content: center;
  margin-top: 28px;
}

@media (max-width: 480px) {
  .search-title { font-size: 26px; }
  .search-btn { flex: 1 1 auto; }
  .state-empty { padding: 40px 12px; }
  .result-title { font-size: 17px; }
}
</style>
