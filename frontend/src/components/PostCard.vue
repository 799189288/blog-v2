<script setup lang="ts">
import { NTag, NSpace } from 'naive-ui'
import { RouterLink } from 'vue-router'
import dayjs from 'dayjs'
import type { PostSummary } from '../types'

defineProps<{ post: PostSummary }>()
</script>

<template>
  <article class="post-card" :class="{ 'has-cover': post.cover_image }">
    <div class="card-body">
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
    </div>
    <RouterLink
      v-if="post.cover_image"
      :to="{ name: 'post', params: { slug: post.slug } }"
      class="cover-link"
      tabindex="-1"
      aria-hidden="true"
    >
      <img :src="post.cover_image" :alt="post.title" class="cover-img" loading="lazy" />
    </RouterLink>
  </article>
</template>

<style scoped>
.post-card {
  position: relative;
  display: flex;
  align-items: stretch;
  padding: 0;
  margin-bottom: 12px;
  border: 1px solid var(--n-border-color, rgba(127, 127, 127, 0.18));
  border-radius: 8px;
  background: var(--n-card-color, transparent);
  overflow: hidden;
  transition: border-color 0.2s, transform 0.18s, box-shadow 0.18s;
}
/* Left accent bar */
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
  z-index: 1;
}
.post-card:hover::before { width: 5px; }
.post-card:hover {
  border-color: rgba(192, 57, 43, 0.35);
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.08);
}

.card-body {
  flex: 1;
  min-width: 0;
  padding: 18px 18px 18px 22px;
}
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

/* Cover image */
.cover-link {
  flex: 0 0 120px;
  display: block;
  overflow: hidden;
}
.cover-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
  transition: transform 0.3s;
}
.post-card:hover .cover-img { transform: scale(1.04); }

@media (max-width: 768px) {
  .card-body { padding: 14px 14px 14px 18px; }
  .title { font-size: 18px; }
  .meta { gap: 8px 10px; font-size: 12px; }
  .meta-tags { margin-left: 0; }
  .cover-link { flex-basis: 90px; }
}
@media (max-width: 480px) {
  .post-card { flex-direction: column-reverse; }
  .cover-link { flex: 0 0 auto; height: 140px; }
  .cover-img { width: 100%; height: 100%; }
}
</style>
