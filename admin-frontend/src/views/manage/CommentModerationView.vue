<script setup lang="ts">
import { computed, h, onMounted, ref, watch } from 'vue'
import {
  NRadioGroup, NRadioButton, NDataTable, NTag, NButton, NSpace, useDialog, useMessage,
} from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import dayjs from 'dayjs'
import * as commentsApi from '../../api/comments'
import type { Comment } from '../../types'
import { useDictStore } from '../../stores/dict'
import { useBreakpoint } from '../../composables/useBreakpoint'
import MobileRowCard from '../../components/MobileRowCard.vue'

const dialog = useDialog()
const message = useMessage()
const { t } = useI18n()
const dict = useDictStore()
const { isMobile } = useBreakpoint()

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

onMounted(async () => {
  await dict.ensure('comment.status')
  await load()
})
watch(status, load)

async function setStatus(c: Comment, next: 'approved' | 'spam' | 'pending') {
  try {
    await commentsApi.adminSetStatus(c.id, next)
    message.success(t('moderation.statusUpdated', { status: dict.label('comment.status', next) }))
    await load()
  } catch (e: any) {
    message.error(e?.response?.data?.error ?? t('common.failed'))
  }
}

function confirmDelete(c: Comment) {
  dialog.warning({
    title: t('moderation.deleteTitle'),
    content: t('moderation.deleteConfirm'),
    positiveText: t('common.confirmDelete'),
    negativeText: t('common.cancel'),
    onPositiveClick: async () => {
      try {
        await commentsApi.adminDelete(c.id)
        message.success(t('common.deleted'))
        await load()
      } catch (e: any) {
        message.error(e?.response?.data?.error ?? t('common.deleteFailed'))
      }
    },
  })
}

const columns = computed<DataTableColumns<Comment>>(() => [
  { title: t('moderation.cols.author'), key: 'author_name', width: 130 },
  {
    title: t('moderation.cols.post'), key: 'post_title', width: 200,
    render(r) {
      return h(
        RouterLink,
        { to: { name: 'manage-post-edit', params: { id: r.post_id } }, class: 'post-link' },
        () => r.post_title || `#${r.post_id}`,
      )
    },
  },
  {
    title: t('moderation.cols.replyTo'), key: 'parent_author_name', width: 130,
    render(r) {
      if (!r.parent_id) return h('span', { style: 'opacity:.4' }, '—')
      const name = r.parent_author_name ?? `#${r.parent_id}`
      return h('span', null, `@${name}`)
    },
  },
  { title: t('moderation.cols.content'), key: 'content', render(r) { return h('div', { style: 'white-space: pre-wrap' }, r.content) } },
  {
    title: t('moderation.cols.status'), key: 'status', width: 100,
    render(r) {
      const type = r.status === 'approved' ? 'success' : r.status === 'spam' ? 'error' : 'warning'
      return h(NTag, { type, size: 'small' }, () => dict.label('comment.status', r.status))
    },
  },
  { title: t('moderation.cols.when'), key: 'created_at', width: 130, render: r => dayjs(r.created_at).format('YYYY-MM-DD HH:mm') },
  {
    title: t('moderation.cols.actions'), key: 'actions', width: 240,
    render(r) {
      return h(NSpace, { size: 'small' }, () => [
        r.status !== 'approved' && h(NButton, { size: 'small', type: 'primary', onClick: () => setStatus(r, 'approved') }, () => t('moderation.approve')),
        r.status !== 'spam' && h(NButton, { size: 'small', onClick: () => setStatus(r, 'spam') }, () => t('moderation.markSpam')),
        h(NButton, { size: 'small', type: 'error', ghost: true, onClick: () => confirmDelete(r) }, () => t('common.delete')),
      ].filter(Boolean) as any)
    },
  },
])
</script>

<template>
  <div class="page-header">
    <h2 class="page-title">{{ t('moderation.title') }}</h2>
    <NRadioGroup v-model:value="status" size="small" class="filter-radios">
      <NRadioButton value="pending">{{ t('moderation.statuses.pending') }}</NRadioButton>
      <NRadioButton value="approved">{{ t('moderation.statuses.approved') }}</NRadioButton>
      <NRadioButton value="spam">{{ t('moderation.statuses.spam') }}</NRadioButton>
      <NRadioButton value="all">{{ t('moderation.statuses.all') }}</NRadioButton>
    </NRadioGroup>
  </div>
  <NDataTable
    v-if="!isMobile"
    :columns="columns"
    :data="rows"
    :loading="loading"
    :row-key="(r: Comment) => r.id"
  />
  <div v-else class="m-list">
    <MobileRowCard v-for="c in rows" :key="c.id">
      <template #title>
        {{ c.author_name }}
        <span v-if="c.author_email" class="m-email">{{ c.author_email }}</span>
      </template>
      <template #body>
        <RouterLink
          :to="{ name: 'manage-post-edit', params: { id: c.post_id } }"
          class="m-post-link"
        >
          {{ c.post_title || `#${c.post_id}` }}
        </RouterLink>
        <div class="m-content">{{ c.content }}</div>
        <div v-if="c.parent_id" class="muted">
          @{{ c.parent_author_name ?? `#${c.parent_id}` }}
        </div>
        <div class="m-line">
          <NTag
            :type="c.status === 'approved' ? 'success' : c.status === 'spam' ? 'error' : 'warning'"
            size="small"
          >
            {{ dict.label('comment.status', c.status) }}
          </NTag>
          <span class="muted">{{ dayjs(c.created_at).format('YYYY-MM-DD HH:mm') }}</span>
        </div>
      </template>
      <template #actions>
        <NButton v-if="c.status !== 'approved'" size="small" type="primary" @click="setStatus(c, 'approved')">
          {{ t('moderation.approve') }}
        </NButton>
        <NButton v-if="c.status !== 'spam'" size="small" @click="setStatus(c, 'spam')">
          {{ t('moderation.markSpam') }}
        </NButton>
        <NButton size="small" type="error" ghost @click="confirmDelete(c)">
          {{ t('common.delete') }}
        </NButton>
      </template>
    </MobileRowCard>
  </div>
</template>

<style scoped>
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
  flex-wrap: wrap;
  gap: 8px;
}
.page-title { margin: 0; }
.filter-radios { display: inline-flex; flex-wrap: wrap; }
:deep(.post-link) {
  color: inherit;
  text-decoration: none;
  font-size: 13px;
  opacity: 0.85;
}
:deep(.post-link:hover) { text-decoration: underline; }
.m-email { font-size: 12px; opacity: 0.65; font-weight: 400; margin-left: 6px; }
.m-post-link {
  display: block;
  font-size: 12px;
  opacity: 0.75;
  color: inherit;
  text-decoration: none;
  margin-bottom: 4px;
}
.m-post-link:hover { text-decoration: underline; }
.m-content { white-space: pre-wrap; word-break: break-word; }
.m-line { display: flex; flex-wrap: wrap; gap: 8px 10px; align-items: center; }
.muted { opacity: 0.7; font-size: 12px; }
</style>
