<script setup lang="ts">
import { computed, h, onMounted, ref } from 'vue'
import { NDataTable, NInput, NSpace, NButton } from 'naive-ui'
import type { DataTableColumns, DataTableSortState } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import * as dataApi from '../../api/data'
import type { TagBrowseRow } from '../../types'
import RowDetailDrawer from '../../components/RowDetailDrawer.vue'

const { t } = useI18n()
const rows = ref<TagBrowseRow[]>([])
const total = ref(0)
const page = ref(1)
const perPage = ref(20)
const loading = ref(false)
const sort = ref<{ key: string; dir: 'asc' | 'desc' } | null>({ key: 'name', dir: 'asc' })
const q = ref('')
const drawer = ref(false)
const detail = ref<unknown>(null)

async function load() {
  loading.value = true
  try {
    const res = await dataApi.listTags({
      page: page.value, per_page: perPage.value,
      sort: sort.value?.key, dir: sort.value?.dir,
      q: q.value || undefined,
    })
    rows.value = res.items
    total.value = res.total
  } finally { loading.value = false }
}

onMounted(load)

function onSorterChange(s: DataTableSortState | null) {
  if (!s || !s.order) sort.value = null
  else sort.value = { key: String(s.columnKey), dir: s.order === 'ascend' ? 'asc' : 'desc' }
  page.value = 1
  load()
}

function openDetail(row: TagBrowseRow) { detail.value = row; drawer.value = true }

const columns = computed<DataTableColumns<TagBrowseRow>>(() => [
  { title: t('dataTags.cols.id'), key: 'id', width: 80, sorter: true },
  { title: t('dataTags.cols.name'), key: 'name', sorter: true, render: r => h('a', { onClick: () => openDetail(r), style: 'cursor:pointer' }, r.name) },
  { title: t('dataTags.cols.slug'), key: 'slug', sorter: true },
  { title: t('dataTags.cols.posts'), key: 'post_count', width: 100, sorter: true },
])
</script>

<template>
  <h2 style="margin-top: 0">{{ t('dataTags.title') }}</h2>
  <NSpace style="margin-bottom: 12px;" :size="12" align="center">
    <NInput v-model:value="q" :placeholder="t('dataTags.searchPlaceholder')" clearable style="width: 240px;" @update:value="() => { page = 1; load() }" />
    <NButton @click="load">{{ t('common.refresh') }}</NButton>
  </NSpace>
  <NDataTable
    remote
    :columns="columns"
    :data="rows"
    :loading="loading"
    :row-key="(r: TagBrowseRow) => r.id"
    :pagination="{
      page, pageSize: perPage, itemCount: total,
      pageSizes: [10, 20, 50, 100], showSizePicker: true,
      onChange: (p: number) => { page = p; load() },
      onUpdatePageSize: (size: number) => { perPage = size; page = 1; load() },
    }"
    @update:sorter="onSorterChange"
  />
  <RowDetailDrawer v-model:show="drawer" :title="t('dataTags.drawerTitle')" :data="detail" />
</template>
