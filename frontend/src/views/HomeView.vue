<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { NPagination, NSpin, NEmpty } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import * as postsApi from '../api/posts'
import * as tagsApi from '../api/tags'
import PostCard from '../components/PostCard.vue'
import HomeHero from '../components/HomeHero.vue'
import { RouterLink } from 'vue-router'
import { useHead } from '../composables/useHead'
import type { PostSummary, TagWithCount } from '../types'

const { t } = useI18n()
useHead(() => ({ title: t('layout.brand'), description: t('home.metaDescription') }))

const posts = ref<PostSummary[]>([])
const total = ref(0)
const page = ref(1)
const perPage = 10
const loading = ref(false)
const error = ref(false)
const tags = ref<TagWithCount[]>([])

async function load() {
  loading.value = true
  error.value = false
  try {
    const res = await postsApi.listPublished({ page: page.value, per_page: perPage })
    posts.value = res.items
    total.value = res.total
  } catch {
    error.value = true
  } finally {
    loading.value = false
  }
}

onMounted(async () => {
  await load()
  tags.value = await tagsApi.list()
})

// Map post_count to a font-size between 12px and 22px for the tag cloud.
const tagFontSize = computed(() => {
  if (tags.value.length === 0) return () => 14
  const counts = tags.value.map(t => t.post_count)
  const min = Math.min(...counts)
  const max = Math.max(...counts)
  return (count: number) => {
    if (max === min) return 14
    return Math.round(12 + ((count - min) / (max - min)) * 10)
  }
})
</script>

<template>
  <HomeHero />
  <div class="home">
    <div class="posts">
      <NSpin :show="loading">
        <NEmpty v-if="error" :description="$t('common.loadError')" />
        <NEmpty v-else-if="!loading && posts.length === 0" :description="$t('home.emptyPosts')" />
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
    </div>
    <aside class="sidebar">
      <h3>{{ $t('home.tags') }}</h3>
      <div class="tag-cloud">
        <RouterLink
          v-for="tag in tags"
          :key="tag.id"
          :to="{ name: 'tag', params: { slug: tag.slug } }"
          class="tag-chip"
          :style="{ fontSize: tagFontSize(tag.post_count) + 'px' }"
          :title="`${tag.name} (${tag.post_count})`"
        >{{ tag.name }}</RouterLink>
      </div>
    </aside>
  </div>
</template>

<style scoped>
.home {
  display: grid;
  grid-template-columns: 1fr 220px;
  gap: 32px;
}
@media (max-width: 768px) {
  .home { grid-template-columns: 1fr; }
}
.sidebar h3 { margin: 16px 0 12px; }

/* Tag cloud */
.tag-cloud {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: baseline;
}
.tag-chip {
  display: inline-block;
  padding: 3px 10px;
  border-radius: 999px;
  border: 1px solid rgba(127, 127, 127, 0.25);
  text-decoration: none;
  color: inherit;
  line-height: 1.5;
  transition: border-color 0.15s, color 0.15s, background 0.15s;
  white-space: nowrap;
}
.tag-chip:hover {
  border-color: var(--brand-color, #c0392b);
  color: var(--brand-color, #c0392b);
  background: rgba(192, 57, 43, 0.06);
}
</style>
