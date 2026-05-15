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
        {{ $t('post.viewsCount', { n: post.views }) }}
      </span>
      <span v-if="post.reading_time_min > 0" class="reading-time" :title="$t('post.readingTime')">
        {{ $t('post.readingTimeShort', { min: post.reading_time_min }) }}
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
  position: relative;
  padding: 18px 18px 18px 22px;
  margin-bottom: 12px;
  border: 1px solid var(--n-border-color, rgba(127, 127, 127, 0.18));
  border-radius: 8px;
  background: var(--n-card-color, transparent);
  transition: border-color 0.2s;
}
/* Left accent bar that grows on hover. Using a pseudo-element keeps
   the layout stable (no width change on the card itself). */
.post-card::before {
  content: '';
  position: absolute;
  left: 0;
  top: 12px;
  bottom: 12px;
  width: 3px;
  background: var(--brand-color, #c0392b);
  border-radius: 0 2px 2px 0;
  transition: width 0.2s;
}
.post-card:hover::before { width: 5px; }
.post-card:hover { border-color: rgba(192, 57, 43, 0.4); }
.title { margin: 0 0 4px; font-size: 22px; }
.title a { text-decoration: none; }
.title a:hover { text-decoration: underline; }
.meta { font-size: 13px; opacity: 0.7; display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
.meta-tags { margin-left: 12px; }
.views { display: inline-flex; align-items: center; gap: 4px; }
.reading-time { display: inline-flex; align-items: center; gap: 4px; }
.excerpt { margin: 8px 0 0; }
.tag-link { text-decoration: none; cursor: pointer; }
.tag-link :deep(.n-tag) { cursor: pointer; }
@media (max-width: 768px) {
  .post-card { padding: 14px 14px 14px 18px; }
  .title { font-size: 18px; }
  .meta { gap: 8px 10px; font-size: 12px; }
  .meta-tags { margin-left: 0; }
}
</style>
