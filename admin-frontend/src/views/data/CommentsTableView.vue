<script setup lang="ts">
import { h, onMounted, ref } from 'vue'
import { NDataTable, NTag, NInput, NSelect, NInputNumber, NSpace, NButton } from 'naive-ui'
import type { DataTableColumns, DataTableSortState } from 'naive-ui'
import dayjs from 'dayjs'
import * as dataApi from '../../api/data'
import type { CommentBrowseRow } from '../../types'
import RowDetailDrawer from '../../components/RowDetailDrawer.vue'

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

const statusOptions = [
  { label: 'Pending', value: 'pending' },
  { label: 'Approved', value: 'approved' },
  { label: 'Spam', value: 'spam' },
]

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

onMounted(load)

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

const columns: DataTableColumns<CommentBrowseRow> = [
  { title: 'ID', key: 'id', width: 70, sorter: true },
  { title: 'Post ID', key: 'post_id', width: 80, sorter: true },
  { title: 'Author', key: 'author_name', sorter: true, width: 140, render: r => h('a', { onClick: () => openDetail(r), style: 'cursor:pointer' }, r.author_name) },
  { title: 'Content', key: 'content', ellipsis: { tooltip: true } },
  {
    title: 'Status', key: 'status', width: 110, sorter: true,
    render(r) {
      const type = r.status === 'approved' ? 'success' : r.status === 'spam' ? 'error' : 'warning'
      return h(NTag, { type, size: 'small' }, () => r.status)
    },
  },
  { title: 'Created', key: 'created_at', width: 160, sorter: true, render: r => dayjs(r.created_at).format('YYYY-MM-DD HH:mm') },
]
</script>

<template>
  <h2 style="margin-top: 0">Comments</h2>
  <NSpace style="margin-bottom: 12px;" :size="12" align="center">
    <NInput v-model:value="q" placeholder="Search author / content..." clearable style="width: 240px;" @update:value="() => { page = 1; load() }" />
    <NSelect v-model:value="status" :options="statusOptions" placeholder="Status" style="width: 160px;" clearable @update:value="() => { page = 1; load() }" />
    <NInputNumber v-model:value="postId" placeholder="Post ID" clearable :show-button="false" style="width: 120px;" @update:value="() => { page = 1; load() }" />
    <NButton @click="load">Refresh</NButton>
  </NSpace>

  <NDataTable
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

  <RowDetailDrawer v-model:show="drawer" title="Comment row" :data="detail" />
</template>
