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
import MobileRowCard from '../../components/MobileRowCard.vue'
import { useDictStore } from '../../stores/dict'
import { useBreakpoint } from '../../composables/useBreakpoint'

const { t } = useI18n()
const dict = useDictStore()
const { isMobile } = useBreakpoint()
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
  <h2 class="page-title">{{ t('dataPosts.title') }}</h2>

  <NSpace class="filter-row" :size="12" align="center" :wrap="true">
    <NInput v-model:value="q" :placeholder="t('dataPosts.searchPlaceholder')" clearable class="filter-input" @update:value="() => { page = 1; load() }" />
    <NSelect v-model:value="status" :options="statusOptions" :placeholder="t('dataPosts.statusPlaceholder')" class="filter-select" clearable @update:value="() => { page = 1; load() }" />
    <NButton @click="load">{{ t('common.refresh') }}</NButton>
  </NSpace>

  <NDataTable
    v-if="!isMobile"
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
  <div v-else class="m-list">
    <MobileRowCard v-for="r in rows" :key="r.id">
      <template #title>{{ r.title }}</template>
      <template #body>
        <div class="muted slug">{{ r.slug }}</div>
        <div class="m-line">
          <NTag :type="r.status === 'published' ? 'success' : 'default'" size="small">
            {{ dict.label('post.status', r.status) }}
          </NTag>
          <span class="muted">{{ t('dashboard.viewsCount', { n: r.views }) }}</span>
          <span class="muted">{{ dayjs(r.created_at).format('YYYY-MM-DD') }}</span>
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

  <RowDetailDrawer v-model:show="drawer" :title="t('dataPosts.drawerTitle')" :data="detail" />
</template>

<style scoped>
.page-title { margin-top: 0; }
.filter-row { margin-bottom: 12px; }
.filter-input { width: 240px; max-width: 100%; }
.filter-select { width: 160px; max-width: 100%; }
.slug {
  font-family: ui-monospace, "Cascadia Mono", Menlo, monospace;
  word-break: break-all;
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
  .filter-select { width: 100%; }
}
</style>
