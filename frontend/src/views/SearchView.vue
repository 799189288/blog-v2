<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { NInput, NButton, NSpin, NEmpty, NPagination } from 'naive-ui'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import * as postsApi from '../api/posts'
import PostCard from '../components/PostCard.vue'
import BackButton from '../components/BackButton.vue'
import { useHead } from '../composables/useHead'
import type { PostSummary } from '../types'

const route = useRoute()
const router = useRouter()
const { t } = useI18n()

useHead(() => ({ title: t('search.title') }))

const q = ref<string>((route.query.q as string) ?? '')
const page = ref(1)
const perPage = 10
const total = ref(0)
const posts = ref<PostSummary[]>([])
const loading = ref(false)

const hasQuery = computed(() => q.value.trim().length > 0)

async function load() {
  if (!hasQuery.value) {
    posts.value = []
    total.value = 0
    return
  }
  loading.value = true
  try {
    const res = await postsApi.search(q.value.trim(), page.value)
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
  <BackButton />
  <h2>{{ $t('search.title') }}</h2>
  <div style="display: flex; gap: 8px; max-width: 480px; margin-bottom: 16px;">
    <NInput v-model:value="q" :placeholder="$t('search.keywordsPlaceholder')" @keyup.enter="onSubmit" />
    <NButton type="primary" @click="onSubmit">{{ $t('common.search') }}</NButton>
  </div>
  <NSpin :show="loading">
    <NEmpty v-if="!loading && hasQuery && posts.length === 0" :description="$t('search.noResults')" />
    <PostCard v-for="p in posts" :key="p.id" :post="p" />
    <div v-if="total > perPage" style="display: flex; justify-content: center; margin-top: 24px;">
      <NPagination
        v-model:page="page"
        :item-count="total"
        :page-size="perPage"
        @update:page="load"
      />
    </div>
  </NSpin>
</template>
