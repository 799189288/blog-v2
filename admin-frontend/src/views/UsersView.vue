<script setup lang="ts">
import { computed, h, onMounted, ref } from 'vue'
import {
  NDataTable, NTag, NButton, NSpace, NModal, NForm, NFormItem, NInput,
  useDialog, useMessage,
} from 'naive-ui'
import type { DataTableColumns, DataTableSortState } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import dayjs from 'dayjs'
import * as usersApi from '../api/users'
import type { UserRow } from '../types'
import { useAuthStore } from '../stores/auth'
import { useDictStore } from '../stores/dict'
import { useBreakpoint } from '../composables/useBreakpoint'
import MobileRowCard from '../components/MobileRowCard.vue'

const auth = useAuthStore()
const dialog = useDialog()
const message = useMessage()
const { t } = useI18n()
const dict = useDictStore()
const { isMobile } = useBreakpoint()

const rows = ref<UserRow[]>([])
const loading = ref(false)
const sortState = ref<{ key: string; order: 'ascend' | 'descend' } | null>(null)

const sorted = computed(() => {
  if (!sortState.value) return rows.value
  const { key, order } = sortState.value
  return [...rows.value].sort((a, b) => {
    const av = (a as any)[key] ?? ''
    const bv = (b as any)[key] ?? ''
    const cmp = String(av).localeCompare(String(bv))
    return order === 'ascend' ? cmp : -cmp
  })
})

const createOpen = ref(false)
const createForm = ref({ username: '', password: '' })
const creating = ref(false)

const resetOpen = ref(false)
const resetTarget = ref<UserRow | null>(null)
const resetPwd = ref('')
const resetting = ref(false)

async function load() {
  loading.value = true
  try {
    rows.value = await usersApi.list()
  } finally { loading.value = false }
}

onMounted(async () => {
  await dict.ensure('user.role')
  await load()
})

async function doCreate() {
  if (!createForm.value.username.trim() || createForm.value.password.length < 8) {
    message.warning(t('users.createRequiredError'))
    return
  }
  creating.value = true
  try {
    await usersApi.create(createForm.value.username.trim(), createForm.value.password)
    message.success(t('users.createSuccess'))
    createOpen.value = false
    createForm.value = { username: '', password: '' }
    await load()
  } catch (e: any) {
    message.error(e?.response?.data?.error ?? t('users.createFailed'))
  } finally { creating.value = false }
}

function openReset(u: UserRow) {
  resetTarget.value = u
  resetPwd.value = ''
  resetOpen.value = true
}

async function doReset() {
  if (!resetTarget.value || resetPwd.value.length < 8) {
    message.warning(t('users.pwdLengthError'))
    return
  }
  resetting.value = true
  try {
    await usersApi.resetPassword(resetTarget.value.id, resetPwd.value)
    message.success(t('users.resetSuccess', { name: resetTarget.value.username }))
    resetOpen.value = false
  } catch (e: any) {
    message.error(e?.response?.data?.error ?? t('users.resetFailed'))
  } finally { resetting.value = false }
}

function confirmDelete(u: UserRow) {
  dialog.warning({
    title: t('users.deleteTitle'),
    content: t('users.deleteConfirm', { name: u.username }),
    positiveText: t('common.confirmDelete'),
    negativeText: t('common.cancel'),
    onPositiveClick: async () => {
      try {
        await usersApi.remove(u.id)
        message.success(t('common.deleted'))
        await load()
      } catch (e: any) {
        message.error(e?.response?.data?.error ?? t('common.deleteFailed'))
      }
    },
  })
}

function onSorterChange(s: DataTableSortState | null) {
  if (!s || !s.order) sortState.value = null
  else sortState.value = { key: String(s.columnKey), order: s.order }
}

const columns = computed<DataTableColumns<UserRow>>(() => [
  { title: t('users.cols.id'), key: 'id', width: 70, sorter: true },
  {
    title: t('users.cols.username'), key: 'username', sorter: true,
    render(r) {
      const isMe = auth.user?.id === r.id
      return h('span', null, [r.username, isMe ? h(NTag, { size: 'small', type: 'info', style: 'margin-left:8px' }, () => t('common.you')) : null])
    },
  },
  { title: t('users.cols.role'), key: 'role', width: 120, sorter: true, render: r => h(NTag, { size: 'small', type: 'info' }, () => dict.label('user.role', r.role)) },
  { title: t('users.cols.created'), key: 'created_at', width: 160, sorter: true, render: r => dayjs(r.created_at).format('YYYY-MM-DD HH:mm') },
  {
    title: t('users.cols.actions'), key: 'actions', width: 260,
    render(r) {
      const isMe = auth.user?.id === r.id
      return h(NSpace, { size: 'small' }, () => [
        h(NButton, { size: 'small', onClick: () => openReset(r) }, () => t('users.resetPassword')),
        h(
          NButton,
          { size: 'small', type: 'error', ghost: true, disabled: isMe, onClick: () => confirmDelete(r) },
          () => t('common.delete'),
        ),
      ])
    },
  },
])
</script>

<template>
  <div class="page-header">
    <h2 class="page-title">{{ t('users.title') }}</h2>
    <NButton type="primary" @click="createOpen = true">{{ t('users.newUser') }}</NButton>
  </div>

  <NDataTable
    v-if="!isMobile"
    :columns="columns"
    :data="sorted"
    :loading="loading"
    :row-key="(r: UserRow) => r.id"
    @update:sorter="onSorterChange"
  />
  <div v-else class="m-list">
    <MobileRowCard v-for="r in sorted" :key="r.id">
      <template #title>
        {{ r.username }}
        <NTag v-if="auth.user?.id === r.id" size="small" type="info" class="me-tag">
          {{ t('common.you') }}
        </NTag>
      </template>
      <template #body>
        <div class="m-line">
          <NTag size="small" type="info">{{ dict.label('user.role', r.role) }}</NTag>
          <span class="muted">{{ dayjs(r.created_at).format('YYYY-MM-DD HH:mm') }}</span>
        </div>
      </template>
      <template #actions>
        <NButton size="small" @click="openReset(r)">{{ t('users.resetPassword') }}</NButton>
        <NButton
          size="small"
          type="error"
          ghost
          :disabled="auth.user?.id === r.id"
          @click="confirmDelete(r)"
        >
          {{ t('common.delete') }}
        </NButton>
      </template>
    </MobileRowCard>
  </div>

  <NModal
    v-model:show="createOpen"
    preset="card"
    :title="t('users.createTitle')"
    :style="{ width: isMobile ? '94vw' : '420px', maxWidth: '94vw' }"
  >
    <NForm label-placement="top">
      <NFormItem :label="t('users.usernameLabel')">
        <NInput v-model:value="createForm.username" />
      </NFormItem>
      <NFormItem :label="t('users.passwordLabel')">
        <NInput v-model:value="createForm.password" type="password" show-password-on="click" />
      </NFormItem>
      <NSpace justify="end">
        <NButton @click="createOpen = false">{{ t('common.cancel') }}</NButton>
        <NButton type="primary" :loading="creating" @click="doCreate">{{ t('common.create') }}</NButton>
      </NSpace>
    </NForm>
  </NModal>

  <NModal
    v-model:show="resetOpen"
    preset="card"
    :title="t('users.resetTitle', { name: resetTarget?.username ?? '' })"
    :style="{ width: isMobile ? '94vw' : '420px', maxWidth: '94vw' }"
  >
    <NForm label-placement="top">
      <NFormItem :label="t('users.newPasswordLabel')">
        <NInput v-model:value="resetPwd" type="password" show-password-on="click" />
      </NFormItem>
      <NSpace justify="end">
        <NButton @click="resetOpen = false">{{ t('common.cancel') }}</NButton>
        <NButton type="primary" :loading="resetting" @click="doReset">{{ t('users.resetPassword') }}</NButton>
      </NSpace>
    </NForm>
  </NModal>
</template>

<style scoped>
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
  flex-wrap: wrap;
  gap: 8px;
}
.page-title { margin: 0; }
.me-tag { margin-left: 8px; }
.m-line { display: flex; flex-wrap: wrap; gap: 8px 10px; align-items: center; }
.muted { opacity: 0.7; font-size: 12px; }
</style>


const auth = useAuthStore()
const dialog = useDialog()
const message = useMessage()
const { t } = useI18n()
const dict = useDictStore()
const { isMobile } = useBreakpoint()

const rows = ref<UserRow[]>([])
const loading = ref(false)

const createOpen = ref(false)
const createForm = ref({ username: '', password: '' })
const creating = ref(false)

const resetOpen = ref(false)
const resetTarget = ref<UserRow | null>(null)
const resetPwd = ref('')
const resetting = ref(false)

async function load() {
  loading.value = true
  try {
    rows.value = await usersApi.list()
  } finally { loading.value = false }
}

onMounted(async () => {
  await dict.ensure('user.role')
  await load()
})

async function doCreate() {
  if (!createForm.value.username.trim() || createForm.value.password.length < 8) {
    message.warning(t('users.createRequiredError'))
    return
  }
  creating.value = true
  try {
    await usersApi.create(createForm.value.username.trim(), createForm.value.password)
    message.success(t('users.createSuccess'))
    createOpen.value = false
    createForm.value = { username: '', password: '' }
    await load()
  } catch (e: any) {
    message.error(e?.response?.data?.error ?? t('users.createFailed'))
  } finally { creating.value = false }
}

function openReset(u: UserRow) {
  resetTarget.value = u
  resetPwd.value = ''
  resetOpen.value = true
}

async function doReset() {
  if (!resetTarget.value || resetPwd.value.length < 8) {
    message.warning(t('users.pwdLengthError'))
    return
  }
  resetting.value = true
  try {
    await usersApi.resetPassword(resetTarget.value.id, resetPwd.value)
    message.success(t('users.resetSuccess', { name: resetTarget.value.username }))
    resetOpen.value = false
  } catch (e: any) {
    message.error(e?.response?.data?.error ?? t('users.resetFailed'))
  } finally { resetting.value = false }
}

function confirmDelete(u: UserRow) {
  dialog.warning({
    title: t('users.deleteTitle'),
    content: t('users.deleteConfirm', { name: u.username }),
    positiveText: t('common.confirmDelete'),
    negativeText: t('common.cancel'),
    onPositiveClick: async () => {
      try {
        await usersApi.remove(u.id)
        message.success(t('common.deleted'))
        await load()
      } catch (e: any) {
        message.error(e?.response?.data?.error ?? t('common.deleteFailed'))
      }
    },
  })
}

const columns = computed<DataTableColumns<UserRow>>(() => [
  { title: t('users.cols.id'), key: 'id', width: 70 },
  {
    title: t('users.cols.username'), key: 'username',
    render(r) {
      const isMe = auth.user?.id === r.id
      return h('span', null, [r.username, isMe ? h(NTag, { size: 'small', type: 'info', style: 'margin-left:8px' }, () => t('common.you')) : null])
    },
  },
  { title: t('users.cols.role'), key: 'role', width: 120, render: r => h(NTag, { size: 'small', type: 'info' }, () => dict.label('user.role', r.role)) },
  { title: t('users.cols.created'), key: 'created_at', width: 160, render: r => dayjs(r.created_at).format('YYYY-MM-DD HH:mm') },
  {
    title: t('users.cols.actions'), key: 'actions', width: 260,
    render(r) {
      const isMe = auth.user?.id === r.id
      return h(NSpace, { size: 'small' }, () => [
        h(NButton, { size: 'small', onClick: () => openReset(r) }, () => t('users.resetPassword')),
        h(
          NButton,
          { size: 'small', type: 'error', ghost: true, disabled: isMe, onClick: () => confirmDelete(r) },
          () => t('common.delete'),
        ),
      ])
    },
  },
])
</script>

<template>
  <div class="page-header">
    <h2 class="page-title">{{ t('users.title') }}</h2>
    <NButton type="primary" @click="createOpen = true">{{ t('users.newUser') }}</NButton>
  </div>

  <NDataTable
    v-if="!isMobile"
    :columns="columns"
    :data="rows"
    :loading="loading"
    :row-key="(r: UserRow) => r.id"
  />
  <div v-else class="m-list">
    <MobileRowCard v-for="r in rows" :key="r.id">
      <template #title>
        {{ r.username }}
        <NTag v-if="auth.user?.id === r.id" size="small" type="info" class="me-tag">
          {{ t('common.you') }}
        </NTag>
      </template>
      <template #body>
        <div class="m-line">
          <NTag size="small" type="info">{{ dict.label('user.role', r.role) }}</NTag>
          <span class="muted">{{ dayjs(r.created_at).format('YYYY-MM-DD HH:mm') }}</span>
        </div>
      </template>
      <template #actions>
        <NButton size="small" @click="openReset(r)">{{ t('users.resetPassword') }}</NButton>
        <NButton
          size="small"
          type="error"
          ghost
          :disabled="auth.user?.id === r.id"
          @click="confirmDelete(r)"
        >
          {{ t('common.delete') }}
        </NButton>
      </template>
    </MobileRowCard>
  </div>

  <NModal
    v-model:show="createOpen"
    preset="card"
    :title="t('users.createTitle')"
    :style="{ width: isMobile ? '94vw' : '420px', maxWidth: '94vw' }"
  >
    <NForm label-placement="top">
      <NFormItem :label="t('users.usernameLabel')">
        <NInput v-model:value="createForm.username" />
      </NFormItem>
      <NFormItem :label="t('users.passwordLabel')">
        <NInput v-model:value="createForm.password" type="password" show-password-on="click" />
      </NFormItem>
      <NSpace justify="end">
        <NButton @click="createOpen = false">{{ t('common.cancel') }}</NButton>
        <NButton type="primary" :loading="creating" @click="doCreate">{{ t('common.create') }}</NButton>
      </NSpace>
    </NForm>
  </NModal>

  <NModal
    v-model:show="resetOpen"
    preset="card"
    :title="t('users.resetTitle', { name: resetTarget?.username ?? '' })"
    :style="{ width: isMobile ? '94vw' : '420px', maxWidth: '94vw' }"
  >
    <NForm label-placement="top">
      <NFormItem :label="t('users.newPasswordLabel')">
        <NInput v-model:value="resetPwd" type="password" show-password-on="click" />
      </NFormItem>
      <NSpace justify="end">
        <NButton @click="resetOpen = false">{{ t('common.cancel') }}</NButton>
        <NButton type="primary" :loading="resetting" @click="doReset">{{ t('users.resetPassword') }}</NButton>
      </NSpace>
    </NForm>
  </NModal>
</template>

<style scoped>
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
  flex-wrap: wrap;
  gap: 8px;
}
.page-title { margin: 0; }
.me-tag { margin-left: 8px; }
.m-line { display: flex; flex-wrap: wrap; gap: 8px 10px; align-items: center; }
.muted { opacity: 0.7; font-size: 12px; }
</style>
