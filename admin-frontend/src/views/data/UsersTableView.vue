<script setup lang="ts">
import { computed, h, onMounted, ref } from 'vue'
import { NDataTable, NSpace, NButton, NTag } from 'naive-ui'
import type { DataTableColumns, DataTableSortState } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import dayjs from 'dayjs'
import * as dataApi from '../../api/data'
import type { UserRow } from '../../types'
import RowDetailDrawer from '../../components/RowDetailDrawer.vue'
import { useDictStore } from '../../stores/dict'

const { t } = useI18n()
const dict = useDictStore()
const rows = ref<UserRow[]>([])
const total = ref(0)
const page = ref(1)
const perPage = ref(20)
const loading = ref(false)
const sort = ref<{ key: string; dir: 'asc' | 'desc' } | null>({ key: 'id', dir: 'asc' })
const drawer = ref(false)
const detail = ref<unknown>(null)

async function load() {
  loading.value = true
  try {
    const res = await dataApi.listUsers({
      page: page.value, per_page: perPage.value,
      sort: sort.value?.key, dir: sort.value?.dir,
    })
    rows.value = res.items
    total.value = res.total
  } finally { loading.value = false }
}

onMounted(async () => {
  await dict.ensure('user.role')
  await load()
})

function onSorterChange(s: DataTableSortState | null) {
  if (!s || !s.order) sort.value = null
  else sort.value = { key: String(s.columnKey), dir: s.order === 'ascend' ? 'asc' : 'desc' }
  page.value = 1
  load()
}

function openDetail(row: UserRow) { detail.value = row; drawer.value = true }

const columns = computed<DataTableColumns<UserRow>>(() => [
  { title: t('dataUsers.cols.id'), key: 'id', width: 70, sorter: true },
  { title: t('dataUsers.cols.username'), key: 'username', sorter: true, render: r => h('a', { onClick: () => openDetail(r), style: 'cursor:pointer' }, r.username) },
  { title: t('dataUsers.cols.role'), key: 'role', width: 120, sorter: true, render: r => h(NTag, { size: 'small', type: 'info' }, () => dict.label('user.role', r.role)) },
  { title: t('dataUsers.cols.created'), key: 'created_at', width: 160, sorter: true, render: r => dayjs(r.created_at).format('YYYY-MM-DD HH:mm') },
])
</script>

<template>
  <h2 style="margin-top: 0">{{ t('dataUsers.title') }}</h2>
  <NSpace style="margin-bottom: 12px;">
    <NButton @click="load">{{ t('common.refresh') }}</NButton>
  </NSpace>
  <NDataTable
    remote
    :columns="columns"
    :data="rows"
    :loading="loading"
    :row-key="(r: UserRow) => r.id"
    :pagination="{
      page, pageSize: perPage, itemCount: total,
      pageSizes: [10, 20, 50, 100], showSizePicker: true,
      onChange: (p: number) => { page = p; load() },
      onUpdatePageSize: (size: number) => { perPage = size; page = 1; load() },
    }"
    @update:sorter="onSorterChange"
  />
  <RowDetailDrawer v-model:show="drawer" :title="t('dataUsers.drawerTitle')" :data="detail" />
</template>
