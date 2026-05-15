<script setup lang="ts">
import { computed, h, onMounted, ref } from 'vue'
import {
  NDataTable, NTag, NSelect, NDatePicker, NSpace, NButton,
} from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import dayjs from 'dayjs'
import * as auditApi from '../api/audit'
import * as usersApi from '../api/users'
import type { AuditRow, UserRow } from '../types'
import RowDetailDrawer from '../components/RowDetailDrawer.vue'
import MobileRowCard from '../components/MobileRowCard.vue'
import { useDictStore } from '../stores/dict'
import { useBreakpoint } from '../composables/useBreakpoint'

const { t } = useI18n()
const dict = useDictStore()
const { isMobile } = useBreakpoint()
const rows = ref<AuditRow[]>([])
const total = ref(0)
const page = ref(1)
const perPage = ref(50)
const loading = ref(false)

const userId = ref<number | null>(null)
const action = ref<string | null>(null)
const range = ref<[number, number] | null>(null)

const users = ref<UserRow[]>([])

const actionOptions = computed(() => dict.options('audit.action').value)

const drawer = ref(false)
const detail = ref<unknown>(null)

async function loadUsers() {
  users.value = await usersApi.list()
}

async function load() {
  loading.value = true
  try {
    const res = await auditApi.list({
      page: page.value,
      per_page: perPage.value,
      user_id: userId.value ?? undefined,
      action: action.value ?? undefined,
      from: range.value ? new Date(range.value[0]).toISOString() : undefined,
      to: range.value ? new Date(range.value[1]).toISOString() : undefined,
    })
    rows.value = res.items
    total.value = res.total
  } finally { loading.value = false }
}

onMounted(async () => {
  await dict.ensure('audit.action')
  await loadUsers()
  await load()
})

function userOptions() {
  return users.value.map(u => ({ label: u.username, value: u.id }))
}

function openDetail(row: AuditRow) {
  detail.value = row
  drawer.value = true
}

function applyFilters() {
  page.value = 1
  load()
}

function resetFilters() {
  userId.value = null
  action.value = null
  range.value = null
  page.value = 1
  load()
}

const columns = computed<DataTableColumns<AuditRow>>(() => [
  { title: t('audit.cols.id'), key: 'id', width: 70 },
  { title: t('audit.cols.when'), key: 'created_at', width: 160, render: r => dayjs(r.created_at).format('YYYY-MM-DD HH:mm:ss') },
  { title: t('audit.cols.user'), key: 'username', width: 140 },
  {
    title: t('audit.cols.action'), key: 'action', width: 180,
    render: r => h(NTag, { size: 'small', type: actionType(r.action) }, () => dict.label('audit.action', r.action, t('audit.unknown'))),
  },
  { title: t('audit.cols.target'), key: 'target', width: 140, render: r => r.target_type ? `${r.target_type}#${r.target_id ?? '?'}` : t('common.none') },
  { title: t('audit.cols.ip'), key: 'ip', width: 130, render: r => r.ip ?? t('common.none') },
  {
    title: t('audit.cols.detail'), key: 'detail',
    render(r) {
      if (!r.detail) return h('span', { style: 'opacity:.5' }, t('common.none'))
      return h('a', { onClick: () => openDetail(r), style: 'cursor:pointer' }, t('common.view'))
    },
  },
])

function actionType(a: string): 'default' | 'success' | 'warning' | 'error' | 'info' {
  if (a === 'login') return 'info'
  if (a.endsWith('.delete')) return 'error'
  if (a.endsWith('.create')) return 'success'
  return 'default'
}
</script>

<template>
  <h2 class="page-title">{{ t('audit.title') }}</h2>

  <NSpace class="filter-row" :size="12" align="center" :wrap="true">
    <NSelect v-model:value="userId" :options="userOptions()" :placeholder="t('audit.user')" clearable class="filter-user" />
    <NSelect v-model:value="action" :options="actionOptions" :placeholder="t('audit.action')" clearable class="filter-action" />
    <NDatePicker v-model:value="range" type="daterange" clearable class="filter-date" />
    <NButton type="primary" @click="applyFilters">{{ t('common.apply') }}</NButton>
    <NButton @click="resetFilters">{{ t('common.reset') }}</NButton>
  </NSpace>

  <NDataTable
    v-if="!isMobile"
    remote
    :columns="columns"
    :data="rows"
    :loading="loading"
    :row-key="(r: AuditRow) => r.id"
    :pagination="{
      page, pageSize: perPage, itemCount: total,
      pageSizes: [25, 50, 100, 200], showSizePicker: true,
      onChange: (p: number) => { page = p; load() },
      onUpdatePageSize: (size: number) => { perPage = size; page = 1; load() },
    }"
  />
  <div v-else class="m-list">
    <MobileRowCard v-for="r in rows" :key="r.id">
      <template #title>
        <NTag size="small" :type="actionType(r.action)">
          {{ dict.label('audit.action', r.action, t('audit.unknown')) }}
        </NTag>
        <span class="m-when">{{ dayjs(r.created_at).format('YYYY-MM-DD HH:mm:ss') }}</span>
      </template>
      <template #body>
        <div class="m-line">
          <span class="muted">{{ r.username }}</span>
          <span v-if="r.target_type" class="muted">{{ r.target_type }}#{{ r.target_id ?? '?' }}</span>
          <span v-if="r.ip" class="muted">{{ r.ip }}</span>
        </div>
      </template>
      <template #actions>
        <NButton v-if="r.detail" size="small" @click="openDetail(r)">{{ t('common.view') }}</NButton>
      </template>
    </MobileRowCard>
    <div class="mobile-pager">
      <NButton size="small" :disabled="page <= 1" @click="page = page - 1; load()">‹</NButton>
      <span class="page-info">{{ page }} / {{ Math.max(1, Math.ceil(total / perPage)) }}</span>
      <NButton size="small" :disabled="page * perPage >= total" @click="page = page + 1; load()">›</NButton>
    </div>
  </div>

  <RowDetailDrawer v-model:show="drawer" :title="t('audit.drawerTitle')" :data="detail" />
</template>

<style scoped>
.page-title { margin-top: 0; }
.filter-row { margin-bottom: 12px; }
.filter-user { width: 200px; max-width: 100%; }
.filter-action { width: 220px; max-width: 100%; }
.filter-date { max-width: 100%; }
.m-when { font-size: 12px; opacity: 0.65; font-weight: 400; margin-left: 8px; }
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
  .filter-user,
  .filter-action { width: 100%; }
}
</style>
