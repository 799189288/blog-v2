<script setup lang="ts">
import { computed, h, onMounted, ref } from 'vue'
import { NDataTable, NInput, NSpace, NButton, NTag } from 'naive-ui'
import type { DataTableColumns, DataTableSortState } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import * as dataApi from '../../api/data'
import type { TagBrowseRow } from '../../types'
import RowDetailDrawer from '../../components/RowDetailDrawer.vue'
import MobileRowCard from '../../components/MobileRowCard.vue'
import { useBreakpoint } from '../../composables/useBreakpoint'

const { t } = useI18n()
const { isMobile } = useBreakpoint()
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
  <h2 class="page-title">{{ t('dataTags.title') }}</h2>
  <NSpace class="filter-row" :size="12" align="center" :wrap="true">
    <NInput v-model:value="q" :placeholder="t('dataTags.searchPlaceholder')" clearable class="filter-input" @update:value="() => { page = 1; load() }" />
    <NButton @click="load">{{ t('common.refresh') }}</NButton>
  </NSpace>
  <NDataTable
    v-if="!isMobile"
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
  <div v-else class="m-list">
    <MobileRowCard v-for="r in rows" :key="r.id">
      <template #title>{{ r.name }}</template>
      <template #body>
        <div class="m-line">
          <span class="muted slug">{{ r.slug }}</span>
          <NTag size="small" :type="r.post_count > 0 ? 'info' : 'default'">
            {{ r.post_count }}
          </NTag>
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
  <RowDetailDrawer v-model:show="drawer" :title="t('dataTags.drawerTitle')" :data="detail" />
</template>

<style scoped>
.page-title { margin-top: 0; }
.filter-row { margin-bottom: 12px; }
.filter-input { width: 240px; max-width: 100%; }
.m-line { display: flex; flex-wrap: wrap; gap: 8px 10px; align-items: center; }
.muted { opacity: 0.7; font-size: 12px; }
.slug { font-family: ui-monospace, "Cascadia Mono", Menlo, monospace; word-break: break-all; }
.mobile-pager {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 12px;
  margin-top: 12px;
}
.page-info { font-size: 13px; }
@media (max-width: 480px) {
  .filter-input { width: 100%; }
}
</style>
