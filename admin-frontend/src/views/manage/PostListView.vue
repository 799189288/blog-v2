<script setup lang="ts">
import { h, onMounted, ref } from 'vue'
import { NDataTable, NTag, NButton, NSpace, useDialog, useMessage } from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { RouterLink, useRouter } from 'vue-router'
import dayjs from 'dayjs'
import * as postsApi from '../../api/posts'
import type { PostSummary } from '../../types'

const router = useRouter()
const dialog = useDialog()
const message = useMessage()

const posts = ref<PostSummary[]>([])
const loading = ref(false)

async function load() {
  loading.value = true
  try {
    posts.value = await postsApi.adminList()
  } finally {
    loading.value = false
  }
}

onMounted(load)

function confirmDelete(p: PostSummary) {
  dialog.warning({
    title: 'Delete post',
    content: `Delete "${p.title}"? This cannot be undone.`,
    positiveText: 'Delete',
    negativeText: 'Cancel',
    onPositiveClick: async () => {
      try {
        await postsApi.adminDelete(p.id)
        message.success('Post deleted')
        await load()
      } catch (e: any) {
        message.error(e?.response?.data?.error ?? 'Delete failed')
      }
    },
  })
}

const columns: DataTableColumns<PostSummary> = [
  {
    title: 'Title', key: 'title',
    render(row) {
      return h(RouterLink, { to: { name: 'manage-post-edit', params: { id: row.id } } }, () => row.title)
    },
  },
  {
    title: 'Status', key: 'status', width: 110,
    render(row) {
      return h(NTag, { type: row.status === 'published' ? 'success' : 'default', size: 'small' }, () => row.status)
    },
  },
  { title: 'Tags', key: 'tags', render: r => r.tags.map(t => t.name).join(', ') },
  { title: 'Published', key: 'published_at', width: 140, render: r => r.published_at ? dayjs(r.published_at).format('YYYY-MM-DD') : '—' },
  {
    title: 'Actions', key: 'actions', width: 200,
    render(row) {
      return h(NSpace, {}, () => [
        h(NButton, { size: 'small', onClick: () => router.push({ name: 'manage-post-edit', params: { id: row.id } }) }, () => 'Edit'),
        h(NButton, { size: 'small', type: 'error', ghost: true, onClick: () => confirmDelete(row) }, () => 'Delete'),
      ])
    },
  },
]
</script>

<template>
  <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
    <h2 style="margin: 0">Posts</h2>
    <NButton type="primary" @click="router.push({ name: 'manage-post-new' })">New post</NButton>
  </div>
  <NDataTable :columns="columns" :data="posts" :loading="loading" :row-key="(r: PostSummary) => r.id" />
</template>
