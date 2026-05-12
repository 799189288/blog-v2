<script setup lang="ts">
import { computed, h, onMounted, ref } from 'vue'
import {
  NDataTable, NTag, NButton, NSpace, NModal, NForm, NFormItem, NInput,
  useDialog, useMessage,
} from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import dayjs from 'dayjs'
import * as usersApi from '../api/users'
import type { UserRow } from '../types'
import { useAuthStore } from '../stores/auth'
import { useDictStore } from '../stores/dict'

const auth = useAuthStore()
const dialog = useDialog()
const message = useMessage()
const { t } = useI18n()
const dict = useDictStore()

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
  <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
    <h2 style="margin: 0">{{ t('users.title') }}</h2>
    <NButton type="primary" @click="createOpen = true">{{ t('users.newUser') }}</NButton>
  </div>

  <NDataTable :columns="columns" :data="rows" :loading="loading" :row-key="(r: UserRow) => r.id" />

  <NModal v-model:show="createOpen" preset="card" :title="t('users.createTitle')" style="width: 420px;">
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

  <NModal v-model:show="resetOpen" preset="card" :title="t('users.resetTitle', { name: resetTarget?.username ?? '' })" style="width: 420px;">
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
