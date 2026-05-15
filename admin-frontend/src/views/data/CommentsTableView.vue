<script setup lang="ts">
import { computed, h, onMounted, ref } from 'vue'
import { NDataTable, NTag, NInput, NSelect, NInputNumber, NSpace, NButton } from 'naive-ui'
import type { DataTableColumns, DataTableSortState } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import dayjs from 'dayjs'
import * as dataApi from '../../api/data'
import type { CommentBrowseRow } from '../../types'
import RowDetailDrawer from '../../components/RowDetailDrawer.vue'
import MobileRowCard from '../../components/MobileRowCard.vue'
import { useDictStore } from '../../stores/dict'
import { useBreakpoint } from '../../composables/useBreakpoint'

const { t } = useI18n()
const dict = useDictStore()
const { isMobile } = useBreakpoint()
const rows = ref<CommentBrowseRow[]>([])
const total = ref(0)
const page = ref(1)
const perPage = ref(20)
const loading = ref(false)
const sort = ref<{ key: string; dir: 'asc' | 'desc' } | null>({ key: 'created_at', dir: 'desc' })
const q = ref('')
const status = ref<string | null>(null)
const postId = ref<number | null>(null)
const drawer = ref(false)
const detail = ref<unknown>(null)

const statusOptions = computed(() => dict.options('comment.status').value)

async function load() {
  loading.value = true
  try {
    const res = await dataApi.listComments({
      page: page.value,
      per_page: perPage.value,
      sort: sort.value?.key,
      dir: sort.value?.dir,
      q: q.value || undefined,
      status: status.value ?? undefined,
      post_id: postId.value ?? undefined,
    })
    rows.value = res.items
    total.value = res.total
  } finally {
    loading.value = false
  }
}

onMounted(async () => {
  await dict.ensure('comment.status')
  await load()
})

function onSorterChange(s: DataTableSortState | null) {
  if (!s || !s.order) sort.value = null
  else sort.value = { key: String(s.columnKey), dir: s.order === 'ascend' ? 'asc' : 'desc' }
  page.value = 1
  load()
}

function openDetail(row: CommentBrowseRow) {
  detail.value = row
  drawer.value = true
}

const columns = computed<DataTableColumns<CommentBrowseRow>>(() => [
  { title: t('dataComments.cols.id'), key: 'id', width: 70, sorter: true },
  { title: t('dataComments.cols.postId'), key: 'post_id', width: 80, sorter: true },
  { title: t('dataComments.cols.author'), key: 'author_name', sorter: true, width: 140, render: r => h('a', { onClick: () => openDetail(r), style: 'cursor:pointer' }, r.author_name) },
  {
    title: t('dataComments.cols.replyTo'), key: 'parent_author_name', width: 140,
    render(r) {
      if (!r.parent_id) return h('span', { style: 'opacity:.4' }, '—')
      const name = r.parent_author_name ?? `#${r.parent_id}`
      return h('span', null, `@${name}`)
    },
  },
  { title: t('dataComments.cols.content'), key: 'content', ellipsis: { tooltip: true } },
  {
    title: t('dataComments.cols.status'), key: 'status', width: 110, sorter: true,
    render(r) {
      const type = r.status === 'approved' ? 'success' : r.status === 'spam' ? 'error' : 'warning'
      return h(NTag, { type, size: 'small' }, () => dict.label('comment.status', r.status))
    },
  },
  { title: t('dataComments.cols.created'), key: 'created_at', width: 160, sorter: true, render: r => dayjs(r.created_at).format('YYYY-MM-DD HH:mm') },
])
</script>

<template>
  <h2 class="page-title">{{ t('dataComments.title') }}</h2>
  <NSpace class="filter-row" :size="12" align="center" :wrap="true">
    <NInput v-model:value="q" :placeholder="t('dataComments.searchPlaceholder')" clearable class="filter-input" @update:value="() => { page = 1; load() }" />
    <NSelect v-model:value="status" :options="statusOptions" :placeholder="t('dataComments.statusPlaceholder')" class="filter-select" clearable @update:value="() => { page = 1; load() }" />
    <NInputNumber v-model:value="postId" :placeholder="t('dataComments.postIdPlaceholder')" clearable :show-button="false" class="filter-num" @update:value="() => { page = 1; load() }" />
    <NButton @click="load">{{ t('common.refresh') }}</NButton>
  </NSpace>

  <NDataTable
    v-if="!isMobile"
    remote
    :columns="columns"
    :data="rows"
    :loading="loading"
    :row-key="(r: CommentBrowseRow) => r.id"
    :pagination="{
      page, pageSize: perPage, itemCount: total,
      pageSizes: [10, 20, 50, 100], showSizePicker: true,
      onChange: (p: number) => { page = p; load() },
      onUpdatePageSize: (size: number) => { perPage = size; page = 1; load() },
    }"
    @update:sorter="onSorterChange"
  />
  <div v-else class="m-list">
    <MobileRowCard v-for="r in rows" :key="r.id">
      <template #title>
        {{ r.author_name }}
        <span class="m-when">{{ dayjs(r.created_at).format('YYYY-MM-DD HH:mm') }}</span>
      </template>
      <template #body>
        <div class="m-content">{{ r.content }}</div>
        <div class="m-line">
          <NTag
            :type="r.status === 'approved' ? 'success' : r.status === 'spam' ? 'error' : 'warning'"
            size="small"
          >
            {{ dict.label('comment.status', r.status) }}
          </NTag>
          <span class="muted">post #{{ r.post_id }}</span>
          <span v-if="r.parent_id" class="muted">@{{ r.parent_author_name ?? `#${r.parent_id}` }}</span>
        </div>
      </template>
      <template #actions>
        <NButton size="small" @click="openDetail(r)">{{ t('common.viewDetails') }}</NButton>
      </template>
    </MobileRowCard>
    <div class="mobile-pager">
      <NButton size="small" :disabled="page <= 1" @click="page = page - 1; load()">‹</NButton>
      <span class="page-info">{{ page }} / {{ Math.max(1, Math.ceil(total / perPage)) }}</span>
      <NButton size="small" :disabled="page * perPage >= total" @click="page = page + 1; load()">›</NButton>
    </div>
  </div>

  <RowDetailDrawer v-model:show="drawer" :title="t('dataComments.drawerTitle')" :data="detail" />
</template>

<style scoped>
.page-title { margin-top: 0; }
.filter-row { margin-bottom: 12px; }
.filter-input { width: 240px; max-width: 100%; }
.filter-select { width: 160px; max-width: 100%; }
.filter-num { width: 120px; max-width: 100%; }
.m-when { font-size: 12px; opacity: 0.65; font-weight: 400; margin-left: 6px; }
.m-content {
  white-space: pre-wrap;
  word-break: break-word;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
.m-line { display: flex; flex-wrap: wrap; gap: 8px 10px; align-items: center; }
.muted { opacity: 0.7; font-size: 12px; }
.mobile-pager {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 12px;
  margin-top: 12px;
}
.page-info { font-size: 13px; }
@media (max-width: 480px) {
  .filter-input,
  .filter-select,
  .filter-num { width: 100%; }
}
</style>
