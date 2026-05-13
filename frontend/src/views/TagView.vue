<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { NSpin, NEmpty } from 'naive-ui'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import * as postsApi from '../api/posts'
import * as tagsApi from '../api/tags'
import PostCard from '../components/PostCard.vue'
import BackButton from '../components/BackButton.vue'
import { useHead } from '../composables/useHead'
import type { PostSummary } from '../types'

const props = defineProps<{ slug: string }>()
const route = useRoute()
const { t } = useI18n()

const posts = ref<PostSummary[]>([])
const loading = ref(true)
const tagName = ref<string>(props.slug)
const tagMissing = ref(false)

useHead(() => ({
  title: t('tag.title', { slug: tagName.value }),
}))

async function load() {
  loading.value = true
  tagMissing.value = false
  try {
    const [tag, res] = await Promise.allSettled([
      tagsApi.getBySlug(props.slug),
      postsApi.listPublished({ tag: props.slug, per_page: 50 }),
    ])
    if (tag.status === 'fulfilled') {
      tagName.value = tag.value.name
    } else {
      tagName.value = props.slug
      tagMissing.value = (tag.reason?.response?.status === 404)
    }
    posts.value = res.status === 'fulfilled' ? res.value.items : []
  } finally {
    loading.value = false
  }
}
onMounted(load)
watch(() => route.params.slug, load)
</script>

<template>
  <BackButton />
  <h2>{{ $t('tag.title', { slug: tagName }) }}</h2>
  <NSpin :show="loading">
    <NEmpty v-if="!loading && tagMissing" :description="$t('tag.notFound')" />
    <NEmpty v-else-if="!loading && posts.length === 0" :description="$t('tag.emptyPosts')" />
    <PostCard v-for="p in posts" :key="p.id" :post="p" />
  </NSpin>
</template>
