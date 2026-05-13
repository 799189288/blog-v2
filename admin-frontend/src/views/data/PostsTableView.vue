<script setup lang="ts">
import { computed, h, onMounted, ref } from 'vue'
import {
  NDataTable, NTag, NInput, NSelect, NSpace, NButton,
} from 'naive-ui'
import type { DataTableColumns, DataTableSortState } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import dayjs from 'dayjs'
import * as dataApi from '../../api/data'
import type { PostBrowseRow } from '../../types'
import RowDetailDrawer from '../../components/RowDetailDrawer.vue'
import { useDictStore } from '../../stores/dict'

const { t } = useI18n()
const dict = useDictStore()
const rows = ref<PostBrowseRow[]>([])
const total = ref(0)
const page = ref(1)
const perPage = ref(20)
const loading = ref(false)
const sort = ref<{ key: string; dir: 'asc' | 'desc' } | null>({ key: 'created_at', dir: 'desc' })
const q = ref('')
const status = ref<string | null>(null)
const drawer = ref(false)
const detail = ref<unknown>(null)

const statusOptions = computed(() => dict.options('post.status').value)

async function load() {
  loading.value = true
  try {
    const res = await dataApi.listPosts({
      page: page.value,
      per_page: perPage.value,
      sort: sort.value?.key,
      dir: sort.value?.dir,
      q: q.value || undefined,
      status: status.value ?? undefined,
    })
    rows.value = res.items
    total.value = res.total
  } finally {
    loading.value = false
  }
}

onMounted(async () => {
  await dict.ensure('post.status')
  await load()
})

function onSorterChange(s: DataTableSortState | null) {
  if (!s || !s.order) {
    sort.value = null
  } else {
    sort.value = { key: String(s.columnKey), dir: s.order === 'ascend' ? 'asc' : 'desc' }
  }
  page.value = 1
  load()
}

function openDetail(row: PostBrowseRow) {
  detail.value = row
  drawer.value = true
}

const columns = computed<DataTableColumns<PostBrowseRow>>(() => [
  { title: t('dataPosts.cols.id'), key: 'id', width: 70, sorter: true },
  { title: t('dataPosts.cols.title'), key: 'title', sorter: true, render: r => h('a', { onClick: () => openDetail(r), style: 'cursor:pointer' }, r.title) },
  { title: t('dataPosts.cols.slug'), key: 'slug', ellipsis: { tooltip: true } },
  {
    title: t('dataPosts.cols.status'), key: 'status', width: 110, sorter: true,
    render(r) {
      return h(NTag, { type: r.status === 'published' ? 'success' : 'default', size: 'small' }, () => dict.label('post.status', r.status))
    },
  },
  { title: t('dataPosts.cols.author'), key: 'author_id', width: 90 },
  { title: t('dataPosts.cols.views'), key: 'views', width: 90, sorter: true },
  { title: t('dataPosts.cols.published'), key: 'published_at', width: 150, sorter: true, render: r => r.published_at ? dayjs(r.published_at).format('YYYY-MM-DD HH:mm') : t('common.none') },
  { title: t('dataPosts.cols.created'), key: 'created_at', width: 150, sorter: true, render: r => dayjs(r.created_at).format('YYYY-MM-DD HH:mm') },
])
</script>

<template>
  <h2 style="margin-top: 0">{{ t('dataPosts.title') }}</h2>

  <NSpace style="margin-bottom: 12px;" :size="12" align="center">
    <NInput v-model:value="q" :placeholder="t('dataPosts.searchPlaceholder')" clearable style="width: 240px;" @update:value="() => { page = 1; load() }" />
    <NSelect v-model:value="status" :options="statusOptions" :placeholder="t('dataPosts.statusPlaceholder')" style="width: 160px;" clearable @update:value="() => { page = 1; load() }" />
    <NButton @click="load">{{ t('common.refresh') }}</NButton>
  </NSpace>

  <NDataTable
    remote
    :columns="columns"
    :data="rows"
    :loading="loading"
    :row-key="(r: PostBrowseRow) => r.id"
    :pagination="{
      page,
      pageSize: perPage,
      itemCount: total,
      pageSizes: [10, 20, 50, 100],
      showSizePicker: true,
      onChange: (p: number) => { page = p; load() },
      onUpdatePageSize: (size: number) => { perPage = size; page = 1; load() },
    }"
    @update:sorter="onSorterChange"
  />

  <RowDetailDrawer v-model:show="drawer" :title="t('dataPosts.drawerTitle')" :data="detail" />
</template>
