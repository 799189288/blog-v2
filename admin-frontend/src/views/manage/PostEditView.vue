<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import {
  NForm, NFormItem, NInput, NDynamicTags, NSelect, NButton, NSpace, NSpin, useMessage,
} from 'naive-ui'
import { MdEditor } from 'md-editor-v3'
import * as postsApi from '../../api/posts'

const props = defineProps<{ id?: string }>()
const router = useRouter()
const message = useMessage()

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

const statusOptions = [
  { label: 'Draft', value: 'draft' },
  { label: 'Published', value: 'published' },
]

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
    form.tags = p.tags.map(t => t.name)
  } catch (e: any) {
    message.error(e?.response?.data?.error ?? 'Failed to load post')
  } finally {
    loading.value = false
  }
}

onMounted(load)

async function save() {
  if (!form.title.trim() || !form.content_md.trim()) {
    message.warning('Title and content are required')
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
      message.success('Updated')
    } else {
      const created = await postsApi.adminCreate(payload)
      message.success('Created')
      router.replace({ name: 'manage-post-edit', params: { id: created.id } })
    }
  } catch (e: any) {
    message.error(e?.response?.data?.error ?? 'Save failed')
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <NSpin :show="loading">
    <h2 style="margin-top: 0">{{ isEdit ? 'Edit post' : 'New post' }}</h2>
    <NForm label-placement="top">
      <NFormItem label="Title" required>
        <NInput v-model:value="form.title" maxlength="200" />
      </NFormItem>
      <NFormItem label="Slug (optional — generated from title if empty)">
        <NInput v-model:value="form.slug" placeholder="my-post-slug" />
      </NFormItem>
      <NFormItem label="Excerpt (optional)">
        <NInput v-model:value="form.excerpt" type="textarea" :rows="2" />
      </NFormItem>
      <NFormItem label="Tags">
        <NDynamicTags v-model:value="form.tags" />
      </NFormItem>
      <NFormItem label="Status">
        <NSelect v-model:value="form.status" :options="statusOptions" style="max-width: 200px;" />
      </NFormItem>
      <NFormItem label="Content (Markdown)" required>
        <MdEditor v-model="form.content_md" :preview-only="false" style="height: 540px; width: 100%;" />
      </NFormItem>
      <NSpace>
        <NButton type="primary" :loading="saving" @click="save">Save</NButton>
        <NButton @click="router.push({ name: 'manage-posts' })">Cancel</NButton>
      </NSpace>
    </NForm>
  </NSpin>
</template>
