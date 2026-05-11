<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { NSpin, NEmpty } from 'naive-ui'
import { useRoute } from 'vue-router'
import * as postsApi from '../api/posts'
import PostCard from '../components/PostCard.vue'
import BackButton from '../components/BackButton.vue'
import type { PostSummary } from '../types'

const props = defineProps<{ slug: string }>()
const route = useRoute()

const posts = ref<PostSummary[]>([])
const loading = ref(true)

async function load() {
  loading.value = true
  try {
    const res = await postsApi.listPublished({ tag: props.slug, per_page: 50 })
    posts.value = res.items
  } finally {
    loading.value = false
  }
}
onMounted(load)
watch(() => route.params.slug, load)
</script>

<template>
  <BackButton />
  <h2>Tag: {{ slug }}</h2>
  <NSpin :show="loading">
    <NEmpty v-if="!loading && posts.length === 0" description="No posts in this tag" />
    <PostCard v-for="p in posts" :key="p.id" :post="p" />
  </NSpin>
</template>
