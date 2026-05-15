<script setup lang="ts">
import { computed, h, onMounted, ref } from 'vue'
import { NDataTable, NSpace, NButton, NTag } from 'naive-ui'
import type { DataTableColumns, DataTableSortState } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import dayjs from 'dayjs'
import * as dataApi from '../../api/data'
import type { UserRow } from '../../types'
import RowDetailDrawer from '../../components/RowDetailDrawer.vue'
import MobileRowCard from '../../components/MobileRowCard.vue'
import { useDictStore } from '../../stores/dict'
import { useBreakpoint } from '../../composables/useBreakpoint'

const { t } = useI18n()
const dict = useDictStore()
const { isMobile } = useBreakpoint()
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
  <h2 class="page-title">{{ t('dataUsers.title') }}</h2>
  <NSpace style="margin-bottom: 12px;">
    <NButton @click="load">{{ t('common.refresh') }}</NButton>
  </NSpace>
  <NDataTable
    v-if="!isMobile"
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
  <div v-else class="m-list">
    <MobileRowCard v-for="r in rows" :key="r.id">
      <template #title>{{ r.username }}</template>
      <template #body>
        <div class="m-line">
          <NTag size="small" type="info">{{ dict.label('user.role', r.role) }}</NTag>
          <span class="muted">{{ dayjs(r.created_at).format('YYYY-MM-DD HH:mm') }}</span>
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
  <RowDetailDrawer v-model:show="drawer" :title="t('dataUsers.drawerTitle')" :data="detail" />
</template>

<style scoped>
.page-title { margin-top: 0; }
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
</style>
