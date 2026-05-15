<script setup lang="ts">
import { NTag, NSpace } from 'naive-ui'
import { RouterLink } from 'vue-router'
import dayjs from 'dayjs'
import type { PostSummary } from '../types'

defineProps<{ post: PostSummary }>()
</script>

<template>
  <article class="post-card">
    <h2 class="title">
      <RouterLink :to="{ name: 'post', params: { slug: post.slug } }">{{ post.title }}</RouterLink>
    </h2>
    <div class="meta">
      <span v-if="post.published_at">
        {{ dayjs(post.published_at).format('YYYY-MM-DD') }}
      </span>
      <span class="views" :title="$t('post.views')">
        <span class="eye">👁</span> {{ post.views }}
      </span>
      <span v-if="post.reading_time_min > 0" class="reading-time" :title="$t('post.readingTime')">
        ⏱ {{ $t('post.readingTimeShort', { min: post.reading_time_min }) }}
      </span>
      <NSpace :size="6" inline class="meta-tags">
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
    <p v-if="post.excerpt" class="excerpt">{{ post.excerpt }}</p>
  </article>
</template>

<style scoped>
.post-card {
  padding: 16px 0;
  border-bottom: 1px solid var(--n-border-color, #eee);
}
.title { margin: 0 0 4px; font-size: 22px; }
.title a { text-decoration: none; }
.title a:hover { text-decoration: underline; }
.meta { font-size: 13px; opacity: 0.7; display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
.meta-tags { margin-left: 12px; }
.views { display: inline-flex; align-items: center; gap: 4px; }
.views .eye { font-size: 14px; }
.reading-time { display: inline-flex; align-items: center; gap: 4px; }
.excerpt { margin: 8px 0 0; }
.tag-link { text-decoration: none; cursor: pointer; }
.tag-link :deep(.n-tag) { cursor: pointer; }
@media (max-width: 768px) {
  .title { font-size: 18px; }
  .meta { gap: 8px 10px; font-size: 12px; }
  .meta-tags { margin-left: 0; }
}
</style>
