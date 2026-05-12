<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import {
  NForm, NFormItem, NInput, NDynamicTags, NSelect, NButton, NSpace, NSpin, useMessage,
} from 'naive-ui'
import { MdEditor } from 'md-editor-v3'
import { useI18n } from 'vue-i18n'
import * as postsApi from '../../api/posts'
import { useDictStore } from '../../stores/dict'

const props = defineProps<{ id?: string }>()
const router = useRouter()
const message = useMessage()
const { t } = useI18n()
const dict = useDictStore()

const isEdit = computed(() => !!props.id)
const loading = ref(false)
const saving = ref(false)

const form = reactive({
  title: '',
  slug: '',
  excerpt: '',
  content_md: '',
  status: 'draft' as 'draft' | 'published',
  tags: [] as string[],
})

const statusOptions = computed(() => dict.options('post.status').value)

async function load() {
  if (!props.id) return
  loading.value = true
  try {
    const p = await postsApi.adminGet(Number(props.id))
    form.title = p.title
    form.slug = p.slug
    form.excerpt = p.excerpt ?? ''
    form.content_md = p.content_md
    form.status = p.status
    form.tags = p.tags.map(tg => tg.name)
  } catch (e: any) {
    message.error(e?.response?.data?.error ?? t('postEdit.loadFailed'))
  } finally {
    loading.value = false
  }
}

onMounted(async () => {
  await dict.ensure('post.status')
  await load()
})

async function save() {
  if (!form.title.trim() || !form.content_md.trim()) {
    message.warning(t('postEdit.requiredError'))
    return
  }
  saving.value = true
  try {
    const payload = {
      title: form.title.trim(),
      slug: form.slug.trim() || undefined,
      excerpt: form.excerpt.trim() || undefined,
      content_md: form.content_md,
      status: form.status,
      tags: form.tags,
    }
    if (isEdit.value) {
      await postsApi.adminUpdate(Number(props.id), payload)
      message.success(t('postEdit.saved'))
    } else {
      const created = await postsApi.adminCreate(payload)
      message.success(t('postEdit.created'))
      router.replace({ name: 'manage-post-edit', params: { id: created.id } })
    }
  } catch (e: any) {
    message.error(e?.response?.data?.error ?? t('common.saveFailed'))
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <NSpin :show="loading">
    <h2 style="margin-top: 0">{{ isEdit ? t('postEdit.titleEdit') : t('postEdit.titleNew') }}</h2>
    <NForm label-placement="top">
      <NFormItem :label="t('postEdit.fields.title')" required>
        <NInput v-model:value="form.title" maxlength="200" />
      </NFormItem>
      <NFormItem :label="t('postEdit.fields.slug')">
        <NInput v-model:value="form.slug" :placeholder="t('postEdit.fields.slugPlaceholder')" />
      </NFormItem>
      <NFormItem :label="t('postEdit.fields.excerpt')">
        <NInput v-model:value="form.excerpt" type="textarea" :rows="2" />
      </NFormItem>
      <NFormItem :label="t('postEdit.fields.tags')">
        <NDynamicTags v-model:value="form.tags" />
      </NFormItem>
      <NFormItem :label="t('postEdit.fields.status')">
        <NSelect v-model:value="form.status" :options="statusOptions" style="max-width: 200px;" />
      </NFormItem>
      <NFormItem :label="t('postEdit.fields.content')" required>
        <MdEditor v-model="form.content_md" :preview-only="false" style="height: 540px; width: 100%;" />
      </NFormItem>
      <NSpace>
        <NButton type="primary" :loading="saving" @click="save">{{ t('common.save') }}</NButton>
        <NButton @click="router.push({ name: 'manage-posts' })">{{ t('common.cancel') }}</NButton>
      </NSpace>
    </NForm>
  </NSpin>
</template>
