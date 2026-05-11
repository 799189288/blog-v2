<script setup lang="ts">
import { h, onMounted, ref } from 'vue'
import {
  NDataTable, NTag, NButton, NSpace, NModal, NForm, NFormItem, NInput,
  useDialog, useMessage,
} from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import dayjs from 'dayjs'
import * as usersApi from '../api/users'
import type { UserRow } from '../types'
import { useAuthStore } from '../stores/auth'

const auth = useAuthStore()
const dialog = useDialog()
const message = useMessage()

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

onMounted(load)

async function doCreate() {
  if (!createForm.value.username.trim() || createForm.value.password.length < 8) {
    message.warning('Username + password (≥8 chars) required')
    return
  }
  creating.value = true
  try {
    await usersApi.create(createForm.value.username.trim(), createForm.value.password)
    message.success('User created')
    createOpen.value = false
    createForm.value = { username: '', password: '' }
    await load()
  } catch (e: any) {
    message.error(e?.response?.data?.error ?? 'Create failed')
  } finally { creating.value = false }
}

function openReset(u: UserRow) {
  resetTarget.value = u
  resetPwd.value = ''
  resetOpen.value = true
}

async function doReset() {
  if (!resetTarget.value || resetPwd.value.length < 8) {
    message.warning('Password must be ≥8 characters')
    return
  }
  resetting.value = true
  try {
    await usersApi.resetPassword(resetTarget.value.id, resetPwd.value)
    message.success(`Password reset for ${resetTarget.value.username}`)
    resetOpen.value = false
  } catch (e: any) {
    message.error(e?.response?.data?.error ?? 'Reset failed')
  } finally { resetting.value = false }
}

function confirmDelete(u: UserRow) {
  dialog.warning({
    title: 'Delete user',
    content: `Delete user "${u.username}"? This cannot be undone.`,
    positiveText: 'Delete',
    negativeText: 'Cancel',
    onPositiveClick: async () => {
      try {
        await usersApi.remove(u.id)
        message.success('Deleted')
        await load()
      } catch (e: any) {
        message.error(e?.response?.data?.error ?? 'Delete failed')
      }
    },
  })
}

const columns: DataTableColumns<UserRow> = [
  { title: 'ID', key: 'id', width: 70 },
  {
    title: 'Username', key: 'username',
    render(r) {
      const isMe = auth.user?.id === r.id
      return h('span', null, [r.username, isMe ? h(NTag, { size: 'small', type: 'info', style: 'margin-left:8px' }, () => 'you') : null])
    },
  },
  { title: 'Role', key: 'role', width: 120, render: r => h(NTag, { size: 'small', type: 'info' }, () => r.role) },
  { title: 'Created', key: 'created_at', width: 160, render: r => dayjs(r.created_at).format('YYYY-MM-DD HH:mm') },
  {
    title: 'Actions', key: 'actions', width: 260,
    render(r) {
      const isMe = auth.user?.id === r.id
      return h(NSpace, { size: 'small' }, () => [
        h(NButton, { size: 'small', onClick: () => openReset(r) }, () => 'Reset password'),
        h(
          NButton,
          { size: 'small', type: 'error', ghost: true, disabled: isMe, onClick: () => confirmDelete(r) },
          () => 'Delete',
        ),
      ])
    },
  },
]
</script>

<template>
  <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
    <h2 style="margin: 0">User management</h2>
    <NButton type="primary" @click="createOpen = true">New user</NButton>
  </div>

  <NDataTable :columns="columns" :data="rows" :loading="loading" :row-key="(r: UserRow) => r.id" />

  <NModal v-model:show="createOpen" preset="card" title="Create admin user" style="width: 420px;">
    <NForm label-placement="top">
      <NFormItem label="Username">
        <NInput v-model:value="createForm.username" />
      </NFormItem>
      <NFormItem label="Password (≥8 chars)">
        <NInput v-model:value="createForm.password" type="password" show-password-on="click" />
      </NFormItem>
      <NSpace justify="end">
        <NButton @click="createOpen = false">Cancel</NButton>
        <NButton type="primary" :loading="creating" @click="doCreate">Create</NButton>
      </NSpace>
    </NForm>
  </NModal>

  <NModal v-model:show="resetOpen" preset="card" :title="`Reset password — ${resetTarget?.username ?? ''}`" style="width: 420px;">
    <NForm label-placement="top">
      <NFormItem label="New password (≥8 chars)">
        <NInput v-model:value="resetPwd" type="password" show-password-on="click" />
      </NFormItem>
      <NSpace justify="end">
        <NButton @click="resetOpen = false">Cancel</NButton>
        <NButton type="primary" :loading="resetting" @click="doReset">Reset</NButton>
      </NSpace>
    </NForm>
  </NModal>
</template>
