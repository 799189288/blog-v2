<script setup lang="ts">
import { computed, h, onMounted, ref } from 'vue'
import { NDataTable, NTag, NButton, NSpace, useDialog, useMessage } from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { RouterLink, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import dayjs from 'dayjs'
import * as postsApi from '../../api/posts'
import type { PostSummary } from '../../types'
import { useDictStore } from '../../stores/dict'
import { useBreakpoint } from '../../composables/useBreakpoint'
import MobileRowCard from '../../components/MobileRowCard.vue'

const router = useRouter()
const dialog = useDialog()
const message = useMessage()
const { t } = useI18n()
const dict = useDictStore()
const { isMobile } = useBreakpoint()

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

onMounted(async () => {
  await dict.ensure('post.status')
  await load()
})

function confirmDelete(p: PostSummary) {
  dialog.warning({
    title: t('managePosts.deleteTitle'),
    content: t('managePosts.deleteConfirm', { title: p.title }),
    positiveText: t('common.confirmDelete'),
    negativeText: t('common.cancel'),
    onPositiveClick: async () => {
      try {
        await postsApi.adminDelete(p.id)
        message.success(t('managePosts.deleted'))
        await load()
      } catch (e: any) {
        message.error(e?.response?.data?.error ?? t('common.deleteFailed'))
      }
    },
  })
}

const columns = computed<DataTableColumns<PostSummary>>(() => [
  {
    title: t('managePosts.cols.title'), key: 'title',
    render(row) {
      return h(
        RouterLink,
        {
          to: { name: 'manage-post-edit', params: { id: row.id } },
          class: 'row-link',
        },
        () => row.title,
      )
    },
  },
  {
    title: t('managePosts.cols.status'), key: 'status', width: 110,
    render(row) {
      return h(NTag, { type: row.status === 'published' ? 'success' : 'default', size: 'small' }, () => dict.label('post.status', row.status))
    },
  },
  { title: t('managePosts.cols.tags'), key: 'tags', render: r => r.tags.map(tg => tg.name).join(', ') },
  { title: t('managePosts.cols.views'), key: 'views', width: 90 },
  { title: t('managePosts.cols.published'), key: 'published_at', width: 140, render: r => r.published_at ? dayjs(r.published_at).format('YYYY-MM-DD') : t('common.none') },
  {
    title: t('managePosts.cols.actions'), key: 'actions', width: 200,
    render(row) {
      return h(NSpace, {}, () => [
        h(NButton, { size: 'small', onClick: () => router.push({ name: 'manage-post-edit', params: { id: row.id } }) }, () => t('common.edit')),
        h(NButton, { size: 'small', type: 'error', ghost: true, onClick: () => confirmDelete(row) }, () => t('common.delete')),
      ])
    },
  },
])
</script>

<template>
  <div class="page-header">
    <h2 class="page-title">{{ t('managePosts.title') }}</h2>
    <NButton type="primary" @click="router.push({ name: 'manage-post-new' })">{{ t('managePosts.newPost') }}</NButton>
  </div>
  <NDataTable
    v-if="!isMobile"
    :columns="columns"
    :data="posts"
    :loading="loading"
    :row-key="(r: PostSummary) => r.id"
  />
  <div v-else class="m-list">
    <MobileRowCard v-for="p in posts" :key="p.id">
      <template #title>
        <RouterLink :to="{ name: 'manage-post-edit', params: { id: p.id } }" class="row-link">
          {{ p.title }}
        </RouterLink>
      </template>
      <template #body>
        <div class="m-line">
          <NTag :type="p.status === 'published' ? 'success' : 'default'" size="small">
            {{ dict.label('post.status', p.status) }}
          </NTag>
          <span v-if="p.published_at" class="muted">
            {{ dayjs(p.published_at).format('YYYY-MM-DD') }}
          </span>
          <span class="muted">👁 {{ p.views }}</span>
        </div>
        <div v-if="p.tags.length" class="m-line muted">
          {{ p.tags.map(tg => tg.name).join(', ') }}
        </div>
      </template>
      <template #actions>
        <NButton size="small" @click="router.push({ name: 'manage-post-edit', params: { id: p.id } })">
          {{ t('common.edit') }}
        </NButton>
        <NButton size="small" type="error" ghost @click="confirmDelete(p)">
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
/* RouterLink renders as <a>, which the UA paints in its default link
   color (blue/purple, including :visited). That's invisible against the
   dark theme. Inherit the surrounding theme text color and let Naive UI
   decide the actual hue; underline on hover keeps the affordance. */
:deep(.row-link) {
  color: inherit;
  text-decoration: none;
}
:deep(.row-link:hover) {
  text-decoration: underline;
  color: var(--n-text-color-hover, inherit);
}
.row-link {
  color: inherit;
  text-decoration: none;
}
.row-link:hover { text-decoration: underline; }
.m-line {
  display: flex;
  flex-wrap: wrap;
  gap: 8px 10px;
  align-items: center;
}
.muted { opacity: 0.7; font-size: 12px; }
</style>
