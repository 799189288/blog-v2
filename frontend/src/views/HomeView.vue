<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { NPagination, NSpin, NEmpty } from 'naive-ui'
import * as postsApi from '../api/posts'
import * as tagsApi from '../api/tags'
import PostCard from '../components/PostCard.vue'
import { RouterLink } from 'vue-router'
import type { PostSummary, TagWithCount } from '../types'

const posts = ref<PostSummary[]>([])
const total = ref(0)
const page = ref(1)
const perPage = 10
const loading = ref(false)
const tags = ref<TagWithCount[]>([])

async function load() {
  loading.value = true
  try {
    const res = await postsApi.listPublished({ page: page.value, per_page: perPage })
    posts.value = res.items
    total.value = res.total
  } finally {
    loading.value = false
  }
}

onMounted(async () => {
  await load()
  tags.value = await tagsApi.list()
})
</script>

<template>
  <div class="home">
    <div class="posts">
      <NSpin :show="loading">
        <NEmpty v-if="!loading && posts.length === 0" description="No posts yet" />
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
      <h3>Tags</h3>
      <ul class="tag-list">
        <li v-for="t in tags" :key="t.id">
          <RouterLink :to="{ name: 'tag', params: { slug: t.slug } }">
            {{ t.name }} <span class="count">({{ t.post_count }})</span>
          </RouterLink>
        </li>
      </ul>
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
.sidebar h3 { margin: 16px 0 8px; }
.tag-list { list-style: none; padding: 0; }
.tag-list li { padding: 4px 0; }
.tag-list a { text-decoration: none; }
.tag-list a:hover { text-decoration: underline; }
.count { opacity: 0.6; font-size: 12px; }
</style>
