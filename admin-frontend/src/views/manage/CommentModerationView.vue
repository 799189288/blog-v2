<script setup lang="ts">
import { h, onMounted, ref, watch } from 'vue'
import {
  NRadioGroup, NRadioButton, NDataTable, NTag, NButton, NSpace, useDialog, useMessage,
} from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import dayjs from 'dayjs'
import * as commentsApi from '../../api/comments'
import type { Comment } from '../../types'

const dialog = useDialog()
const message = useMessage()

const status = ref<'pending' | 'approved' | 'spam' | 'all'>('pending')
const rows = ref<Comment[]>([])
const loading = ref(false)

async function load() {
  loading.value = true
  try {
    rows.value = await commentsApi.adminList(status.value === 'all' ? undefined : status.value)
  } finally {
    loading.value = false
  }
}

onMounted(load)
watch(status, load)

async function setStatus(c: Comment, next: 'approved' | 'spam' | 'pending') {
  try {
    await commentsApi.adminSetStatus(c.id, next)
    message.success(`Comment ${next}`)
    await load()
  } catch (e: any) {
    message.error(e?.response?.data?.error ?? 'Failed')
  }
}

function confirmDelete(c: Comment) {
  dialog.warning({
    title: 'Delete comment',
    content: 'Permanently delete this comment?',
    positiveText: 'Delete',
    negativeText: 'Cancel',
    onPositiveClick: async () => {
      try {
        await commentsApi.adminDelete(c.id)
        message.success('Deleted')
        await load()
      } catch (e: any) {
        message.error(e?.response?.data?.error ?? 'Delete failed')
      }
    },
  })
}

const columns: DataTableColumns<Comment> = [
  { title: 'Author', key: 'author_name', width: 160 },
  { title: 'Email', key: 'author_email', width: 200 },
  { title: 'Content', key: 'content', render(r) { return h('div', { style: 'white-space: pre-wrap' }, r.content) } },
  {
    title: 'Status', key: 'status', width: 110,
    render(r) {
      const type = r.status === 'approved' ? 'success' : r.status === 'spam' ? 'error' : 'warning'
      return h(NTag, { type, size: 'small' }, () => r.status)
    },
  },
  { title: 'When', key: 'created_at', width: 140, render: r => dayjs(r.created_at).format('YYYY-MM-DD HH:mm') },
  {
    title: 'Actions', key: 'actions', width: 260,
    render(r) {
      return h(NSpace, { size: 'small' }, () => [
        r.status !== 'approved' && h(NButton, { size: 'small', type: 'primary', onClick: () => setStatus(r, 'approved') }, () => 'Approve'),
        r.status !== 'spam' && h(NButton, { size: 'small', onClick: () => setStatus(r, 'spam') }, () => 'Spam'),
        h(NButton, { size: 'small', type: 'error', ghost: true, onClick: () => confirmDelete(r) }, () => 'Delete'),
      ].filter(Boolean) as any)
    },
  },
]
</script>

<template>
  <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
    <h2 style="margin: 0">Comment moderation</h2>
    <NRadioGroup v-model:value="status" size="small">
      <NRadioButton value="pending">Pending</NRadioButton>
      <NRadioButton value="approved">Approved</NRadioButton>
      <NRadioButton value="spam">Spam</NRadioButton>
      <NRadioButton value="all">All</NRadioButton>
    </NRadioGroup>
  </div>
  <NDataTable :columns="columns" :data="rows" :loading="loading" :row-key="(r: Comment) => r.id" />
</template>
