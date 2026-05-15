<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { NInput, NButton, NSpin, NPagination } from 'naive-ui'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import * as postsApi from '../api/posts'
import PostCard from '../components/PostCard.vue'
import { useHead } from '../composables/useHead'
import type { PostSummary } from '../types'

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

const hasSubmitted = computed(() => submittedQ.value.trim().length > 0)
const hasResults = computed(() => hasSubmitted.value && posts.value.length > 0)
const isEmptyResult = computed(() => hasSubmitted.value && !loading.value && posts.value.length === 0)

async function load() {
  const term = q.value.trim()
  submittedQ.value = term
  if (!term) {
    posts.value = []
    total.value = 0
    return
  }
  loading.value = true
  try {
    const res = await postsApi.search(term, page.value)
    posts.value = res.items
    total.value = res.total
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
      <template v-if="hasResults">
        <PostCard v-for="p in posts" :key="p.id" :post="p" />
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
.pager {
  display: flex;
  justify-content: center;
  margin-top: 28px;
}

@media (max-width: 480px) {
  .search-title { font-size: 26px; }
  .search-btn { flex: 1 1 auto; }
  .state-empty { padding: 40px 12px; }
}
</style>
