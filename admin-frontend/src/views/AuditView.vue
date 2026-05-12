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
import { useDictStore } from '../stores/dict'

const { t } = useI18n()
const dict = useDictStore()
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
  <h2 style="margin-top: 0">{{ t('audit.title') }}</h2>

  <NSpace style="margin-bottom: 12px;" :size="12" align="center" wrap>
    <NSelect v-model:value="userId" :options="userOptions()" :placeholder="t('audit.user')" clearable style="width: 200px;" />
    <NSelect v-model:value="action" :options="actionOptions" :placeholder="t('audit.action')" clearable style="width: 220px;" />
    <NDatePicker v-model:value="range" type="daterange" clearable />
    <NButton type="primary" @click="applyFilters">{{ t('common.apply') }}</NButton>
    <NButton @click="resetFilters">{{ t('common.reset') }}</NButton>
  </NSpace>

  <NDataTable
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

  <RowDetailDrawer v-model:show="drawer" :title="t('audit.drawerTitle')" :data="detail" />
</template>
