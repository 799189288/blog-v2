<script setup lang="ts">
import { computed, h, onMounted, ref } from 'vue'
import {
  NDataTable, NTag, NButton, NSpace, NModal, NForm, NFormItem, NInput,
  useDialog, useMessage,
} from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import * as tagsApi from '../../api/tags'
import type { TagBrowseRow } from '../../types'

const dialog = useDialog()
const message = useMessage()
const { t } = useI18n()

const rows = ref<TagBrowseRow[]>([])
const loading = ref(false)

const editorOpen = ref(false)
const editing = ref<TagBrowseRow | null>(null)
const form = ref({ name: '', slug: '' })
const saving = ref(false)

async function load() {
  loading.value = true
  try {
    rows.value = await tagsApi.adminList()
  } finally {
    loading.value = false
  }
}
onMounted(load)

function openCreate() {
  editing.value = null
  form.value = { name: '', slug: '' }
  editorOpen.value = true
}

function openEdit(row: TagBrowseRow) {
  editing.value = row
  form.value = { name: row.name, slug: row.slug }
  editorOpen.value = true
}

async function save() {
  if (!form.value.name.trim()) {
    message.warning(t('manageTags.nameRequired'))
    return
  }
  saving.value = true
  try {
    const payload = {
      name: form.value.name.trim(),
      slug: form.value.slug.trim() || undefined,
    }
    if (editing.value) {
      await tagsApi.adminUpdate(editing.value.id, payload)
      message.success(t('postEdit.saved'))
    } else {
      await tagsApi.adminCreate(payload)
      message.success(t('postEdit.created'))
    }
    editorOpen.value = false
    await load()
  } catch (e: any) {
    message.error(e?.response?.data?.error ?? t('common.saveFailed'))
  } finally {
    saving.value = false
  }
}

function confirmDelete(row: TagBrowseRow) {
  dialog.warning({
    title: t('manageTags.deleteTitle'),
    content: row.post_count > 0
      ? t('manageTags.deleteWithPostsConfirm', { name: row.name, count: row.post_count })
      : t('manageTags.deleteConfirm', { name: row.name }),
    positiveText: t('common.confirmDelete'),
    negativeText: t('common.cancel'),
    onPositiveClick: async () => {
      try {
        await tagsApi.adminDelete(row.id)
        message.success(t('common.deleted'))
        await load()
      } catch (e: any) {
        message.error(e?.response?.data?.error ?? t('common.deleteFailed'))
      }
    },
  })
}

const columns = computed<DataTableColumns<TagBrowseRow>>(() => [
  { title: t('manageTags.cols.id'), key: 'id', width: 80 },
  { title: t('manageTags.cols.name'), key: 'name' },
  { title: t('manageTags.cols.slug'), key: 'slug' },
  {
    title: t('manageTags.cols.posts'), key: 'post_count', width: 120,
    render: r => h(NTag, { size: 'small', type: r.post_count > 0 ? 'info' : 'default' }, () => r.post_count),
  },
  {
    title: t('manageTags.cols.actions'), key: 'actions', width: 200,
    render(row) {
      return h(NSpace, { size: 'small' }, () => [
        h(NButton, { size: 'small', onClick: () => openEdit(row) }, () => t('common.edit')),
        h(NButton, { size: 'small', type: 'error', ghost: true, onClick: () => confirmDelete(row) }, () => t('common.delete')),
      ])
    },
  },
])
</script>

<template>
  <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
    <h2 style="margin: 0">{{ t('manageTags.title') }}</h2>
    <NButton type="primary" @click="openCreate">{{ t('manageTags.newTag') }}</NButton>
  </div>

  <NDataTable
    :columns="columns"
    :data="rows"
    :loading="loading"
    :row-key="(r: TagBrowseRow) => r.id"
    :pagination="{ pageSize: 50 }"
  />

  <NModal
    v-model:show="editorOpen"
    preset="card"
    :title="editing ? t('manageTags.editTitle') : t('manageTags.createTitle')"
    style="width: 480px;"
  >
    <NForm label-placement="top">
      <NFormItem :label="t('manageTags.fields.name')" required>
        <NInput v-model:value="form.name" maxlength="64" />
      </NFormItem>
      <NFormItem :label="t('manageTags.fields.slug')">
        <NInput v-model:value="form.slug" :placeholder="t('manageTags.fields.slugPlaceholder')" />
      </NFormItem>
      <NSpace justify="end">
        <NButton @click="editorOpen = false">{{ t('common.cancel') }}</NButton>
        <NButton type="primary" :loading="saving" @click="save">{{ t('common.save') }}</NButton>
      </NSpace>
    </NForm>
  </NModal>
</template>
