<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { NSpin, NTag, NSpace, NDivider } from 'naive-ui'
import { RouterLink, useRoute } from 'vue-router'
import { MdPreview, config } from 'md-editor-v3'
import mermaid from 'mermaid'
import { useI18n } from 'vue-i18n'
import dayjs from 'dayjs'
import * as postsApi from '../api/posts'
import * as commentsApi from '../api/comments'
import CommentList from '../components/CommentList.vue'
import CommentForm from '../components/CommentForm.vue'
import BackButton from '../components/BackButton.vue'
import type { PostDetail, Comment } from '../types'

config({
  editorExtensions: {
    mermaid: { instance: mermaid },
  },
})

const props = defineProps<{ slug: string }>()
const route = useRoute()
const { locale } = useI18n()

const post = ref<PostDetail | null>(null)
const comments = ref<Comment[]>([])
const loading = ref(true)
const notFound = ref(false)

const editorLang = computed<'zh-CN' | 'en-US'>(() => (locale.value === 'zh' ? 'zh-CN' : 'en-US'))
const editorTheme = computed<'light' | 'dark'>(() =>
  window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light',
)

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
    <div v-if="notFound" class="not-found">{{ $t('post.notFound') }}</div>
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
      <MdPreview
        :model-value="post.content_md"
        :language="editorLang"
        :theme="editorTheme"
        preview-theme="default"
        class="post-md-preview"
      />

      <NDivider />

      <h3>{{ $t('post.comments', { count: comments.length }) }}</h3>
      <CommentList :comments="comments" :slug="post.slug" @submitted="reloadComments" />

      <NDivider />

      <h3>{{ $t('post.leaveComment') }}</h3>
      <CommentForm :slug="post.slug" @submitted="reloadComments" />
    </article>
  </NSpin>
</template>

<style scoped>
.meta { font-size: 13px; opacity: 0.7; margin-bottom: 24px; display: flex; align-items: center; }
.tag-link { text-decoration: none; cursor: pointer; }
.tag-link :deep(.n-tag) { cursor: pointer; }
.not-found { padding: 60px 0; text-align: center; opacity: 0.6; }
.post-md-preview {
  background: transparent !important;
}
.post-md-preview :deep(.md-editor-preview-wrapper) {
  padding: 0;
}
</style>
