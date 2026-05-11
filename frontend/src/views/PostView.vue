<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { NSpin, NTag, NSpace, NDivider } from 'naive-ui'
import { RouterLink, useRoute } from 'vue-router'
import dayjs from 'dayjs'
import * as postsApi from '../api/posts'
import * as commentsApi from '../api/comments'
import PostContent from '../components/PostContent.vue'
import CommentList from '../components/CommentList.vue'
import CommentForm from '../components/CommentForm.vue'
import BackButton from '../components/BackButton.vue'
import type { PostDetail, Comment } from '../types'

const props = defineProps<{ slug: string }>()
const route = useRoute()

const post = ref<PostDetail | null>(null)
const comments = ref<Comment[]>([])
const loading = ref(true)
const notFound = ref(false)

async function load() {
  loading.value = true
  notFound.value = false
  try {
    post.value = await postsApi.getBySlug(props.slug)
    comments.value = await commentsApi.listApproved(props.slug)
  } catch (e: any) {
    if (e?.response?.status === 404) notFound.value = true
    else throw e
  } finally {
    loading.value = false
  }
}

onMounted(load)
watch(() => route.params.slug, load)

async function reloadComments() {
  comments.value = await commentsApi.listApproved(props.slug)
}
</script>

<template>
  <BackButton />
  <NSpin :show="loading">
    <div v-if="notFound" class="not-found">Post not found.</div>
    <article v-else-if="post">
      <h1>{{ post.title }}</h1>
      <div class="meta">
        <span v-if="post.published_at">{{ dayjs(post.published_at).format('YYYY-MM-DD') }}</span>
        <NSpace inline :size="6" style="margin-left: 12px;">
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
      <PostContent :html="post.content_html" />

      <NDivider />

      <h3>Comments ({{ comments.length }})</h3>
      <CommentList :comments="comments" />

      <NDivider />

      <h3>Leave a comment</h3>
      <CommentForm :slug="post.slug" @submitted="reloadComments" />
    </article>
  </NSpin>
</template>

<style scoped>
.meta { font-size: 13px; opacity: 0.7; margin-bottom: 24px; display: flex; align-items: center; }
.tag-link { text-decoration: none; }
.not-found { padding: 60px 0; text-align: center; opacity: 0.6; }
</style>
