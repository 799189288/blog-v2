<script setup lang="ts">
import { onMounted, ref, computed } from 'vue'
import { NSpin, NEmpty } from 'naive-ui'
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import dayjs from 'dayjs'
import * as postsApi from '../api/posts'
import { useHead } from '../composables/useHead'
import type { ArchiveGroup } from '../types'

const { t, locale } = useI18n()
useHead(() => ({ title: t('archive.title') }))

const groups = ref<ArchiveGroup[]>([])
const loading = ref(true)

const totalCount = computed(() =>
  groups.value.reduce((n, g) => n + g.posts.length, 0),
)

onMounted(async () => {
  try {
    groups.value = await postsApi.archive()
  } finally {
    loading.value = false
  }
})

/// Format the group heading. Chinese gets "YYYY 年 MM 月" with the
/// month padded out long, English gets a short locale month name.
function formatHeader(g: ArchiveGroup): string {
  if (locale.value === 'zh') {
    return `${g.year} 年 ${g.month} 月`
  }
  const d = dayjs(`${g.year}-${String(g.month).padStart(2, '0')}-01`)
  return d.format('MMMM YYYY')
}
</script>

<template>
  <h2 class="page-title">
    {{ t('archive.title') }}
    <span v-if="!loading" class="count">{{ t('archive.totalCount', { count: totalCount }) }}</span>
  </h2>

  <NSpin :show="loading">
    <NEmpty v-if="!loading && groups.length === 0" :description="t('archive.empty')" />
    <section v-for="g in groups" :key="g.year_month" class="group">
      <h3 class="group-header">{{ formatHeader(g) }} <span class="group-count">({{ g.posts.length }})</span></h3>
      <ul class="post-list">
        <li v-for="p in g.posts" :key="p.slug">
          <span class="post-date">{{ dayjs(p.published_at).format('MM-DD') }}</span>
          <RouterLink :to="{ name: 'post', params: { slug: p.slug } }" class="post-title">{{ p.title }}</RouterLink>
        </li>
      </ul>
    </section>
  </NSpin>
</template>

<style scoped>
.page-title {
  display: flex;
  align-items: baseline;
  gap: 10px;
  margin: 0 0 24px;
  flex-wrap: wrap;
}
.count {
  font-size: 14px;
  opacity: 0.55;
  font-weight: 400;
}
.group {
  margin-bottom: 28px;
}
.group-header {
  margin: 0 0 10px;
  font-size: 16px;
  font-weight: 600;
  padding-bottom: 4px;
  border-bottom: 1px solid rgba(127, 127, 127, 0.15);
}
.group-count {
  font-weight: 400;
  opacity: 0.55;
  font-size: 13px;
}
.post-list {
  list-style: none;
  padding: 0;
  margin: 0;
}
.post-list li {
  display: flex;
  gap: 14px;
  align-items: baseline;
  padding: 5px 0;
}
.post-date {
  font-family: ui-monospace, "Cascadia Mono", Menlo, monospace;
  font-size: 12px;
  opacity: 0.6;
  /* Lock width so titles align even with single-digit days. */
  min-width: 48px;
}
.post-title {
  text-decoration: none;
  color: inherit;
}
.post-title:hover {
  text-decoration: underline;
}
@media (max-width: 480px) {
  .post-list li { gap: 10px; flex-wrap: wrap; }
  .post-date { min-width: 0; }
  .group-header { font-size: 15px; }
}
</style>
