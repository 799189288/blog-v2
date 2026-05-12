<script setup lang="ts">
import { computed, h, onMounted, ref } from 'vue'
import {
  NCard, NDataTable, NTag, NButton, NSpace, NModal, NForm, NFormItem, NInput, NInputNumber, NSwitch, NEmpty,
  useDialog, useMessage,
} from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import * as dictApi from '../api/dict'
import type { DictType, DictItem } from '../api/dict'
import { useDictStore } from '../stores/dict'

const { t } = useI18n()
const dialog = useDialog()
const message = useMessage()
const dictStore = useDictStore()

const types = ref<DictType[]>([])
const selected = ref<DictType | null>(null)
const items = ref<DictItem[]>([])
const loadingTypes = ref(false)
const loadingItems = ref(false)

// ----- Type modal -----
const typeOpen = ref(false)
const editingType = ref<DictType | null>(null)
const typeForm = ref({ code: '', name_zh: '', name_en: '' })
const typeSaving = ref(false)

// ----- Item modal -----
const itemOpen = ref(false)
const editingItem = ref<DictItem | null>(null)
const itemForm = ref({ code: '', label_zh: '', label_en: '', sort: 0, enabled: true })
const itemSaving = ref(false)

async function loadTypes() {
  loadingTypes.value = true
  try {
    types.value = await dictApi.adminListTypes()
    if (selected.value) {
      // Refresh the selected reference if we edited its row.
      selected.value = types.value.find(t => t.id === selected.value?.id) ?? null
    }
    if (!selected.value && types.value.length > 0) {
      await selectType(types.value[0])
    }
  } finally {
    loadingTypes.value = false
  }
}

async function selectType(row: DictType) {
  selected.value = row
  await loadItems()
}

async function loadItems() {
  if (!selected.value) {
    items.value = []
    return
  }
  loadingItems.value = true
  try {
    items.value = await dictApi.adminListItems(selected.value.id)
  } finally {
    loadingItems.value = false
  }
}

onMounted(loadTypes)

// ----- Type ops -----

function openCreateType() {
  editingType.value = null
  typeForm.value = { code: '', name_zh: '', name_en: '' }
  typeOpen.value = true
}

function openEditType(row: DictType) {
  editingType.value = row
  typeForm.value = { code: row.code, name_zh: row.name_zh, name_en: row.name_en }
  typeOpen.value = true
}

async function saveType() {
  const f = typeForm.value
  if (!f.code.trim() || !f.name_zh.trim() || !f.name_en.trim()) {
    message.warning(t('manageDict.requiredError'))
    return
  }
  typeSaving.value = true
  try {
    const payload = {
      code: f.code.trim(),
      name_zh: f.name_zh.trim(),
      name_en: f.name_en.trim(),
    }
    if (editingType.value) {
      const updated = await dictApi.adminUpdateType(editingType.value.id, payload)
      message.success(t('postEdit.saved'))
      // If the type's items might be affected by a code change, refresh.
      if (editingType.value.code !== updated.code) {
        await dictStore.reload(editingType.value.code)
      }
      await dictStore.reload(updated.code)
    } else {
      await dictApi.adminCreateType(payload)
      message.success(t('postEdit.created'))
    }
    typeOpen.value = false
    await loadTypes()
  } catch (e: any) {
    message.error(e?.response?.data?.error ?? t('common.saveFailed'))
  } finally {
    typeSaving.value = false
  }
}

function confirmDeleteType(row: DictType) {
  if (row.is_system) {
    message.warning(t('manageDict.systemTypeProtected'))
    return
  }
  dialog.warning({
    title: t('manageDict.deleteTypeTitle'),
    content: t('manageDict.deleteTypeConfirm', { name: row.name_zh }),
    positiveText: t('common.confirmDelete'),
    negativeText: t('common.cancel'),
    onPositiveClick: async () => {
      try {
        await dictApi.adminDeleteType(row.id)
        await dictStore.reload(row.code)
        message.success(t('common.deleted'))
        if (selected.value?.id === row.id) selected.value = null
        await loadTypes()
      } catch (e: any) {
        message.error(e?.response?.data?.error ?? t('common.deleteFailed'))
      }
    },
  })
}

// ----- Item ops -----

function openCreateItem() {
  if (!selected.value) return
  editingItem.value = null
  itemForm.value = { code: '', label_zh: '', label_en: '', sort: 0, enabled: true }
  itemOpen.value = true
}

function openEditItem(row: DictItem) {
  editingItem.value = row
  itemForm.value = {
    code: row.code,
    label_zh: row.label_zh,
    label_en: row.label_en,
    sort: row.sort,
    enabled: row.enabled,
  }
  itemOpen.value = true
}

async function saveItem() {
  if (!selected.value) return
  const f = itemForm.value
  if (!f.code.trim() || !f.label_zh.trim() || !f.label_en.trim()) {
    message.warning(t('manageDict.requiredError'))
    return
  }
  itemSaving.value = true
  try {
    const payload = {
      code: f.code.trim(),
      label_zh: f.label_zh.trim(),
      label_en: f.label_en.trim(),
      sort: f.sort,
      enabled: f.enabled,
    }
    if (editingItem.value) {
      await dictApi.adminUpdateItem(editingItem.value.id, payload)
      message.success(t('postEdit.saved'))
    } else {
      await dictApi.adminCreateItem(selected.value.id, payload)
      message.success(t('postEdit.created'))
    }
    await dictStore.reload(selected.value.code)
    itemOpen.value = false
    await loadItems()
  } catch (e: any) {
    message.error(e?.response?.data?.error ?? t('common.saveFailed'))
  } finally {
    itemSaving.value = false
  }
}

function confirmDeleteItem(row: DictItem) {
  if (selected.value?.is_system) {
    message.warning(t('manageDict.systemItemProtected'))
    return
  }
  dialog.warning({
    title: t('manageDict.deleteItemTitle'),
    content: t('manageDict.deleteItemConfirm', { code: row.code }),
    positiveText: t('common.confirmDelete'),
    negativeText: t('common.cancel'),
    onPositiveClick: async () => {
      try {
        await dictApi.adminDeleteItem(row.id)
        if (selected.value) await dictStore.reload(selected.value.code)
        message.success(t('common.deleted'))
        await loadItems()
      } catch (e: any) {
        message.error(e?.response?.data?.error ?? t('common.deleteFailed'))
      }
    },
  })
}

// ----- Columns -----

const typeColumns = computed<DataTableColumns<DictType>>(() => [
  {
    title: t('manageDict.typeCols.code'), key: 'code',
    render(row) {
      const isSelected = selected.value?.id === row.id
      return h(
        'a',
        {
          style: `cursor:pointer; ${isSelected ? 'font-weight:600' : ''}`,
          onClick: () => selectType(row),
        },
        [
          row.code,
          row.is_system ? h(NTag, { size: 'small', type: 'info', style: 'margin-left: 6px' }, () => 'system') : null,
        ],
      )
    },
  },
  {
    title: t('manageDict.typeCols.name'), key: 'name',
    render(row) { return h('span', null, `${row.name_zh} / ${row.name_en}`) },
  },
  {
    title: t('manageDict.typeCols.actions'), key: 'actions', width: 160,
    render(row) {
      return h(NSpace, { size: 'small' }, () => [
        h(NButton, { size: 'small', onClick: () => openEditType(row) }, () => t('common.edit')),
        h(
          NButton,
          { size: 'small', type: 'error', ghost: true, disabled: row.is_system, onClick: () => confirmDeleteType(row) },
          () => t('common.delete'),
        ),
      ])
    },
  },
])

const itemColumns = computed<DataTableColumns<DictItem>>(() => [
  { title: t('manageDict.itemCols.code'), key: 'code', width: 200 },
  { title: t('manageDict.itemCols.labelZh'), key: 'label_zh' },
  { title: t('manageDict.itemCols.labelEn'), key: 'label_en' },
  { title: t('manageDict.itemCols.sort'), key: 'sort', width: 80 },
  {
    title: t('manageDict.itemCols.enabled'), key: 'enabled', width: 90,
    render(row) {
      return row.enabled
        ? h(NTag, { size: 'small', type: 'success' }, () => t('manageDict.enabled'))
        : h(NTag, { size: 'small' }, () => t('manageDict.disabled'))
    },
  },
  {
    title: t('manageDict.itemCols.actions'), key: 'actions', width: 160,
    render(row) {
      const sys = selected.value?.is_system ?? false
      return h(NSpace, { size: 'small' }, () => [
        h(NButton, { size: 'small', onClick: () => openEditItem(row) }, () => t('common.edit')),
        h(
          NButton,
          { size: 'small', type: 'error', ghost: true, disabled: sys, onClick: () => confirmDeleteItem(row) },
          () => t('common.delete'),
        ),
      ])
    },
  },
])

const codeReadonly = computed(() => {
  // For system types, item code cannot be edited (backend rejects), so disable input.
  return !!editingItem.value && (selected.value?.is_system ?? false)
})
const typeCodeReadonly = computed(() => !!editingType.value && (editingType.value.is_system ?? false))
</script>

<template>
  <h2 style="margin-top: 0">{{ t('manageDict.title') }}</h2>

  <div class="dict-layout">
    <NCard class="types-card" :title="t('manageDict.types')">
      <template #header-extra>
        <NButton size="small" type="primary" @click="openCreateType">{{ t('manageDict.newType') }}</NButton>
      </template>
      <NDataTable
        :columns="typeColumns"
        :data="types"
        :loading="loadingTypes"
        :row-key="(r: DictType) => r.id"
        size="small"
      />
    </NCard>

    <NCard class="items-card" :title="selected ? `${selected.name_zh} (${selected.code})` : t('manageDict.itemsTitle')">
      <template #header-extra>
        <NButton
          v-if="selected"
          size="small"
          type="primary"
          @click="openCreateItem"
        >
          {{ t('manageDict.newItem') }}
        </NButton>
      </template>
      <NEmpty v-if="!selected" :description="t('manageDict.pickType')" />
      <NDataTable
        v-else
        :columns="itemColumns"
        :data="items"
        :loading="loadingItems"
        :row-key="(r: DictItem) => r.id"
        size="small"
      />
    </NCard>
  </div>

  <!-- Type modal -->
  <NModal
    v-model:show="typeOpen"
    preset="card"
    :title="editingType ? t('manageDict.editTypeTitle') : t('manageDict.createTypeTitle')"
    style="width: 480px;"
  >
    <NForm label-placement="top">
      <NFormItem :label="t('manageDict.fields.code')" required>
        <NInput v-model:value="typeForm.code" :disabled="typeCodeReadonly" :placeholder="t('manageDict.fields.codePlaceholder')" />
      </NFormItem>
      <NFormItem :label="t('manageDict.fields.nameZh')" required>
        <NInput v-model:value="typeForm.name_zh" />
      </NFormItem>
      <NFormItem :label="t('manageDict.fields.nameEn')" required>
        <NInput v-model:value="typeForm.name_en" />
      </NFormItem>
      <NSpace justify="end">
        <NButton @click="typeOpen = false">{{ t('common.cancel') }}</NButton>
        <NButton type="primary" :loading="typeSaving" @click="saveType">{{ t('common.save') }}</NButton>
      </NSpace>
    </NForm>
  </NModal>

  <!-- Item modal -->
  <NModal
    v-model:show="itemOpen"
    preset="card"
    :title="editingItem ? t('manageDict.editItemTitle') : t('manageDict.createItemTitle')"
    style="width: 480px;"
  >
    <NForm label-placement="top">
      <NFormItem :label="t('manageDict.fields.code')" required>
        <NInput v-model:value="itemForm.code" :disabled="codeReadonly" :placeholder="t('manageDict.fields.codePlaceholder')" />
      </NFormItem>
      <NFormItem :label="t('manageDict.fields.labelZh')" required>
        <NInput v-model:value="itemForm.label_zh" />
      </NFormItem>
      <NFormItem :label="t('manageDict.fields.labelEn')" required>
        <NInput v-model:value="itemForm.label_en" />
      </NFormItem>
      <NFormItem :label="t('manageDict.fields.sort')">
        <NInputNumber v-model:value="itemForm.sort" />
      </NFormItem>
      <NFormItem :label="t('manageDict.fields.enabled')">
        <NSwitch v-model:value="itemForm.enabled" />
      </NFormItem>
      <NSpace justify="end">
        <NButton @click="itemOpen = false">{{ t('common.cancel') }}</NButton>
        <NButton type="primary" :loading="itemSaving" @click="saveItem">{{ t('common.save') }}</NButton>
      </NSpace>
    </NForm>
  </NModal>
</template>

<style scoped>
.dict-layout {
  display: grid;
  grid-template-columns: minmax(360px, 1fr) 2fr;
  gap: 16px;
  align-items: start;
}
@media (max-width: 1100px) {
  .dict-layout { grid-template-columns: 1fr; }
}
.types-card :deep(.n-data-table) { font-size: 13px; }
</style>
